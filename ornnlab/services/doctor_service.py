from __future__ import annotations

import os
from importlib import metadata
from pathlib import Path
from typing import Any

from ornnlab.services.docker_orphan_service import DockerOrphanService
from ornnlab.services.recovery_service import RunRecoveryService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

RUNTIME_ENV_PAIRS = {
    "ORNNLAB_HARBOR_ENGINE": "HARNESSLAB_HARBOR_ENGINE",
    "ORNNLAB_HARBOR_SUBPROCESS_COMMAND": "HARNESSLAB_HARBOR_SUBPROCESS_COMMAND",
    "ORNNLAB_DOCKER_COMMAND": "HARNESSLAB_DOCKER_COMMAND",
    "ORNNLAB_REAL_HARBOR": "HARNESSLAB_REAL_HARBOR",
    "ORNNLAB_REAL_HARBOR_AGENT": "HARNESSLAB_REAL_HARBOR_AGENT",
    "ORNNLAB_REAL_HARBOR_BENCHMARK": "HARNESSLAB_REAL_HARBOR_BENCHMARK",
    "ORNNLAB_REAL_HARBOR_BENCHMARK_VERSION": "HARNESSLAB_REAL_HARBOR_BENCHMARK_VERSION",
    "ORNNLAB_REAL_HARBOR_N_TASKS": "HARNESSLAB_REAL_HARBOR_N_TASKS",
    "ORNNLAB_REAL_HARBOR_CANCEL_DELAY": "HARNESSLAB_REAL_HARBOR_CANCEL_DELAY",
}


class DoctorService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def status(self, include_logs: bool = False) -> dict:
        schema_version = sqlite.initialize(self.settings)
        docker_orphans = DockerOrphanService().scan_ornnlab_containers()
        stale_running_runs = RunRecoveryService(self.settings).stale_running_count()
        status = {
            "harbor_version": self._package_version("harbor"),
            "docker": {
                "cli": docker_orphans["command"][0],
                "available": docker_orphans["available"],
                "ornnlab_orphans": docker_orphans,
                "harnesslab_orphans": docker_orphans,
            },
            "data_dir": str(self.settings.home),
            "db_path": str(self.settings.db_path),
            "db_schema_version": schema_version,
            "stale_running_runs": stale_running_runs,
            "migration": self.settings.migration,
            "warnings": self._warnings(docker_orphans, stale_running_runs),
            "runtime_env_warnings": _runtime_env_warnings(),
        }
        if self.settings.warnings:
            status["legacy_warnings"] = list(self.settings.warnings)
        if self.settings.migration:
            status["using_legacy_home"] = "using_legacy_home" in self.settings.warnings
            status["migrated_home"] = self.settings.migration.get("ok", False)
            status["legacy_env_in_use"] = "legacy_env_in_use" in self.settings.warnings
            status["migration_error"] = self.settings.migration.get("error")
        if "legacy_env_in_use" in self.settings.warnings:
            status["legacy_env_in_use"] = True
            status["using_legacy_home"] = True
        if include_logs:
            status["logs"] = self.logs_report(status)
        return status

    def docker_orphans(self) -> dict:
        return DockerOrphanService().scan_ornnlab_containers()

    def logs_report(self, status: dict[str, Any] | None = None) -> dict[str, Any]:
        sqlite.initialize(self.settings)
        latest = self._latest_failed_run()
        return {
            "latest_failed_run": _run_log_payload(latest) if latest else None,
            "remediation": _remediation(status or self.status(), latest),
        }

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
        warnings.extend(docker_orphans.get("warnings", []))
        warnings.extend(_runtime_env_warnings())
        warnings.extend(self.settings.warnings)
        return list(dict.fromkeys(warnings))

    @staticmethod
    def _package_version(package: str) -> str | None:
        try:
            return metadata.version(package)
        except metadata.PackageNotFoundError:
            return None

    def _latest_failed_run(self) -> dict[str, Any] | None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                """
                SELECT
                  runs.id,
                  runs.experiment_id,
                  experiments.name AS experiment_name,
                  runs.status,
                  runs.failure_class,
                  runs.failure_code,
                  runs.failure_summary,
                  runs.job_dir,
                  runs.result_path,
                  runs.report_path,
                  runs.finished_at,
                  runs.updated_at
                FROM runs
                JOIN experiments ON experiments.id = runs.experiment_id
                WHERE runs.status IN ('failed', 'interrupted')
                ORDER BY COALESCE(runs.finished_at, runs.updated_at) DESC
                LIMIT 1
                """,
            )
        return rows[0] if rows else None


def _run_log_payload(run: dict[str, Any]) -> dict[str, Any]:
    job_dir = run.get("job_dir")
    report_path = run.get("report_path")
    fallback_result = str(Path(job_dir) / "result.json") if job_dir else None
    return {
        "run_id": run["id"],
        "experiment_id": run["experiment_id"],
        "experiment_name": run["experiment_name"],
        "status": run["status"],
        "failure_class": run.get("failure_class"),
        "failure_code": run.get("failure_code"),
        "failure_summary": run.get("failure_summary"),
        "finished_at": run.get("finished_at"),
        "updated_at": run.get("updated_at"),
        "paths": {
            "job_dir": job_dir,
            "job_log": str(Path(job_dir) / "job.log") if job_dir else None,
            "result": run.get("result_path") or fallback_result,
            "report": report_path,
            "report_summary": str(Path(report_path).with_name("summary.json"))
            if report_path
            else None,
        },
    }


def _remediation(status: dict[str, Any], latest: dict[str, Any] | None) -> list[str]:
    actions: list[str] = []
    warnings = set(status.get("warnings", []))
    if "docker_cli_missing" in warnings:
        actions.append("install_or_start_docker")
    if "docker_orphan_scan_failed" in warnings:
        actions.append("check_docker_context")
    if "docker_orphans_detected" in warnings:
        actions.append("review_docker_orphan_cleanup_plan")
    if "stale_running_runs" in warnings:
        actions.append("restart_recovery_or_inspect_sqlite")
    if latest and latest.get("failure_class") == "docker_resource_failure":
        actions.append("check_docker_resources")
    if latest and latest.get("failure_class") == "harbor_recovery":
        actions.append("inspect_harbor_job_dir")
    return actions


def _runtime_env_warnings() -> list[str]:
    warnings: list[str] = []
    for new_name, legacy_name in RUNTIME_ENV_PAIRS.items():
        legacy_warning = _legacy_warning_name(legacy_name)
        if os.environ.get(new_name) and os.environ.get(legacy_name):
            warnings.append(f"{legacy_warning}_ignored")
        elif os.environ.get(legacy_name):
            warnings.append(f"{legacy_warning}_in_use")
    return warnings


def _legacy_warning_name(env_name: str) -> str:
    return env_name.lower().replace("harnesslab_", "legacy_")
