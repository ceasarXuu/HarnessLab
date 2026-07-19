from __future__ import annotations

import json
import os
import platform
import subprocess
from collections.abc import Mapping
from dataclasses import dataclass


class DockerTargetDiscoveryError(RuntimeError):
    pass


@dataclass(frozen=True)
class DockerProxyTarget:
    kind: str
    context: str
    endpoint_kind: str
    bind_address: str | None = None

    @property
    def supports_host_relay(self) -> bool:
        return self.kind == "local-rootful-linux" and self.bind_address is not None


def discover_docker_proxy_target(
    environ: Mapping[str, str] | None = None,
) -> DockerProxyTarget:
    source = environ if environ is not None else os.environ
    configured_context = source.get("DOCKER_CONTEXT", "").strip()
    configured_host = source.get("DOCKER_HOST", "").strip()
    if configured_context:
        context = configured_context
        endpoint = _context_endpoint(context)
        target_arguments = ["--context", context]
    elif configured_host:
        context = "DOCKER_HOST"
        endpoint = configured_host
        target_arguments = ["--host", endpoint]
    else:
        context = _docker_output(["context", "show"], "current Docker context")
        endpoint = _context_endpoint(context)
        target_arguments = ["--context", context]
    endpoint_kind = _endpoint_kind(endpoint)
    info = _docker_json(
        [*target_arguments, "info", "--format", "{{json .}}"],
        "Docker daemon information",
    )
    security_options = [str(value).lower() for value in info.get("SecurityOptions", [])]
    operating_system = str(info.get("OperatingSystem", "")).lower()

    if any("rootless" in value for value in security_options):
        return DockerProxyTarget("rootless", context, endpoint_kind)
    if "docker desktop" in operating_system:
        return DockerProxyTarget("docker-desktop", context, endpoint_kind)
    if endpoint_kind != "local-unix":
        return DockerProxyTarget("remote-or-virtualized", context, endpoint_kind)
    if platform.system() != "Linux" or str(info.get("OSType", "")).lower() != "linux":
        return DockerProxyTarget("unsupported-local", context, endpoint_kind)

    gateway = _docker_output(
        [
            *target_arguments,
            "network",
            "inspect",
            "bridge",
            "--format",
            "{{(index .IPAM.Config 0).Gateway}}",
        ],
        "default Docker bridge gateway",
    )
    return DockerProxyTarget("local-rootful-linux", context, endpoint_kind, gateway)


def _context_endpoint(context: str) -> str:
    return _docker_output(
        ["context", "inspect", context, "--format", "{{.Endpoints.docker.Host}}"],
        "Docker context endpoint",
    )


def _docker_output(arguments: list[str], description: str) -> str:
    try:
        result = subprocess.run(
            ["docker", *arguments],
            check=True,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise DockerTargetDiscoveryError(f"Could not read {description}") from error
    value = result.stdout.strip()
    if not value:
        raise DockerTargetDiscoveryError(f"Docker returned an empty {description}")
    return value


def _docker_json(arguments: list[str], description: str) -> dict:
    raw = _docker_output(arguments, description)
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise DockerTargetDiscoveryError(f"Docker returned invalid {description}") from error
    if not isinstance(value, dict):
        raise DockerTargetDiscoveryError(f"Docker returned invalid {description}")
    return value


def _endpoint_kind(endpoint: str) -> str:
    normalized = endpoint.lower()
    if normalized.startswith("unix://"):
        return "local-unix"
    if normalized.startswith("npipe://"):
        return "local-named-pipe"
    if normalized.startswith("ssh://"):
        return "remote-ssh"
    if normalized.startswith(("tcp://", "http://", "https://")):
        return "remote-tcp"
    return "unknown"
