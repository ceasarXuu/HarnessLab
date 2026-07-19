from __future__ import annotations

import asyncio
from urllib.parse import urlsplit

import pytest

from ornnlab.services.container_proxy_runtime import (
    ContainerProxyRuntime,
    ProxyConfigurationError,
    RuntimeProxyPolicy,
)
from ornnlab.services.docker_proxy_target import DockerProxyTarget
from ornnlab.services.failure_classifier import classify_exception
from ornnlab.services.harbor_engine import HarborConfigBuilder


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.mark.anyio
async def test_non_loopback_proxy_is_inherited_without_relay() -> None:
    runtime = ContainerProxyRuntime(
        environ={
            "https_proxy": "http://10.10.0.8:8080",
            "NO_PROXY": "localhost,127.0.0.1",
        },
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    policy = await runtime.prepare_policy()

    assert policy.relay_count == 0
    assert policy.agent_env_defaults == {
        "HTTPS_PROXY": "${ORNNLAB_CONTAINER_HTTPS_PROXY}",
        "https_proxy": "${ORNNLAB_CONTAINER_HTTPS_PROXY}",
        "NO_PROXY": "${ORNNLAB_CONTAINER_NO_PROXY}",
        "no_proxy": "${ORNNLAB_CONTAINER_NO_PROXY}",
    }
    assert policy.subprocess_env == {
        "ORNNLAB_CONTAINER_HTTPS_PROXY": "http://10.10.0.8:8080",
        "ORNNLAB_CONTAINER_NO_PROXY": "localhost,127.0.0.1",
    }
    await runtime.close()


@pytest.mark.anyio
async def test_remote_target_can_inherit_independently_addressed_proxy_without_host_relay() -> None:
    def target_resolver() -> DockerProxyTarget:
        raise AssertionError("direct proxy must not require Docker discovery")

    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://proxy.example:8080"},
        docker_target_resolver=target_resolver,
    )

    policy = await runtime.prepare_policy()

    assert policy.strategy == "direct"
    assert policy.target_kind is None
    assert policy.subprocess_env["ORNNLAB_CONTAINER_HTTPS_PROXY"] == (
        "http://proxy.example:8080"
    )


@pytest.mark.anyio
async def test_loopback_proxy_is_relayed_on_docker_reachable_address() -> None:
    received = bytearray()

    async def echo(reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        payload = await reader.read(1024)
        received.extend(payload)
        writer.write(payload)
        await writer.drain()
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(echo, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{upstream_port}"},
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    policy = await runtime.prepare_policy()
    relay = urlsplit(policy.subprocess_env["ORNNLAB_CONTAINER_HTTPS_PROXY"])
    reader, writer = await asyncio.open_connection(relay.hostname, relay.port)
    writer.write(b"proxy-smoke")
    await writer.drain()

    assert await reader.read(1024) == b"proxy-smoke"
    assert received == b"proxy-smoke"
    assert policy.agent_env_defaults["https_proxy"] == ("${ORNNLAB_CONTAINER_HTTPS_PROXY}")
    assert policy.relay_count == 1

    writer.close()
    await writer.wait_closed()
    await policy.close()
    with pytest.raises(OSError):
        await asyncio.open_connection(relay.hostname, relay.port)
    await runtime.close()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_loopback_proxy_credentials_are_rejected_without_leaking_secret() -> None:
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://alice:top-secret@127.0.0.1:7890"},
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    with pytest.raises(ProxyConfigurationError) as raised:
        await runtime.prepare_policy()

    assert "top-secret" not in str(raised.value)
    await runtime.close()


@pytest.mark.anyio
async def test_proxy_auto_detection_can_be_disabled() -> None:
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://127.0.0.1:7890"},
        mode="off",
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    policy = await runtime.prepare_policy()

    assert policy.agent_env_defaults == {}
    assert policy.subprocess_env == {}
    assert policy.relay_count == 0
    await runtime.close()


@pytest.mark.anyio
async def test_conflicting_proxy_case_variants_fail_fast_without_values() -> None:
    runtime = ContainerProxyRuntime(
        environ={
            "HTTPS_PROXY": "http://first.internal:8080",
            "https_proxy": "http://second.internal:8080",
        },
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    with pytest.raises(ProxyConfigurationError) as raised:
        await runtime.prepare_policy()

    assert str(raised.value) == (
        "HTTPS_PROXY and https_proxy disagree; make them identical or unset one"
    )
    await runtime.close()


@pytest.mark.anyio
@pytest.mark.parametrize("target_kind", ["docker-desktop", "rootless", "remote-or-virtualized"])
async def test_loopback_proxy_rejects_targets_without_safe_host_relay(target_kind: str) -> None:
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://127.0.0.1:7890"},
        docker_target_resolver=lambda: DockerProxyTarget(target_kind, "test", "remote-tcp"),
    )

    with pytest.raises(ProxyConfigurationError, match=target_kind):
        await runtime.prepare_policy()


@pytest.mark.anyio
async def test_explicit_proxy_group_skips_conflicting_automatic_discovery() -> None:
    resolver_called = False

    def target_resolver() -> DockerProxyTarget:
        nonlocal resolver_called
        resolver_called = True
        return DockerProxyTarget("remote-or-virtualized", "remote", "remote-ssh")

    runtime = ContainerProxyRuntime(
        environ={
            "HTTPS_PROXY": "http://first.internal:8080",
            "https_proxy": "http://second.internal:8080",
        },
        docker_target_resolver=target_resolver,
    )

    policy = await runtime.prepare_policy({"HTTPS_PROXY"})

    assert policy.agent_env_defaults == {}
    assert policy.subprocess_env == {}
    assert resolver_called is False


@pytest.mark.anyio
async def test_network_allowlist_disables_default_automatic_proxy() -> None:
    resolver_called = False

    def target_resolver() -> DockerProxyTarget:
        nonlocal resolver_called
        resolver_called = True
        return _local_target("127.0.0.1")

    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://127.0.0.1:7890"},
        docker_target_resolver=target_resolver,
    )

    policy = await runtime.prepare_policy(automatic_proxy_allowed=False)

    assert policy.agent_env_defaults == {}
    assert resolver_called is False


@pytest.mark.anyio
async def test_cancelled_policy_close_can_be_retried() -> None:
    release_count = 0

    async def release() -> None:
        nonlocal release_count
        release_count += 1
        if release_count == 1:
            raise asyncio.CancelledError

    policy = RuntimeProxyPolicy({}, {}, 0, _release=release)

    with pytest.raises(asyncio.CancelledError):
        await policy.close()
    await policy.close()

    assert release_count == 2


@pytest.mark.anyio
async def test_unbindable_local_gateway_has_stable_proxy_failure() -> None:
    async def accept_and_close(
        _reader: asyncio.StreamReader, writer: asyncio.StreamWriter
    ) -> None:
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(accept_and_close, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{upstream_port}"},
        docker_target_resolver=lambda: _local_target("192.0.2.10"),
    )

    with pytest.raises(ProxyConfigurationError, match="not bindable"):
        await runtime.prepare_policy()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_unavailable_loopback_upstream_fails_before_harbor_start() -> None:
    temporary = await asyncio.start_server(lambda *_: None, "127.0.0.1", 0)
    unused_port = temporary.sockets[0].getsockname()[1]
    temporary.close()
    await temporary.wait_closed()
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{unused_port}"},
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    with pytest.raises(ProxyConfigurationError, match="not accepting connections"):
        await runtime.prepare_policy()


@pytest.mark.anyio
async def test_invalid_gateway_fails_without_leaving_listener() -> None:
    async def accept_and_close(
        _reader: asyncio.StreamReader, writer: asyncio.StreamWriter
    ) -> None:
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(accept_and_close, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{upstream_port}"},
        docker_target_resolver=lambda: _local_target("not-an-ip"),
    )

    with pytest.raises(ProxyConfigurationError, match="invalid host gateway"):
        await runtime.prepare_policy()

    assert runtime._active_servers == set()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_prepare_cancellation_releases_partially_created_relays(monkeypatch) -> None:
    blocker = asyncio.Event()
    second_preflight_started = asyncio.Event()
    preflight_count = 0

    async def controlled_preflight(*_args) -> None:
        nonlocal preflight_count
        preflight_count += 1
        if preflight_count == 2:
            second_preflight_started.set()
            await blocker.wait()

    runtime = ContainerProxyRuntime(
        environ={
            "HTTP_PROXY": "http://127.0.0.1:7001",
            "ALL_PROXY": "socks5://127.0.0.1:7002",
        },
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )
    monkeypatch.setattr(runtime, "_ensure_upstream_available", controlled_preflight)
    preparing = asyncio.create_task(runtime.prepare_policy())
    await second_preflight_started.wait()
    relay_addresses = [server.sockets[0].getsockname() for server in runtime._active_servers]

    preparing.cancel()
    with pytest.raises(asyncio.CancelledError):
        await preparing

    assert runtime._active_servers == set()
    assert len(relay_addresses) == 1
    with pytest.raises(OSError):
        await asyncio.open_connection(*relay_addresses[0])


@pytest.mark.anyio
async def test_same_endpoint_with_different_schemes_uses_distinct_relay_urls() -> None:
    async def accept(
        reader: asyncio.StreamReader, writer: asyncio.StreamWriter
    ) -> None:
        await reader.read()
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(accept, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={
            "HTTP_PROXY": f"http://127.0.0.1:{upstream_port}",
            "ALL_PROXY": f"socks5://127.0.0.1:{upstream_port}",
        },
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )

    policy = await runtime.prepare_policy()

    assert urlsplit(policy.subprocess_env["ORNNLAB_CONTAINER_HTTP_PROXY"]).scheme == "http"
    assert urlsplit(policy.subprocess_env["ORNNLAB_CONTAINER_ALL_PROXY"]).scheme == "socks5"
    assert policy.relay_count == 2
    await policy.close()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_policy_close_terminates_active_relay_connections() -> None:
    async def hold_connection(
        reader: asyncio.StreamReader, writer: asyncio.StreamWriter
    ) -> None:
        await reader.read()
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(hold_connection, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{upstream_port}"},
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )
    policy = await runtime.prepare_policy()
    relay = urlsplit(policy.subprocess_env["ORNNLAB_CONTAINER_HTTPS_PROXY"])
    reader, writer = await asyncio.open_connection(relay.hostname, relay.port)
    await asyncio.sleep(0.05)

    await asyncio.wait_for(policy.close(), timeout=1)

    assert await asyncio.wait_for(reader.read(), timeout=1) == b""
    writer.close()
    await writer.wait_closed()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_closing_one_concurrent_policy_keeps_other_relay_available() -> None:
    async def echo(reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        while payload := await reader.read(1024):
            writer.write(payload)
            await writer.drain()
        writer.close()
        await writer.wait_closed()

    upstream = await asyncio.start_server(echo, "127.0.0.1", 0)
    upstream_port = upstream.sockets[0].getsockname()[1]
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": f"http://127.0.0.1:{upstream_port}"},
        docker_target_resolver=lambda: _local_target("127.0.0.1"),
    )
    first, second = await asyncio.gather(runtime.prepare_policy(), runtime.prepare_policy())
    first_url = urlsplit(first.subprocess_env["ORNNLAB_CONTAINER_HTTPS_PROXY"])
    second_url = urlsplit(second.subprocess_env["ORNNLAB_CONTAINER_HTTPS_PROXY"])

    await first.close()
    with pytest.raises(OSError):
        await asyncio.open_connection(first_url.hostname, first_url.port)
    reader, writer = await asyncio.open_connection(second_url.hostname, second_url.port)
    writer.write(b"still-active")
    await writer.drain()

    assert await reader.read(1024) == b"still-active"
    writer.close()
    await writer.wait_closed()
    await second.close()
    upstream.close()
    await upstream.wait_closed()


def test_proxy_configuration_failure_has_stable_classification() -> None:
    failure = classify_exception(ProxyConfigurationError("Docker bridge is unavailable"))

    assert failure == {
        "failure_class": "proxy_configuration_failure",
        "failure_code": "docker_proxy_unavailable",
        "failure_summary": "Docker bridge is unavailable",
    }


def test_harbor_builder_merges_runtime_proxy_as_agent_defaults(settings) -> None:
    builder = HarborConfigBuilder(settings)

    config = builder.build(
        {"name": "nop"},
        "terminal-bench",
        "2.0",
        1,
        1,
        1,
        str(settings.home / "jobs"),
        overrides={
            "environment": {
                "type": "docker",
                "env": {"HTTPS_PROXY": "http://profile-proxy:9000"},
            }
        },
        runtime_agent_env_defaults={
            "HTTP_PROXY": "http://172.17.0.1:32000",
            "HTTPS_PROXY": "http://172.17.0.1:32000",
            "https_proxy": "http://172.17.0.1:32000",
        },
    )

    assert config.agent["env"] == {
        "HTTP_PROXY": "http://172.17.0.1:32000",
    }
    assert config.environment["env"]["HTTPS_PROXY"] == "http://profile-proxy:9000"


def test_harbor_builder_does_not_inject_proxy_into_non_docker_environment(settings) -> None:
    builder = HarborConfigBuilder(settings)

    config = builder.build(
        {"name": "nop"},
        "terminal-bench",
        "2.0",
        1,
        1,
        1,
        str(settings.home / "jobs"),
        overrides={"environment": {"type": "local"}},
        runtime_agent_env_defaults={"HTTPS_PROXY": "http://172.17.0.1:32000"},
    )

    assert "env" not in config.agent


def _local_target(gateway: str) -> DockerProxyTarget:
    return DockerProxyTarget("local-rootful-linux", "default", "local-unix", gateway)
