from __future__ import annotations

import http.client
import json
import logging
import os
import platform
import re
import shutil
import subprocess
from pathlib import Path
from urllib.parse import urlparse

import psutil

logger = logging.getLogger(__name__)
_probe_signatures: dict[str, tuple[str, str]] = {}


def probe_system_health(home: Path, doctor: dict, harbor_executable: str) -> list[dict]:
    cache_dir = Path("~/.cache/harbor").expanduser()
    return [
        _dev_service_component(),
        _harbor_component(doctor["harbor_version"], harbor_executable),
        _docker_component(doctor["docker"]),
        _cache_component(cache_dir),
        _cpu_component(),
        _gpu_component(),
        _disk_component(_disk_usage(home), home),
    ]


def _dev_service_component() -> dict:
    state = _read_dev_service_state()
    if not state:
        return {
            "kind": "ornnlab-service",
            "state": "stopped",
            "endpoint": None,
            "logsPath": str(_dev_service_logs_dir()),
            "error": None,
            "actions": ["check-update", "restart-service"],
        }
    daemon_alive = _pid_token_alive(state.get("daemonPid"), state.get("daemonToken"))
    backend_alive = _pid_token_alive(state.get("backendPid"), state.get("backendToken"))
    frontend_alive = _pid_token_alive(state.get("frontendPid"), state.get("frontendToken"))
    frontend_healthy = frontend_alive and _health_endpoint_ok(str(state.get("frontendUrl", "")))
    running = (
        state.get("status") == "running"
        and daemon_alive
        and backend_alive
        and frontend_healthy
    )
    service_state = state.get("status", "unknown")
    if daemon_alive and service_state == "running" and not (backend_alive and frontend_healthy):
        service_state = "degraded"
    if running:
        service_state = "running"
    elif service_state not in {"starting", "restarting", "degraded", "stopped", "error"}:
        service_state = "degraded" if daemon_alive else "stopped"
    return {
        "kind": "ornnlab-service",
        "state": service_state,
        "endpoint": state.get("frontendUrl") or None,
        "logsPath": str(_dev_service_logs_dir()),
        "error": state.get("lastError") if service_state in {"degraded", "error"} else None,
        "actions": ["check-update", "restart-service"],
    }


def _harbor_component(version: str | None, executable: str) -> dict:
    return {
        "kind": "harbor-cli",
        "state": "installed" if version else "not-installed",
        "version": version,
        "executablePath": executable,
        "actions": [],
    }


def _docker_component(doctor: dict) -> dict:
    scan = doctor.get("ornnlab_orphans") or {}
    command = scan.get("command") or [doctor.get("cli", "docker")]
    executable = str(command[0])
    error = scan.get("error")
    if not doctor.get("available"):
        state = "not-installed"
    elif scan.get("ok"):
        state = "running"
    elif _is_docker_not_running_error(error):
        state = "not-running"
    else:
        state = "error"
    client_version, server_version = _docker_versions(command)
    _log_probe_transition("docker", state, error)
    return {
        "kind": "docker",
        "state": state,
        "context": _docker_context(command),
        "clientVersion": client_version,
        "serverVersion": server_version,
        "executablePath": executable,
        "error": error,
        "actions": ["clean-docker-cache"] if state == "running" else [],
    }


def _docker_versions(command: list[str]) -> tuple[str | None, str | None]:
    try:
        result = subprocess.run(
            [*command, "version", "--format", "{{json .}}"],
            capture_output=True,
            text=True,
            timeout=3,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None, None
    try:
        payload = json.loads(result.stdout)
    except (json.JSONDecodeError, TypeError):
        return _docker_client_version(command), None
    client = payload.get("Client") if isinstance(payload, dict) else None
    server = payload.get("Server") if isinstance(payload, dict) else None
    return _version_value(client), _version_value(server)


def _docker_client_version(command: list[str]) -> str | None:
    try:
        result = subprocess.run(
            [*command, "--version"],
            capture_output=True,
            text=True,
            timeout=3,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    match = re.search(r"Docker version\s+([^,\s]+)", result.stdout, re.IGNORECASE)
    return match.group(1) if match else None


def _version_value(section: object) -> str | None:
    if not isinstance(section, dict):
        return None
    value = section.get("Version")
    return str(value) if value else None


def _docker_context(command: list[str]) -> str | None:
    try:
        result = subprocess.run(
            [*command, "context", "show"],
            capture_output=True,
            text=True,
            timeout=3,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return os.environ.get("DOCKER_CONTEXT")
    return result.stdout.strip() or os.environ.get("DOCKER_CONTEXT")


def _is_docker_not_running_error(error: object) -> bool:
    message = str(error or "").lower()
    return any(
        marker in message
        for marker in (
            "cannot connect",
            "connection refused",
            "failed to connect",
            "is the docker daemon running",
            "no such file or directory",
            "docker desktop is not running",
        )
    )


def _cache_component(path: Path) -> dict:
    try:
        size = _directory_size(path)
        state, error = "available", None
    except OSError as exc:
        size, state, error = None, "unavailable", str(exc)
        _log_probe_transition("storage", state, error)
    else:
        _log_probe_transition("storage", state)
    return {
        "kind": "storage",
        "state": state,
        "sizeBytes": size,
        "path": str(path),
        "error": error,
        "actions": ["clean-storage-cache"] if state == "available" else [],
    }


def _cpu_component() -> dict:
    try:
        usage = round(psutil.cpu_percent(interval=0.1), 1)
        component = {
            "kind": "resource-cpu",
            "state": _usage_state(usage),
            "usagePercent": usage,
            "logicalCores": psutil.cpu_count(logical=True),
            "actions": [],
        }
        _log_probe_transition("resource-cpu", component["state"])
        return component
    except Exception as exc:
        _log_probe_transition("resource-cpu", "unavailable", exc)
        return {
            "kind": "resource-cpu",
            "state": "unavailable",
            "usagePercent": None,
            "logicalCores": None,
            "actions": [],
        }


def _gpu_component() -> dict:
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"],
            capture_output=True,
            text=True,
            check=True,
            timeout=3,
        )
        values = [float(line) for line in result.stdout.splitlines() if line.strip()]
    except FileNotFoundError:
        _log_probe_transition("resource-gpu", "not-detected")
        return _gpu_unavailable("not-detected")
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired, ValueError) as exc:
        _log_probe_transition("resource-gpu", "error", exc)
        return _gpu_unavailable("error")
    if not values:
        _log_probe_transition("resource-gpu", "not-detected")
        return _gpu_unavailable("not-detected")
    usage = round(sum(values) / len(values), 1)
    component = {
        "kind": "resource-gpu",
        "state": _usage_state(usage),
        "usagePercent": usage,
        "deviceCount": len(values),
        "actions": [],
    }
    _log_probe_transition("resource-gpu", component["state"])
    return component


def _gpu_unavailable(state: str) -> dict:
    return {
        "kind": "resource-gpu",
        "state": state,
        "usagePercent": None,
        "deviceCount": 0,
        "actions": [],
    }


def _disk_component(disk, path: Path) -> dict:
    if disk is None:
        _log_probe_transition("resource-storage", "unavailable", "disk usage unavailable")
        return {
            "kind": "resource-storage",
            "state": "unavailable",
            "availableBytes": None,
            "totalBytes": None,
            "path": str(path),
            "actions": [],
        }
    state = _disk_state(disk.free, disk.total)
    _log_probe_transition("resource-storage", state)
    return {
        "kind": "resource-storage",
        "state": state,
        "availableBytes": disk.free,
        "totalBytes": disk.total,
        "path": str(path),
        "actions": [],
    }


def _usage_state(usage: float) -> str:
    if usage >= 90:
        return "high"
    if usage >= 70:
        return "elevated"
    return "normal"


def _disk_state(available: int, total: int) -> str:
    ratio = available / total if total > 0 else 0
    if available < 5 * 1024**3 or ratio < 0.05:
        return "critical"
    if available < 20 * 1024**3 or ratio < 0.15:
        return "low"
    return "normal"


def _log_probe_transition(kind: str, state: str, error: object = None) -> None:
    signature = (state, str(error or ""))
    previous = _probe_signatures.get(kind)
    if previous == signature:
        return
    _probe_signatures[kind] = signature
    if error is not None:
        logger.warning("system_health_probe_failed kind=%s state=%s error=%s", kind, state, error)
    elif previous is not None and previous[1]:
        logger.info("system_health_probe_recovered kind=%s state=%s", kind, state)


def _read_dev_service_state() -> dict | None:
    try:
        return json.loads((_dev_service_home() / "state.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None


def _dev_service_home() -> Path:
    configured = os.environ.get("ORNNLAB_DEV_SERVICE_HOME")
    if configured:
        return Path(configured).expanduser()
    launcher_home = Path(
        os.environ.get("ORNNLAB_LAUNCHER_HOME", "~/.ornnlab/launcher")
    ).expanduser()
    return launcher_home.parent / "dev-service"


def _dev_service_logs_dir() -> Path:
    return _dev_service_home() / "logs"


def _pid_token_alive(pid: object, token: object) -> bool:
    if not isinstance(pid, int) or pid < 1 or not isinstance(token, str) or not token:
        return False
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return token in _process_command(pid)


def _process_command(pid: int) -> str:
    command = (
        ["wmic", "process", "where", f"ProcessId={pid}", "get", "CommandLine", "/value"]
        if platform.system() == "Windows"
        else ["ps", "-p", str(pid), "-o", "command="]
    )
    result = subprocess.run(command, capture_output=True, text=True, timeout=3)
    return result.stdout if result.returncode == 0 else ""


def _health_endpoint_ok(url: str) -> bool:
    parsed = urlparse(url)
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        return False
    connection_class = (
        http.client.HTTPSConnection
        if parsed.scheme == "https"
        else http.client.HTTPConnection
    )
    path = parsed.path or "/"
    if parsed.query:
        path = f"{path}?{parsed.query}"
    connection = connection_class(parsed.hostname, parsed.port, timeout=0.5)
    try:
        connection.request("GET", path)
        response = connection.getresponse()
        response.read()
        return 200 <= response.status < 300
    except (OSError, http.client.HTTPException):
        return False
    finally:
        connection.close()


def _directory_size(path: Path) -> int:
    if not path.exists():
        return 0
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def _disk_usage(path: Path):
    try:
        return shutil.disk_usage(path)
    except OSError:
        return None
