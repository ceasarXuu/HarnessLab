from __future__ import annotations

import asyncio
import http.client
import json
import logging
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path
from urllib.parse import urlparse

from packaging.version import InvalidVersion, Version

from ornnlab.services.command_line import split_command
from ornnlab.services.doctor_service import DoctorService
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.settings import Settings

logger = logging.getLogger(__name__)


class WebUiSystemService:
    def __init__(self, settings: Settings, operations: WebUiOperationService):
        self.settings = settings
        self.operations = operations

    def health(self) -> list[dict]:
        doctor = DoctorService(self.settings).status()
        cache_dir = Path("~/.cache/harbor").expanduser()
        disk = _disk_usage(self.settings.home)
        cpu_usage = _cpu_usage()
        gpu_usage = _gpu_usage()
        service = _dev_service_component(self.settings)
        return [
            service,
            _component(
                "harbor-cli",
                "Harbor CLI",
                "healthy" if doctor["harbor_version"] else "failed",
                doctor["harbor_version"] or "not installed",
                _harbor_executable(),
                [],
            ),
            _component(
                "docker",
                "Docker",
                "healthy" if doctor["docker"]["available"] else "failed",
                "available" if doctor["docker"]["available"] else "not available",
                doctor["docker"]["cli"],
                ["clean-docker-cache"],
            ),
            _component(
                "storage",
                "Storage",
                "healthy",
                _format_bytes(_directory_size(cache_dir)),
                str(cache_dir),
                ["clean-storage-cache"],
            ),
            _component(
                "resource-cpu", "CPU", _resource_status(cpu_usage), cpu_usage, "local host", []
            ),
            _component(
                "resource-gpu", "GPU", _resource_status(gpu_usage), gpu_usage, "local host", []
            ),
            _component(
                "resource-storage",
                "Available storage",
                "healthy" if disk is not None else "unavailable",
                _format_bytes(disk.free) if disk is not None else "unavailable",
                str(self.settings.home),
                [],
            ),
        ]

    async def hub_connection(self) -> dict:
        try:
            from harbor.auth.handler import get_auth_handler

            authenticated = await (await get_auth_handler()).is_authenticated()
        except Exception:
            return {"status": "disconnected"}
        return {"status": "connected" if authenticated else "disconnected"}

    async def check_update(self) -> dict:
        current = _npm_version()
        latest = await asyncio.to_thread(_latest_npm_version)
        return {
            "currentVersion": current,
            "latestVersion": latest,
            "updateAvailable": _version_changed(current, latest),
            "releaseNotesUrl": "https://www.npmjs.com/package/ornnlab",
        }

    def install_update(self) -> dict:
        async def work(progress) -> None:
            progress(10, "Installing OrnnLab update")
            await asyncio.to_thread(_run_checked, ["npm", "install", "-g", "ornnlab@latest"])
            progress(100, "OrnnLab update installed")

        return self.operations.submit("install-system-update", "system", "ornnlab-service", work)

    def restart(self) -> dict:
        command = os.environ.get("ORNNLAB_RESTART_COMMAND")
        if not command:
            return self.operations.fail(
                "restart-system-service",
                "system",
                "ornnlab-service",
                "SERVICE_RESTART_UNAVAILABLE",
                "ORNNLAB_RESTART_COMMAND is not configured by the service supervisor",
            )

        async def work(progress) -> None:
            progress(10, "Requesting service restart")
            await asyncio.to_thread(_run_checked, split_command(command))
            progress(100, "Service restart requested")

        return self.operations.submit("restart-system-service", "system", "ornnlab-service", work)

    def clean_docker_cache(self) -> dict:
        async def work(progress) -> None:
            progress(10, "Cleaning Harbor Docker cache")
            await asyncio.to_thread(
                _run_checked, [_harbor_executable(), "cache", "clean", "--force", "--no-cache-dir"]
            )
            progress(100, "Harbor Docker cache cleaned")

        return self.operations.submit("clean-docker-cache", "system", "docker", work)

    def clean_storage_cache(self) -> dict:
        async def work(progress) -> None:
            cache_dir = Path("~/.cache/harbor").expanduser()
            progress(10, "Cleaning Harbor local cache")
            await asyncio.to_thread(_remove_cache_dir, cache_dir)
            progress(100, "Harbor local cache cleaned")

        return self.operations.submit("clean-storage-cache", "system", "storage", work)

    def choose_directory(self) -> dict:
        path = _choose_native_directory()
        if path:
            logger.info("Selected native directory path=%s", path)
        else:
            logger.info("Native directory selection cancelled")
        return {"path": path}


def _component(
    kind: str, component: str, status: str, value: str, path: str, actions: list[str]
) -> dict:
    return {
        "kind": kind,
        "component": component,
        "status": status,
        "value": value,
        "path": path,
        "actions": actions,
    }


def _dev_service_component(settings: Settings) -> dict:
    state = _read_dev_service_state()
    if not state:
        return _component(
            "ornnlab-service",
            "OrnnLab Service",
            "unavailable",
            "stopped",
            str(_dev_service_logs_dir()),
            ["check-update", "restart-service"],
        )
    daemon_alive = _pid_token_alive(state.get("daemonPid"), state.get("daemonToken"))
    backend_alive = _pid_token_alive(state.get("backendPid"), state.get("backendToken"))
    frontend_alive = _pid_token_alive(state.get("frontendPid"), state.get("frontendToken"))
    backend_healthy = backend_alive
    frontend_healthy = frontend_alive and _health_endpoint_ok(str(state.get("frontendUrl", "")))
    running = (
        state.get("status") == "running"
        and daemon_alive
        and backend_healthy
        and frontend_healthy
    )
    status = "healthy" if running else "unavailable"
    service_state = state.get("status", "unknown")
    if daemon_alive and service_state == "running" and not (backend_healthy and frontend_healthy):
        service_state = "degraded"
    value = (
        f"{service_state} {state.get('frontendUrl', '')}".strip()
        if state.get("status")
        else _npm_version()
    )
    return _component(
        "ornnlab-service",
        "OrnnLab Service",
        status,
        value,
        str(_dev_service_logs_dir()),
        ["check-update", "restart-service"],
    )


def _read_dev_service_state() -> dict | None:
    state_path = _dev_service_home() / "state.json"
    try:
        return json.loads(state_path.read_text(encoding="utf-8"))
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
    if not isinstance(pid, int) or pid < 1:
        return False
    if not isinstance(token, str) or not token:
        return False
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return token in _process_command(pid)


def _process_command(pid: int) -> str:
    if platform.system() == "Windows":
        result = subprocess.run(
            ["wmic", "process", "where", f"ProcessId={pid}", "get", "CommandLine", "/value"],
            capture_output=True,
            text=True,
            timeout=3,
        )
        return result.stdout if result.returncode == 0 else ""
    result = subprocess.run(
        ["ps", "-p", str(pid), "-o", "command="],
        capture_output=True,
        text=True,
        timeout=3,
    )
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


def _harbor_executable() -> str:
    return (
        os.environ.get("ORNNLAB_HARBOR_CLI")
        or shutil.which("harbor")
        or str(Path(sys.executable).parent / "harbor")
    )


def _npm_version() -> str:
    root = Path(__file__).resolve().parents[2]
    package = json.loads((root / "package.json").read_text(encoding="utf-8"))
    return str(package["version"])


def _latest_npm_version() -> str:
    result = subprocess.run(
        ["npm", "view", "ornnlab", "version"],
        capture_output=True,
        text=True,
        check=True,
        timeout=20,
    )
    return result.stdout.strip()


def _version_changed(current: str, latest: str) -> bool:
    try:
        return Version(latest) > Version(current)
    except InvalidVersion:
        return current != latest


def _run_checked(command: list[str]) -> None:
    result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    if result.returncode:
        raise RuntimeError(
            result.stderr.strip()
            or result.stdout.strip()
            or f"command exited with {result.returncode}"
        )


def _remove_cache_dir(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)


def _directory_size(path: Path) -> int:
    if not path.exists():
        return 0
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def _disk_usage(path: Path):
    try:
        return shutil.disk_usage(path)
    except OSError:
        return None


def _format_bytes(value: int) -> str:
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if value < 1024 or unit == "TB":
            return f"{value:.1f} {unit}"
        value //= 1024
    return "0 B"


def _cpu_usage() -> str:
    get_load_average = getattr(os, "getloadavg", None)
    if get_load_average is None:
        return "unavailable"
    try:
        return f"load {get_load_average()[0]:.2f}"
    except OSError:
        return "unavailable"


def _gpu_usage() -> str:
    try:
        result = subprocess.run(
            ["nvidia-smi", "--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"],
            capture_output=True,
            text=True,
            check=True,
            timeout=3,
        )
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return "not available"
    return f"{result.stdout.strip().splitlines()[0]}%"


def _resource_status(value: str) -> str:
    return "unavailable" if value in {"unavailable", "not available"} else "healthy"


def _choose_native_directory() -> str | None:
    system = platform.system()
    if system == "Darwin":
        return _run_directory_picker(
            [
                "osascript",
                "-e",
                'POSIX path of (choose folder with prompt "Choose a folder for OrnnLab")',
            ],
            cancel_markers=("-128",),
        )
    if system == "Windows":
        return _run_directory_picker(
            [
                "powershell",
                "-NoProfile",
                "-Command",
                (
                    "Add-Type -AssemblyName System.Windows.Forms; "
                    "$dialog = New-Object System.Windows.Forms.FolderBrowserDialog; "
                    "$dialog.Description = 'Choose a folder for OrnnLab'; "
                    "if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) "
                    "{ [Console]::Write($dialog.SelectedPath) }"
                ),
            ],
            cancel_markers=(),
        )
    if system == "Linux":
        for command in (
            ["zenity", "--file-selection", "--directory", "--title=Choose a folder for OrnnLab"],
            ["kdialog", "--getexistingdirectory"],
        ):
            try:
                return _run_directory_picker(command, cancel_returncodes=(1,))
            except FileNotFoundError:
                continue
        raise ValueError("native directory picker is unavailable; install zenity or kdialog")
    raise ValueError(f"native directory picker is unsupported on {system}")


def _run_directory_picker(
    command: list[str],
    *,
    cancel_markers: tuple[str, ...] = (),
    cancel_returncodes: tuple[int, ...] = (),
) -> str | None:
    result = subprocess.run(command, capture_output=True, text=True, timeout=300)
    if result.returncode:
        message = (result.stderr or result.stdout).strip()
        if result.returncode in cancel_returncodes or any(
            marker in message for marker in cancel_markers
        ):
            return None
        raise ValueError(message or f"native directory picker exited with {result.returncode}")
    selected = result.stdout.strip()
    return str(Path(selected).expanduser().resolve()) if selected else None
