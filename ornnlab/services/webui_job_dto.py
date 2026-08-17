from __future__ import annotations

import json
from datetime import datetime
from pathlib import Path

from ornnlab.services.harbor_results import token_usage_m
from ornnlab.services.harbor_score import result_score
from ornnlab.services.model_pricing import calculate_cost
from ornnlab.services.webui_job_logs import event_log_path
from ornnlab.services.webui_job_progress import (
    TERMINAL_STATUSES,
    execution_completed,
    job_trial_progress,
    runtime_seconds,
    trial_progress_total,
)
from ornnlab.services.webui_job_resume import can_resume_job
from ornnlab.services.webui_job_runtime import load_job_result


def job_dto(row: dict) -> dict:
    config = job_config(row)
    result = load_job_result(row)
    status = str(row["status"])
    expected_total = trial_progress_total(row)
    if status in TERMINAL_STATUSES and execution_completed(result, expected_total):
        status = "completed"
    stats = result.get("stats", {})
    trial = job_trial_progress(
        result,
        expected_total=expected_total,
        terminal_without_result=status in {"completed", "failed", "cancelled", "interrupted"},
    )
    return {
        "id": row["id"],
        "name": config.get("job_name", row.get("experiment_name", row["id"])),
        "status": status,
        "datasetRef": join_ref(row["benchmark_name"], row["benchmark_version"]),
        "agentName": config.get("agent_name", row.get("agent_profile_name", row["agent_id"])),
        "harness": config.get("agent_harness", row["agent_id"]),
        "model": config.get("model", ""),
        "environmentName": config.get("environment_name", config.get("environment_preset_id", "")),
        "trial": trial,
        "score": job_score(result),
        "costUsd": calculate_cost(stats, config.get("pricing")),
        "tokenUsageM": token_usage_m(stats),
        "runtimeSeconds": runtime_seconds(row.get("started_at"), row.get("finished_at")),
        "createdAt": row["created_at"],
        "includeInLeaderboard": bool(row["leaderboard_eligible"]),
        "canResume": can_resume_job(row, status),
        "jobDir": row.get("job_dir"),
        "eventLogPath": event_log_path(row, config),
        "artifactPaths": artifacts(row),
        "failureCode": row.get("failure_code"),
    }


def job_config(row: dict) -> dict:
    return json.loads(row["config_json"]) if row.get("config_json") else {}


def dataset_ref(ref: str) -> tuple[str, str | None]:
    name, separator, version = ref.rpartition("@")
    return (name, version) if separator else (ref, None)


def join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name


def exception_list(value: str) -> list[str] | None:
    values = [item.strip() for item in value.replace(",", "\n").splitlines() if item.strip()]
    return values or None


def event_level(severity: str) -> str:
    return {"error": "error", "warning": "warning"}.get(
        severity, "success" if severity == "info" else "info"
    )


def job_score(result: dict) -> dict | None:
    """Expose scores whose 0..1 scale is explicit in Harbor's result payload."""
    value = result_score(result)
    if value is not None:
        return {"kind": "percentage", "value": value * 100}
    return None


def version_filter(version: str | None) -> str:
    return "runs.benchmark_version IS NULL" if version is None else "runs.benchmark_version = ?"


def artifacts(row: dict) -> list[str]:
    values = [row.get("result_path"), row.get("report_path")]
    if row.get("job_dir"):
        values.append(str(Path(row["job_dir"]) / "harbor.config.json"))
    return [value for value in values if value]


def now() -> str:
    return datetime.now().astimezone().isoformat()
