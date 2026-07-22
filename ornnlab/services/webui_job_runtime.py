from __future__ import annotations

import logging
from pathlib import Path

from ornnlab.services.harbor_results import load_result_payload

logger = logging.getLogger(__name__)


def load_job_result(row: dict) -> dict:
    persisted = row.get("result_path")
    if persisted:
        return load_result_payload(Path(str(persisted)))
    path = _live_native_result_path(row)
    if path is None:
        return {}
    result = load_result_payload(path)
    if result:
        stats_value = result.get("stats")
        stats = stats_value if isinstance(stats_value, dict) else {}
        logger.debug(
            "job_progress.live_result_loaded job_id=%s total=%s completed=%s errored=%s",
            row.get("id"),
            result.get("n_total_trials"),
            stats.get("n_completed_trials"),
            stats.get("n_errored_trials"),
        )
    return result


def _live_native_result_path(row: dict) -> Path | None:
    if row.get("status") != "running" or not row.get("job_dir"):
        return None
    job_name = row.get("harbor_job_name")
    if not isinstance(job_name, str) or not _safe_child_name(job_name):
        return None
    return Path(str(row["job_dir"])) / job_name / "result.json"


def _safe_child_name(value: str) -> bool:
    return value not in {"", ".", ".."} and Path(value).name == value
