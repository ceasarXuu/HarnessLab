from __future__ import annotations

import asyncio
import contextlib
import ipaddress
import logging
import os
from collections.abc import Awaitable, Callable, Collection, Mapping
from dataclasses import dataclass, field
from urllib.parse import SplitResult, urlsplit, urlunsplit

from ornnlab.services.docker_proxy_target import (
    DockerProxyTarget,
    DockerTargetDiscoveryError,
    discover_docker_proxy_target,
)

logger = logging.getLogger("uvicorn.error")

_PROXY_GROUPS = (
    ("HTTP_PROXY", "http_proxy", "ORNNLAB_CONTAINER_HTTP_PROXY"),
    ("HTTPS_PROXY", "https_proxy", "ORNNLAB_CONTAINER_HTTPS_PROXY"),
    ("ALL_PROXY", "all_proxy", "ORNNLAB_CONTAINER_ALL_PROXY"),
)
_NO_PROXY_GROUP = ("NO_PROXY", "no_proxy", "ORNNLAB_CONTAINER_NO_PROXY")
_SUPPORTED_SCHEMES = {"http", "https", "socks4", "socks4a", "socks5", "socks5h"}
_DEFAULT_PORTS = {
    "http": 80,
    "https": 443,
    "socks4": 1080,
    "socks4a": 1080,
    "socks5": 1080,
    "socks5h": 1080,
}


class ProxyConfigurationError(RuntimeError):
    """A host proxy was found but cannot be exposed to Docker safely."""


@dataclass
class RuntimeProxyPolicy:
    agent_env_defaults: dict[str, str]
    subprocess_env: dict[str, str]
    relay_count: int
    strategy: str = "none"
    target_kind: str | None = None
    _release: Callable[[], Awaitable[None]] | None = field(default=None, repr=False)
    _closed: bool = field(default=False, init=False, repr=False)

    async def close(self) -> None:
        if self._closed:
            return
        if self._release is not None:
            await self._release()
        self._closed = True


@dataclass(frozen=True)
class _ProxyEndpoint:
    scheme: str
    host: str
    port: int
    parsed: SplitResult


class ContainerProxyRuntime:
    """Derive container proxy policy from the effective Docker target capabilities."""

    def __init__(
        self,
        *,
        environ: Mapping[str, str] | None = None,
        mode: str | None = None,
        docker_target_resolver: Callable[[], DockerProxyTarget] | None = None,
    ) -> None:
        self._environ = environ if environ is not None else os.environ
        self._mode = (mode or self._environ.get("ORNNLAB_DOCKER_PROXY_MODE", "auto")).lower()
        if self._mode not in {"auto", "off"}:
            raise ValueError("ORNNLAB_DOCKER_PROXY_MODE must be auto or off")
        self._docker_target_resolver = docker_target_resolver or (
            lambda: discover_docker_proxy_target(self._environ)
        )
        self._lock = asyncio.Lock()
        self._active_servers: set[asyncio.Server] = set()
        self._active_connection_tasks: set[asyncio.Task[None]] = set()

    async def start(self) -> None:
        """Lifecycle hook; Docker discovery remains lazy so Docker may start later."""

        proxy_names = _configured_proxy_names(self._environ)
        logger.info(
            "docker_proxy_detection mode=%s variables=%s",
            self._mode,
            ",".join(proxy_names),
            extra={
                "event": "docker_proxy_detection",
                "mode": self._mode,
                "variable_names": proxy_names,
            },
        )

    async def prepare_policy(
        self,
        explicit_proxy_names: Collection[str] = (),
        *,
        automatic_proxy_allowed: bool = True,
    ) -> RuntimeProxyPolicy:
        if self._mode == "off" or not automatic_proxy_allowed:
            if not automatic_proxy_allowed:
                logger.info(
                    "docker_proxy_policy_skipped reason=network_allowlist",
                    extra={
                        "event": "docker_proxy_policy_skipped",
                        "reason": "network_allowlist",
                    },
                )
            return RuntimeProxyPolicy({}, {}, 0)

        async with self._lock:
            explicit_names = set(explicit_proxy_names)
            agent_defaults: dict[str, str] = {}
            subprocess_env: dict[str, str] = {}
            relays: dict[tuple[str, str, int], tuple[asyncio.Server, str]] = {}
            connection_tasks: set[asyncio.Task[None]] = set()
            target: DockerProxyTarget | None = None
            try:
                for upper, lower, runtime_name in _PROXY_GROUPS:
                    if explicit_names.intersection((upper, lower)):
                        continue
                    source_name, value = _proxy_group_value(self._environ, upper, lower)
                    if value is None:
                        continue
                    assert source_name is not None
                    endpoint = _parse_proxy_endpoint(value, source_name)
                    runtime_value = value
                    if _is_loopback(endpoint.host):
                        target = target or await self._resolve_target()
                        self._require_host_relay(target)
                        runtime_value = await self._loopback_relay_url(
                            endpoint, source_name, target, relays, connection_tasks
                        )
                    agent_defaults[upper] = f"${{{runtime_name}}}"
                    agent_defaults[lower] = f"${{{runtime_name}}}"
                    subprocess_env[runtime_name] = runtime_value

                no_proxy_source = None
                if not explicit_names.intersection(_NO_PROXY_GROUP[:2]):
                    no_proxy_source, no_proxy = _proxy_group_value(
                        self._environ, _NO_PROXY_GROUP[0], _NO_PROXY_GROUP[1]
                    )
                    if no_proxy is not None:
                        runtime_name = _NO_PROXY_GROUP[2]
                        agent_defaults["NO_PROXY"] = f"${{{runtime_name}}}"
                        agent_defaults["no_proxy"] = f"${{{runtime_name}}}"
                        subprocess_env[runtime_name] = no_proxy
            except asyncio.CancelledError:
                await self._release_policy(
                    tuple(server for server, _ in relays.values()), connection_tasks
                )
                raise
            except Exception:
                await self._release_policy(
                    tuple(server for server, _ in relays.values()), connection_tasks
                )
                raise

            servers = tuple(server for server, _ in relays.values())
            strategy = "host-relay" if servers else ("direct" if agent_defaults else "none")

            logger.info(
                "docker_proxy_policy_prepared variables=%s relay_count=%s",
                ",".join(sorted(agent_defaults)),
                len(servers),
                extra={
                    "event": "docker_proxy_policy_prepared",
                    "variable_names": sorted(agent_defaults),
                    "relay_count": len(servers),
                    "no_proxy_source": no_proxy_source,
                    "strategy": strategy,
                    "target_kind": target.kind if target else None,
                },
            )
            return RuntimeProxyPolicy(
                agent_defaults,
                subprocess_env,
                len(servers),
                strategy,
                target.kind if target else None,
                lambda: self._release_policy(servers, connection_tasks),
            )

    async def close(self) -> None:
        async with self._lock:
            servers = tuple(self._active_servers)
            connection_tasks = set(self._active_connection_tasks)
        await self._release_policy(servers, connection_tasks)
        logger.info(
            "docker_proxy_runtime_stopped relay_count=%s",
            len(servers),
            extra={"event": "docker_proxy_runtime_stopped", "relay_count": len(servers)},
        )

    async def _resolve_target(self) -> DockerProxyTarget:
        try:
            target = await asyncio.to_thread(self._docker_target_resolver)
        except DockerTargetDiscoveryError as error:
            raise ProxyConfigurationError(
                "Host proxy is loopback-only, but the effective Docker target could not "
                "be identified"
            ) from error
        logger.info(
            "docker_proxy_target_classified kind=%s endpoint_kind=%s context=%s",
            target.kind,
            target.endpoint_kind,
            target.context,
            extra={
                "event": "docker_proxy_target_classified",
                "target_kind": target.kind,
                "endpoint_kind": target.endpoint_kind,
                "docker_context": target.context,
            },
        )
        return target

    @staticmethod
    def _require_host_relay(target: DockerProxyTarget) -> None:
        if not target.supports_host_relay:
            raise ProxyConfigurationError(
                "Host proxy is loopback-only, but Docker target "
                f"{target.kind!r} cannot safely reach a host relay; configure a "
                "container-reachable proxy in the Agent/Environment profile or disable "
                "automatic inheritance"
            )

    async def _loopback_relay_url(
        self,
        endpoint: _ProxyEndpoint,
        variable_name: str,
        target: DockerProxyTarget,
        relays: dict[tuple[str, str, int], tuple[asyncio.Server, str]],
        connection_tasks: set[asyncio.Task[None]],
    ) -> str:
        if endpoint.parsed.username is not None or endpoint.parsed.password is not None:
            raise ProxyConfigurationError(
                f"{variable_name} uses credentials on a loopback proxy; automatic Docker "
                "relay is disabled to keep credentials out of Harbor artifacts"
            )
        if endpoint.scheme == "https":
            raise ProxyConfigurationError(
                f"{variable_name} uses an HTTPS loopback proxy; automatic relay cannot "
                "preserve proxy TLS hostname verification"
            )
        key = (endpoint.scheme, endpoint.host, endpoint.port)
        existing = relays.get(key)
        if existing is None:
            await self._ensure_upstream_available(endpoint, variable_name)
            gateway = target.bind_address
            assert gateway is not None
            try:
                gateway_ip = ipaddress.ip_address(gateway)
            except ValueError as error:
                raise ProxyConfigurationError(
                    "Docker target returned an invalid host gateway address"
                ) from error
            try:
                server = await asyncio.start_server(
                    lambda reader, writer: self._start_relay_connection(
                        reader, writer, endpoint, connection_tasks
                    ),
                    gateway,
                    0,
                )
            except OSError as error:
                raise ProxyConfigurationError(
                    "Docker target was classified as local, but its host gateway is not "
                    "bindable by OrnnLab"
                ) from error
            socket = server.sockets[0] if server.sockets else None
            if socket is None:
                server.close()
                await server.wait_closed()
                raise ProxyConfigurationError("Docker proxy relay started without a socket")
            relay_port = int(socket.getsockname()[1])
            relay_host = f"[{gateway}]" if gateway_ip.version == 6 else gateway
            relay_url = urlunsplit((endpoint.scheme, f"{relay_host}:{relay_port}", "", "", ""))
            relays[key] = (server, relay_url)
            self._active_servers.add(server)
            logger.info(
                "docker_proxy_bridge_started bind=%s:%s scheme=%s",
                gateway,
                relay_port,
                endpoint.scheme,
                extra={
                    "event": "docker_proxy_bridge_started",
                    "bind_address": gateway,
                    "bind_port": relay_port,
                    "proxy_scheme": endpoint.scheme,
                },
            )
            return relay_url
        return existing[1]

    @staticmethod
    async def _ensure_upstream_available(
        endpoint: _ProxyEndpoint, variable_name: str
    ) -> None:
        try:
            _, writer = await asyncio.wait_for(
                asyncio.open_connection(endpoint.host, endpoint.port), timeout=2
            )
        except (OSError, TimeoutError) as error:
            raise ProxyConfigurationError(
                f"{variable_name} loopback proxy is not accepting connections"
            ) from error
        writer.close()
        with contextlib.suppress(OSError):
            await writer.wait_closed()

    def _start_relay_connection(
        self,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter,
        endpoint: _ProxyEndpoint,
        policy_tasks: set[asyncio.Task[None]],
    ) -> None:
        task = asyncio.create_task(self._relay_connection(reader, writer, endpoint))
        policy_tasks.add(task)
        self._active_connection_tasks.add(task)
        task.add_done_callback(policy_tasks.discard)
        task.add_done_callback(self._active_connection_tasks.discard)

    async def _release_policy(
        self,
        servers: Collection[asyncio.Server],
        connection_tasks: Collection[asyncio.Task[None]],
    ) -> None:
        unique_servers = tuple(set(servers))
        for server in unique_servers:
            server.close()
        tasks = tuple(set(connection_tasks))
        for task in tasks:
            task.cancel()
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
        if unique_servers:
            await asyncio.gather(*(server.wait_closed() for server in unique_servers))
        self._active_servers.difference_update(unique_servers)
        self._active_connection_tasks.difference_update(tasks)
        if unique_servers:
            logger.info(
                "docker_proxy_policy_released relay_count=%s connection_count=%s",
                len(unique_servers),
                len(tasks),
                extra={
                    "event": "docker_proxy_policy_released",
                    "relay_count": len(unique_servers),
                    "connection_count": len(tasks),
                },
            )

    async def _relay_connection(
        self,
        reader: asyncio.StreamReader,
        writer: asyncio.StreamWriter,
        endpoint: _ProxyEndpoint,
    ) -> None:
        try:
            upstream_reader, upstream_writer = await asyncio.open_connection(
                endpoint.host, endpoint.port
            )
        except asyncio.CancelledError:
            writer.close()
            await writer.wait_closed()
            raise
        except OSError as error:
            logger.warning(
                "docker_proxy_upstream_failed scheme=%s error_type=%s",
                endpoint.scheme,
                type(error).__name__,
                extra={
                    "event": "docker_proxy_upstream_failed",
                    "proxy_scheme": endpoint.scheme,
                    "error_type": type(error).__name__,
                },
            )
            writer.close()
            await writer.wait_closed()
            return

        async def copy(source: asyncio.StreamReader, target: asyncio.StreamWriter) -> None:
            try:
                while data := await source.read(64 * 1024):
                    target.write(data)
                    await target.drain()
            except (ConnectionError, asyncio.CancelledError):
                pass
            finally:
                target.close()

        await asyncio.gather(
            copy(reader, upstream_writer),
            copy(upstream_reader, writer),
            return_exceptions=True,
        )


def _configured_proxy_names(environ: Mapping[str, str]) -> list[str]:
    names = {name for group in _PROXY_GROUPS for name in group[:2]}
    names.update(_NO_PROXY_GROUP[:2])
    return sorted(name for name in names if environ.get(name))


def _proxy_group_value(
    environ: Mapping[str, str], upper: str, lower: str
) -> tuple[str | None, str | None]:
    upper_value = environ.get(upper, "").strip()
    lower_value = environ.get(lower, "").strip()
    if upper_value and lower_value and upper_value != lower_value:
        raise ProxyConfigurationError(
            f"{upper} and {lower} disagree; make them identical or unset one"
        )
    if lower_value:
        return lower, lower_value
    if upper_value:
        return upper, upper_value
    return None, None


def _parse_proxy_endpoint(value: str, variable_name: str | None) -> _ProxyEndpoint:
    parsed = urlsplit(value)
    if not parsed.scheme or not parsed.hostname:
        raise ProxyConfigurationError(
            f"{variable_name or 'proxy'} must be an absolute proxy URL with a scheme"
        )
    scheme = parsed.scheme.lower()
    if scheme not in _SUPPORTED_SCHEMES:
        raise ProxyConfigurationError(
            f"{variable_name or 'proxy'} uses unsupported proxy scheme {scheme!r}"
        )
    try:
        port = parsed.port or _DEFAULT_PORTS[scheme]
    except ValueError as error:
        raise ProxyConfigurationError(
            f"{variable_name or 'proxy'} contains an invalid port"
        ) from error
    host = parsed.hostname
    try:
        if ipaddress.ip_address(host).is_unspecified:
            raise ProxyConfigurationError(
                f"{variable_name or 'proxy'} cannot target an unspecified address"
            )
    except ValueError:
        pass
    return _ProxyEndpoint(scheme, host, port, parsed)


def _is_loopback(host: str) -> bool:
    if host.lower().rstrip(".") == "localhost":
        return True
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return False
