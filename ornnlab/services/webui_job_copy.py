from __future__ import annotations

import json
import logging

from ornnlab.services.webui_job_query import JOB_SELECT
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


def load_job_copy_config(settings: Settings, job_id: str) -> dict:
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(conn, JOB_SELECT + "WHERE runs.id = ?", (job_id,))
    if not rows:
        raise KeyError(job_id)
    row = rows[0]
    config = json.loads(row["config_json"]) if row.get("config_json") else {}
    if not config:
        raise ValueError("the original Job configuration is unavailable")
    logger.info("webui.job.copy_config_loaded job_id=%s", job_id)
    return build_job_copy_config(row, config)


def build_job_copy_config(row: dict, config: dict) -> dict:
    overrides = config.get("harbor_overrides", {})
    retry = overrides.get("retry", {})
    metrics = overrides.get("metrics", [])
    metric = metrics[0].get("type", "mean") if metrics else "mean"
    return {
        "agentSetupTimeoutMultiplier": overrides.get("agent_setup_timeout_multiplier", 1),
        "agentName": config.get("agent_name", row["agent_profile_name"]),
        "agentTimeoutMultiplier": overrides.get("agent_timeout_multiplier", 1),
        "attempts": int(row["n_attempts"]),
        "concurrency": int(row["n_concurrent"]),
        "datasetRef": _join_ref(row["benchmark_name"], row["benchmark_version"]),
        "debug": bool(overrides.get("debug", False)),
        "environmentPresetId": config.get("environment_preset_id", ""),
        "environmentBuildTimeoutMultiplier": overrides.get(
            "environment_build_timeout_multiplier", 1
        ),
        "extraInstructionPaths": overrides.get("extra_instruction_paths", []),
        "includeInLeaderboard": bool(row["leaderboard_eligible"]),
        "jobName": f"{config.get('job_name', row['experiment_name'])}-copy",
        "jobsDir": config.get("jobs_dir", row.get("job_dir") or "jobs/new-job"),
        "maxRetries": int(retry.get("max_retries", 0)),
        "metric": metric,
        "modelName": config.get("model", ""),
        "notes": row.get("job_notes") or "",
        "retryExclude": _join_values(retry.get("exclude_exceptions")),
        "retryInclude": _join_values(retry.get("include_exceptions")),
        "retryMaxWaitSeconds": retry.get("max_wait_sec", 30),
        "retryMinWaitSeconds": retry.get("min_wait_sec", 2),
        "retryWaitMultiplier": retry.get("wait_multiplier", 1.5),
        "selectedTaskNames": overrides.get("task_names"),
        "timeoutMultiplier": overrides.get("timeout_multiplier", 1),
        "verifierTimeoutMultiplier": overrides.get("verifier_timeout_multiplier", 1),
        "verifierMode": "skip"
        if overrides.get("verifier", {}).get("disable")
        else "dataset-default",
    }


def _join_values(values: object) -> str:
    if not isinstance(values, list):
        return ""
    return ", ".join(str(value) for value in values)


def _join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name
