from __future__ import annotations

import json
import os
import shlex
import shutil
import subprocess
from typing import Any

HARNESSLAB_RUN_LABEL = "harnesslab.run_id"


class DockerOrphanService:
    def __init__(self, command: list[str] | None = None, timeout_sec: float = 5.0):
        self.command = command or _command_from_env()
        self.timeout_sec = timeout_sec

    def scan_harnesslab_containers(self) -> dict[str, Any]:
        executable = _resolve_executable(self.command[0])
        if executable is None:
            return {
                "available": False,
                "ok": False,
                "command": self.command,
                "label": HARNESSLAB_RUN_LABEL,
                "count": 0,
                "containers": [],
                "cleanup_plan": [],
                "error": "docker_cli_missing",
            }

        try:
            result = subprocess.run(
                [
                    *self.command,
                    "ps",
                    "-a",
                    "--filter",
                    f"label={HARNESSLAB_RUN_LABEL}",
                    "--format",
                    "{{json .}}",
                ],
                check=False,
                capture_output=True,
                text=True,
                timeout=self.timeout_sec,
            )
        except subprocess.TimeoutExpired:
            return self._failed_scan("docker_ps_timeout")
        if result.returncode != 0:
            return self._failed_scan(
                result.stderr.strip() or result.stdout.strip() or "docker_ps_failed"
            )

        containers: list[dict[str, Any]] = []
        try:
            for line in result.stdout.splitlines():
                if line.strip():
                    containers.append(_container_from_json(line))
        except json.JSONDecodeError as error:
            return self._failed_scan(f"docker_ps_parse_failed: {error.msg}")
        cleanup_plan = [
            {
                "container_id": container["id"],
                "name": container["name"],
                "run_id": container["labels"].get(HARNESSLAB_RUN_LABEL),
                "command": [*self.command, "rm", "-f", container["id"]],
                "dry_run": True,
                "manual_review_required": True,
            }
            for container in containers
        ]
        return {
            "available": True,
            "ok": True,
            "command": self.command,
            "label": HARNESSLAB_RUN_LABEL,
            "count": len(containers),
            "containers": containers,
            "cleanup_plan": cleanup_plan,
            "error": None,
        }

    def _failed_scan(self, error: str) -> dict[str, Any]:
        return {
            "available": True,
            "ok": False,
            "command": self.command,
            "label": HARNESSLAB_RUN_LABEL,
            "count": 0,
            "containers": [],
            "cleanup_plan": [],
            "error": error,
        }


def _command_from_env() -> list[str]:
    raw = os.environ.get("HARNESSLAB_DOCKER_COMMAND", "docker")
    command = shlex.split(raw)
    if not command:
        raise ValueError("HARNESSLAB_DOCKER_COMMAND cannot be empty")
    return command


def _resolve_executable(executable: str) -> str | None:
    if os.path.isabs(executable):
        return executable if os.path.exists(executable) else None
    return shutil.which(executable)


def _container_from_json(line: str) -> dict[str, Any]:
    payload = json.loads(line)
    labels = _parse_labels(str(payload.get("Labels", "")))
    return {
        "id": str(payload.get("ID", "")),
        "name": str(payload.get("Names", "")),
        "image": str(payload.get("Image", "")),
        "status": str(payload.get("Status", "")),
        "labels": labels,
    }


def _parse_labels(value: str) -> dict[str, str]:
    labels: dict[str, str] = {}
    for entry in value.split(","):
        if not entry:
            continue
        key, separator, label_value = entry.partition("=")
        if separator:
            labels[key] = label_value
        else:
            labels[key] = ""
    return labels
