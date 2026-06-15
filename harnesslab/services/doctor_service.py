from __future__ import annotations

from importlib import metadata

from harnesslab.services.docker_orphan_service import DockerOrphanService
from harnesslab.services.recovery_service import RunRecoveryService
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class DoctorService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def status(self) -> dict:
        schema_version = sqlite.initialize(self.settings)
        docker_orphans = DockerOrphanService().scan_harnesslab_containers()
        stale_running_runs = RunRecoveryService(self.settings).stale_running_count()
        return {
            "harbor_version": self._package_version("harbor"),
            "docker": {
                "cli": docker_orphans["command"][0],
                "available": docker_orphans["available"],
                "harnesslab_orphans": docker_orphans,
            },
            "data_dir": str(self.settings.home),
            "db_path": str(self.settings.db_path),
            "db_schema_version": schema_version,
            "stale_running_runs": stale_running_runs,
            "warnings": self._warnings(docker_orphans, stale_running_runs),
        }

    def docker_orphans(self) -> dict:
        return DockerOrphanService().scan_harnesslab_containers()

    def _warnings(self, docker_orphans: dict, stale_running_runs: int) -> list[str]:
        warnings: list[str] = []
        if not self.settings.home.exists():
            warnings.append("data_dir_missing")
        if not docker_orphans.get("available"):
            warnings.append("docker_cli_missing")
        if self._package_version("harbor") is None:
            warnings.append("harbor_package_missing")
        if stale_running_runs > 0:
            warnings.append("stale_running_runs")
        if docker_orphans.get("count", 0) > 0:
            warnings.append("docker_orphans_detected")
        if docker_orphans.get("available") and not docker_orphans.get("ok"):
            warnings.append("docker_orphan_scan_failed")
        return warnings

    @staticmethod
    def _package_version(package: str) -> str | None:
        try:
            return metadata.version(package)
        except metadata.PackageNotFoundError:
            return None
