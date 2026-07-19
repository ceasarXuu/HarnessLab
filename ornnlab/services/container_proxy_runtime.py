from __future__ import annotations

import asyncio
import ipaddress
import logging
import os
import subprocess
from collections.abc import Callable, Mapping
from dataclasses import dataclass
from urllib.parse import SplitResult, urlsplit, urlunsplit

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


@dataclass(frozen=True)
class RuntimeProxyPolicy:
    agent_env_defaults: dict[str, str]
    subprocess_env: dict[str, str]
    relay_count: int


@dataclass(frozen=True)
class _ProxyEndpoint:
    scheme: str
    host: str
    port: int
    parsed: SplitResult


class ContainerProxyRuntime:
    """Discover host proxy settings and expose loopback proxies to local Docker."""

    def __init__(
        self,
        *,
        environ: Mapping[str, str] | None = None,
        mode: str | None = None,
        docker_gateway_resolver: Callable[[], str] | None = None,
    ) -> None:
        self._environ = environ if environ is not None else os.environ
        self._mode = (mode or self._environ.get("ORNNLAB_DOCKER_PROXY_MODE", "auto")).lower()
        if self._mode not in {"auto", "off"}:
            raise ValueError("ORNNLAB_DOCKER_PROXY_MODE must be auto or off")
        self._docker_gateway_resolver = docker_gateway_resolver or _docker_bridge_gateway
        self._lock = asyncio.Lock()
        self._relays: dict[tuple[str, int], tuple[asyncio.Server, str]] = {}

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

    async def prepare_policy(self) -> RuntimeProxyPolicy:
        if self._mode == "off":
            return RuntimeProxyPolicy({}, {}, 0)

        async with self._lock:
            agent_defaults: dict[str, str] = {}
            subprocess_env: dict[str, str] = {}
            for upper, lower, runtime_name in _PROXY_GROUPS:
                source_name, value = _proxy_group_value(self._environ, upper, lower)
                if value is None:
                    continue
                assert source_name is not None
                endpoint = _parse_proxy_endpoint(value, source_name)
                runtime_value = value
                if _is_loopback(endpoint.host):
                    runtime_value = await self._loopback_relay_url(endpoint, source_name)
                agent_defaults[upper] = f"${{{runtime_name}}}"
                agent_defaults[lower] = f"${{{runtime_name}}}"
                subprocess_env[runtime_name] = runtime_value

            no_proxy_source, no_proxy = _proxy_group_value(
                self._environ, _NO_PROXY_GROUP[0], _NO_PROXY_GROUP[1]
            )
            if no_proxy is not None:
                runtime_name = _NO_PROXY_GROUP[2]
                agent_defaults["NO_PROXY"] = f"${{{runtime_name}}}"
                agent_defaults["no_proxy"] = f"${{{runtime_name}}}"
                subprocess_env[runtime_name] = no_proxy

            logger.info(
                "docker_proxy_policy_prepared variables=%s relay_count=%s",
                ",".join(sorted(agent_defaults)),
                len(self._relays),
                extra={
                    "event": "docker_proxy_policy_prepared",
                    "variable_names": sorted(agent_defaults),
                    "relay_count": len(self._relays),
                    "no_proxy_source": no_proxy_source,
                },
            )
            return RuntimeProxyPolicy(agent_defaults, subprocess_env, len(self._relays))

    async def close(self) -> None:
        async with self._lock:
            servers = [server for server, _ in self._relays.values()]
            self._relays.clear()
        for server in servers:
            server.close()
        if servers:
            await asyncio.gather(*(server.wait_closed() for server in servers))
        logger.info(
            "docker_proxy_runtime_stopped relay_count=%s",
            len(servers),
            extra={"event": "docker_proxy_runtime_stopped", "relay_count": len(servers)},
        )

    async def _loopback_relay_url(self, endpoint: _ProxyEndpoint, variable_name: str) -> str:
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
        key = (endpoint.host, endpoint.port)
        existing = self._relays.get(key)
        if existing is None:
            gateway = await asyncio.to_thread(self._docker_gateway_resolver)
            server = await asyncio.start_server(
                lambda reader, writer: self._relay_connection(reader, writer, endpoint),
                gateway,
                0,
            )
            socket = server.sockets[0] if server.sockets else None
            if socket is None:
                server.close()
                await server.wait_closed()
                raise ProxyConfigurationError("Docker proxy relay started without a socket")
            relay_port = int(socket.getsockname()[1])
            relay_url = urlunsplit((endpoint.scheme, f"{gateway}:{relay_port}", "", "", ""))
            self._relays[key] = (server, relay_url)
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


def _docker_bridge_gateway() -> str:
    try:
        result = subprocess.run(
            [
                "docker",
                "network",
                "inspect",
                "bridge",
                "--format",
                "{{(index .IPAM.Config 0).Gateway}}",
            ],
            check=True,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise ProxyConfigurationError(
            "Host proxy is loopback-only, but the local Docker bridge gateway could not be resolved"
        ) from error
    gateway = result.stdout.strip()
    if not gateway:
        raise ProxyConfigurationError("Local Docker bridge has no gateway address")
    return gateway
