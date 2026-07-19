from __future__ import annotations

import asyncio
from urllib.parse import urlsplit

import pytest

from ornnlab.services.container_proxy_runtime import (
    ContainerProxyRuntime,
    ProxyConfigurationError,
)
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
        docker_gateway_resolver=lambda: "127.0.0.1",
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
        docker_gateway_resolver=lambda: "127.0.0.1",
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
    await runtime.close()
    upstream.close()
    await upstream.wait_closed()


@pytest.mark.anyio
async def test_loopback_proxy_credentials_are_rejected_without_leaking_secret() -> None:
    runtime = ContainerProxyRuntime(
        environ={"HTTPS_PROXY": "http://alice:top-secret@127.0.0.1:7890"},
        docker_gateway_resolver=lambda: "127.0.0.1",
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
        docker_gateway_resolver=lambda: "127.0.0.1",
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
        docker_gateway_resolver=lambda: "127.0.0.1",
    )

    with pytest.raises(ProxyConfigurationError) as raised:
        await runtime.prepare_policy()

    assert str(raised.value) == (
        "HTTPS_PROXY and https_proxy disagree; make them identical or unset one"
    )
    await runtime.close()


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
