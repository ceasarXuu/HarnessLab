from __future__ import annotations

import asyncio
import json
import tempfile
from pathlib import Path
from typing import Any

from harbor.environments.docker.docker import (
    DockerEnvironment,
    _sanitize_docker_compose_project_name,
)

ORNNLAB_MANAGED_LABEL = "ornnlab.managed"
ORNNLAB_INSTANCE_LABEL = "ornnlab.instance_id"
ORNNLAB_RUN_LABEL = "ornnlab.run_id"
ORNNLAB_CLEANUP_LABEL = "ornnlab.cleanup"


class OwnedDockerEnvironment(DockerEnvironment):
    """Harbor Docker environment with OrnnLab ownership on every Compose service."""

    def __init__(
        self,
        *args,
        ornnlab_instance_id: str,
        ornnlab_run_id: str,
        ornnlab_cleanup_policy: str = "auto",
        **kwargs,
    ):
        if not ornnlab_instance_id or not ornnlab_run_id:
            raise ValueError("OrnnLab Docker ownership identifiers cannot be empty")
        if ornnlab_cleanup_policy not in {"auto", "retain"}:
            raise ValueError("OrnnLab Docker cleanup policy must be auto or retain")
        self._ornnlab_labels = {
            ORNNLAB_MANAGED_LABEL: "true",
            ORNNLAB_INSTANCE_LABEL: ornnlab_instance_id,
            ORNNLAB_RUN_LABEL: ornnlab_run_id,
            ORNNLAB_CLEANUP_LABEL: ornnlab_cleanup_policy,
        }
        self._ownership_compose_temp_dir: tempfile.TemporaryDirectory[str] | None = None
        self._ownership_compose_path: Path | None = None
        super().__init__(*args, **kwargs)

    @property
    def _docker_compose_paths(self) -> list[Path]:
        paths = list(super()._docker_compose_paths)
        if self._ownership_compose_path is not None:
            paths.append(self._ownership_compose_path)
        return paths

    async def _run_docker_compose_command(self, command, check=True, timeout_sec=None):
        if self._ownership_compose_path is None:
            await self._prepare_ownership_compose_file()
        return await super()._run_docker_compose_command(command, check, timeout_sec)

    async def stop(self, delete: bool):
        try:
            await super().stop(delete)
        finally:
            self._cleanup_ownership_compose_file()

    async def _prepare_ownership_compose_file(self) -> None:
        services, networks, volumes = await self._discover_compose_resources()
        if not services:
            raise RuntimeError("Harbor Docker Compose config contains no services")
        self._ownership_compose_temp_dir = tempfile.TemporaryDirectory(
            prefix="ornnlab-compose-ownership-"
        )
        path = Path(self._ownership_compose_temp_dir.name) / "ownership.json"
        path.write_text(
            json.dumps(
                _ownership_compose_payload(
                    services,
                    self._ornnlab_labels,
                    networks=networks,
                    volumes=volumes,
                )
            ),
            encoding="utf-8",
        )
        self._ownership_compose_path = path
        self.logger.info(
            "docker.ownership.compose_prepared run_id=%s session_id=%s service_count=%s "
            "network_count=%s volume_count=%s",
            self._ornnlab_labels[ORNNLAB_RUN_LABEL],
            self.session_id,
            len(services),
            len(networks),
            len(volumes),
        )

    async def _discover_compose_resources(self) -> tuple[list[str], list[str], list[str]]:
        command = [
            "docker",
            "compose",
            "--project-name",
            _sanitize_docker_compose_project_name(self.session_id),
            "--project-directory",
            str(self.environment_dir.resolve().absolute()),
        ]
        for path in super()._docker_compose_paths:
            command.extend(["-f", str(path.resolve().absolute())])
        command.extend(["config", "--format", "json"])
        process = await asyncio.create_subprocess_exec(
            *command,
            env=self._compose_env_vars(include_os_env=True),
            stdin=asyncio.subprocess.DEVNULL,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )
        output, _ = await process.communicate()
        text = output.decode(errors="replace") if output else ""
        if process.returncode != 0:
            raise RuntimeError(
                "Failed to discover Harbor Docker Compose services before ownership "
                f"labelling: {text.strip()}"
            )
        try:
            model = json.loads(text)
        except json.JSONDecodeError as error:
            raise RuntimeError("Harbor Docker Compose config did not return valid JSON") from error
        return (
            _mapping_keys(model.get("services")),
            _mapping_keys(model.get("networks")),
            _mapping_keys(model.get("volumes")),
        )

    def _cleanup_ownership_compose_file(self) -> None:
        if self._ownership_compose_temp_dir is None:
            return
        try:
            self._ownership_compose_temp_dir.cleanup()
        finally:
            self._ownership_compose_temp_dir = None
            self._ownership_compose_path = None


def _ownership_compose_payload(
    services: list[str],
    labels: dict[str, str],
    *,
    networks: list[str] | None = None,
    volumes: list[str] | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "services": {service: {"labels": labels} for service in services}
    }
    if networks:
        payload["networks"] = {network: {"labels": labels} for network in networks}
    if volumes:
        payload["volumes"] = {volume: {"labels": labels} for volume in volumes}
    return payload


def _mapping_keys(value: Any) -> list[str]:
    return list(value) if isinstance(value, dict) else []
