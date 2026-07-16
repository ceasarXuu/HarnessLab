from __future__ import annotations

import asyncio
import json
import logging
import os
import platform
import shutil
import subprocess
import sys
from pathlib import Path

from packaging.version import InvalidVersion, Version

from ornnlab.services.command_line import split_command
from ornnlab.services.doctor_service import DoctorService
from ornnlab.services.system_health_probe import probe_system_health
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.settings import Settings

logger = logging.getLogger(__name__)


class WebUiSystemService:
    def __init__(self, settings: Settings, operations: WebUiOperationService):
        self.settings = settings
        self.operations = operations

    def health(self) -> list[dict]:
        doctor = DoctorService(self.settings).status()
        return probe_system_health(self.settings.home, doctor, _harbor_executable())

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
