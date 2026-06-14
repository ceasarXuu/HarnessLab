from __future__ import annotations

import shutil
from importlib import metadata

from harnesslab.services.recovery_service import RunRecoveryService
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class DoctorService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def status(self) -> dict:
        schema_version = sqlite.initialize(self.settings)
        return {
            "harbor_version": self._package_version("harbor"),
            "docker": {
                "cli": shutil.which("docker"),
                "available": shutil.which("docker") is not None,
            },
            "data_dir": str(self.settings.home),
            "db_path": str(self.settings.db_path),
            "db_schema_version": schema_version,
            "stale_running_runs": RunRecoveryService(self.settings).stale_running_count(),
            "warnings": self._warnings(),
        }

    def _warnings(self) -> list[str]:
        warnings: list[str] = []
        if not self.settings.home.exists():
            warnings.append("data_dir_missing")
        if shutil.which("docker") is None:
            warnings.append("docker_cli_missing")
        if self._package_version("harbor") is None:
            warnings.append("harbor_package_missing")
        if RunRecoveryService(self.settings).stale_running_count() > 0:
            warnings.append("stale_running_runs")
        return warnings

    @staticmethod
    def _package_version(package: str) -> str | None:
        try:
            return metadata.version(package)
        except metadata.PackageNotFoundError:
            return None
