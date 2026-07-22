from __future__ import annotations

import json
import logging
import os
import shutil
import subprocess
from collections.abc import Iterable
from typing import Any

from ornnlab.services.command_line import split_command
from ornnlab.services.owned_docker_environment import (
    ORNNLAB_CLEANUP_LABEL,
    ORNNLAB_INSTANCE_LABEL,
    ORNNLAB_MANAGED_LABEL,
    ORNNLAB_RUN_LABEL,
)

COMPOSE_PROJECT_LABEL = "com.docker.compose.project"
logger = logging.getLogger(__name__)


class DockerOrphanService:
    def __init__(
        self,
        command: list[str] | None = None,
        timeout_sec: float = 5.0,
        instance_id: str | None = None,
    ):
        if command is None:
            self.command, self.env_warnings = _command_from_env()
        else:
            self.command = command
            self.env_warnings = []
        self.timeout_sec = timeout_sec
        self.instance_id = instance_id

    def scan_ornnlab_containers(
        self,
        active_run_ids: Iterable[str] = (),
    ) -> dict[str, Any]:
        unavailable = self._unavailable_result()
        if unavailable is not None:
            return unavailable
        filters = [f"{ORNNLAB_MANAGED_LABEL}=true"]
        if self.instance_id:
            filters.append(f"{ORNNLAB_INSTANCE_LABEL}={self.instance_id}")
        scan = self._scan_labels(filters)
        if scan["error"]:
            return self._failed_scan(scan["error"])
        active = set(active_run_ids)
        owned = scan["containers"]
        containers = [
            container
            for container in owned
            if container["labels"].get(ORNNLAB_RUN_LABEL) not in active
            and container["labels"].get(ORNNLAB_CLEANUP_LABEL, "auto") != "retain"
        ]
        cleanup_plan = [self._cleanup_plan_item(container) for container in containers]
        return {
            "available": True,
            "ok": True,
            "command": self.command,
            "label": ORNNLAB_RUN_LABEL,
            "instance_id": self.instance_id,
            "owned_count": len(owned),
            "count": len(containers),
            "containers": containers,
            "cleanup_plan": cleanup_plan,
            "warnings": self.env_warnings,
            "error": None,
        }

    def cleanup_run(self, run_id: str) -> dict[str, Any]:
        if not run_id:
            raise ValueError("run_id cannot be empty")
        if not self.instance_id:
            return _cleanup_failure(run_id, None, "docker_cleanup_instance_id_required")
        unavailable = self._unavailable_result()
        if unavailable is not None:
            return _cleanup_failure(run_id, self.instance_id, unavailable["error"])
        filters = [f"{ORNNLAB_MANAGED_LABEL}=true", f"{ORNNLAB_RUN_LABEL}={run_id}"]
        if self.instance_id:
            filters.append(f"{ORNNLAB_INSTANCE_LABEL}={self.instance_id}")
        scan = self._scan_labels(filters)
        if scan["error"]:
            return _cleanup_failure(run_id, self.instance_id, scan["error"])
        containers = [
            item
            for item in scan["containers"]
            if item["labels"].get(ORNNLAB_CLEANUP_LABEL, "auto") != "retain"
        ]
        projects = sorted(
            {
                project
                for item in containers
                if (project := item["labels"].get(COMPOSE_PROJECT_LABEL))
            }
        )
        errors: list[str] = []
        removed_containers = self._remove_resources(
            "container", [item["id"] for item in containers], errors
        )
        owned_filters = [*filters, f"{ORNNLAB_CLEANUP_LABEL}=auto"]
        network_ids = self._list_labelled_resource_ids("network", owned_filters, errors)
        volume_ids = self._list_labelled_resource_ids("volume", owned_filters, errors)
        removed_networks = self._remove_resources("network", network_ids, errors)
        removed_volumes = self._remove_resources("volume", volume_ids, errors)
        if not network_ids:
            for project in projects:
                removed_networks += self._remove_labelled_resources(
                    "network", COMPOSE_PROJECT_LABEL, project, errors
                )
        if not volume_ids:
            for project in projects:
                removed_volumes += self._remove_labelled_resources(
                    "volume", COMPOSE_PROJECT_LABEL, project, errors
                )
        result = {
            "ok": not errors,
            "run_id": run_id,
            "instance_id": self.instance_id,
            "matched_containers": len(containers),
            "removed_containers": removed_containers,
            "removed_networks": removed_networks,
            "removed_volumes": removed_volumes,
            "projects": projects,
            "errors": errors,
        }
        log = logger.info if result["ok"] else logger.warning
        log(
            "docker.ownership.cleanup run_id=%s instance_id=%s matched=%s "
            "containers=%s networks=%s volumes=%s errors=%s",
            run_id,
            self.instance_id,
            len(containers),
            removed_containers,
            removed_networks,
            removed_volumes,
            len(errors),
        )
        return result

    def cleanup_orphans(self, active_run_ids: Iterable[str] = ()) -> dict[str, Any]:
        scan = self.scan_ornnlab_containers(active_run_ids)
        if not scan["ok"]:
            return {
                "ok": False,
                "run_count": 0,
                "results": [],
                "error": scan["error"],
            }
        run_ids = sorted(
            {
                run_id
                for item in scan["containers"]
                if (run_id := item["labels"].get(ORNNLAB_RUN_LABEL))
            }
        )
        results = [self.cleanup_run(run_id) for run_id in run_ids]
        return {
            "ok": all(result["ok"] for result in results),
            "run_count": len(run_ids),
            "results": results,
            "error": None,
        }

    def _scan_labels(self, labels: list[str]) -> dict[str, Any]:
        arguments = [*self.command, "ps", "-a"]
        for label in labels:
            arguments.extend(["--filter", f"label={label}"])
        arguments.extend(["--format", "{{json .}}"])
        result = self._run(arguments)
        if result["error"]:
            error = result["error"]
            if error == "docker_command_timeout":
                error = "docker_ps_timeout"
            return {"containers": [], "error": error}
        containers: list[dict[str, Any]] = []
        try:
            for line in (result["stdout"] or "").splitlines():
                if line.strip():
                    containers.append(_container_from_json(line))
        except json.JSONDecodeError as error:
            return {"containers": [], "error": f"docker_ps_parse_failed: {error.msg}"}
        return {"containers": containers, "error": None}

    def _remove_labelled_resources(
        self,
        resource: str,
        label: str,
        value: str,
        errors: list[str],
    ) -> int:
        listing = self._run(
            [
                *self.command,
                resource,
                "ls",
                "--filter",
                f"label={label}={value}",
                "--format",
                "{{.ID}}",
            ]
        )
        if listing["error"]:
            errors.append(f"{resource}_list_failed: {listing['error']}")
            return 0
        identifiers = [
            line.strip() for line in (listing["stdout"] or "").splitlines() if line.strip()
        ]
        return self._remove_resources(resource, identifiers, errors)

    def _list_labelled_resource_ids(
        self,
        resource: str,
        labels: list[str],
        errors: list[str],
    ) -> list[str]:
        arguments = [*self.command, resource, "ls"]
        for label in labels:
            arguments.extend(["--filter", f"label={label}"])
        arguments.extend(["--format", "{{.ID}}"])
        result = self._run(arguments)
        if result["error"]:
            errors.append(f"{resource}_list_failed: {result['error']}")
            return []
        return [
            line.strip() for line in (result["stdout"] or "").splitlines() if line.strip()
        ]

    def _remove_resources(
        self,
        resource: str,
        identifiers: list[str],
        errors: list[str],
    ) -> int:
        if not identifiers:
            return 0
        arguments = [*self.command]
        if resource == "container":
            arguments.extend(["rm", "-f"])
        else:
            arguments.extend([resource, "rm"])
        arguments.extend(identifiers)
        result = self._run(arguments)
        if result["error"]:
            errors.append(f"{resource}_remove_failed: {result['error']}")
            return 0
        return len(identifiers)

    def _run(self, arguments: list[str]) -> dict[str, str | None]:
        try:
            result = subprocess.run(
                arguments,
                check=False,
                capture_output=True,
                text=True,
                timeout=self.timeout_sec,
            )
        except subprocess.TimeoutExpired:
            return {"stdout": "", "error": "docker_command_timeout"}
        if result.returncode != 0:
            return {
                "stdout": result.stdout,
                "error": result.stderr.strip() or result.stdout.strip() or "docker_command_failed",
            }
        return {"stdout": result.stdout, "error": None}

    def _cleanup_plan_item(self, container: dict[str, Any]) -> dict[str, Any]:
        return {
            "container_id": container["id"],
            "name": container["name"],
            "run_id": container["labels"].get(ORNNLAB_RUN_LABEL),
            "instance_id": container["labels"].get(ORNNLAB_INSTANCE_LABEL),
            "command": [*self.command, "rm", "-f", container["id"]],
            "dry_run": True,
            "manual_review_required": False,
        }

    def _unavailable_result(self) -> dict[str, Any] | None:
        if _resolve_executable(self.command[0]) is not None:
            return None
        return {
            "available": False,
            "ok": False,
            "command": self.command,
            "label": ORNNLAB_RUN_LABEL,
            "instance_id": self.instance_id,
            "owned_count": 0,
            "count": 0,
            "containers": [],
            "cleanup_plan": [],
            "warnings": self.env_warnings,
            "error": "docker_cli_missing",
        }

    def _failed_scan(self, error: str) -> dict[str, Any]:
        return {
            "available": True,
            "ok": False,
            "command": self.command,
            "label": ORNNLAB_RUN_LABEL,
            "instance_id": self.instance_id,
            "owned_count": 0,
            "count": 0,
            "containers": [],
            "cleanup_plan": [],
            "warnings": self.env_warnings,
            "error": error,
        }


def _cleanup_failure(
    run_id: str,
    instance_id: str | None,
    error: str | None,
) -> dict[str, Any]:
    return {
        "ok": False,
        "run_id": run_id,
        "instance_id": instance_id,
        "matched_containers": 0,
        "removed_containers": 0,
        "removed_networks": 0,
        "removed_volumes": 0,
        "projects": [],
        "errors": [error or "docker_cleanup_failed"],
    }


def _command_from_env() -> tuple[list[str], list[str]]:
    raw = os.environ.get("ORNNLAB_DOCKER_COMMAND", "docker")
    try:
        command = split_command(raw)
    except ValueError as error:
        if str(error) != "command cannot be empty":
            raise
        raise ValueError("ORNNLAB_DOCKER_COMMAND cannot be empty") from None
    return command, []


def _resolve_executable(executable: str) -> str | None:
    if os.path.isabs(executable):
        return executable if os.path.exists(executable) else None
    return shutil.which(executable)


def _container_from_json(line: str) -> dict[str, Any]:
    payload = json.loads(line)
    return {
        "id": str(payload.get("ID", "")),
        "name": str(payload.get("Names", "")),
        "image": str(payload.get("Image", "")),
        "status": str(payload.get("Status", "")),
        "labels": _parse_labels(str(payload.get("Labels", ""))),
    }


def _parse_labels(value: str) -> dict[str, str]:
    labels: dict[str, str] = {}
    for entry in value.split(","):
        if not entry:
            continue
        key, separator, label_value = entry.partition("=")
        labels[key] = label_value if separator else ""
    return labels
