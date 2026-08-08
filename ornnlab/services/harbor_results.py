from __future__ import annotations

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlparse

from ornnlab.services.harbor_paths import resolve_harbor_job_path, resolve_harbor_result_path
from ornnlab.services.model_pricing import calculate_cost
from ornnlab.services.webui_job_progress import runtime_seconds


def load_result_payload(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def trial_result_payloads(
    jobs_dir: Path,
    job_name: str | None,
    result_path: str | None,
) -> list[dict[str, Any]]:
    """Read trial results from either Harbor's legacy or native result layout."""
    job_result_path = (
        Path(result_path)
        if result_path
        else resolve_harbor_result_path(jobs_dir, job_name)
    )
    job_result = load_result_payload(job_result_path)
    embedded = job_result.get("trial_results")
    if isinstance(embedded, list):
        return [item for item in embedded if isinstance(item, dict)]

    job_path = resolve_harbor_job_path(jobs_dir, job_name)
    return [
        payload
        for path in sorted(job_path.glob("*/result.json"))
        if (payload := load_result_payload(path))
    ]


def running_trial_descriptors(
    jobs_dir: Path,
    job_name: str | None,
    result_path: str | None,
) -> list[dict[str, Any]]:
    """Lightweight descriptors for trials that started but have no result yet."""
    job_result_path = (
        Path(result_path)
        if result_path
        else resolve_harbor_result_path(jobs_dir, job_name)
    )
    job_result = load_result_payload(job_result_path)
    if isinstance(job_result.get("trial_results"), list):
        return []
    if job_result.get("finished_at"):
        return []
    job_path = resolve_harbor_job_path(jobs_dir, job_name)
    descriptors: list[dict[str, Any]] = []
    for path in sorted(job_path.glob("*/")):
        if not path.is_dir() or (path / "result.json").is_file():
            continue
        config = _read_trial_config(path)
        if not config:
            continue
        log_path = path / "trial.log"
        descriptors.append(
            {
                "trial_name": str(config.get("trial_name") or path.name),
                "task_name": _trial_task_name(config, path),
                "log_path": str(log_path) if log_path.is_file() else None,
            }
        )
    return descriptors


def _read_trial_config(path: Path) -> dict[str, Any]:
    try:
        payload = json.loads((path / "config.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _trial_task_name(config: dict[str, Any], path: Path) -> str:
    task = config.get("task")
    task_path = task.get("path") if isinstance(task, dict) else None
    if isinstance(task_path, str) and task_path.strip():
        return Path(task_path).name
    return str(path.name).split("__", 1)[0]


def running_trial_dto(
    job_id: str, descriptor: dict[str, Any], status: str = "running"
) -> dict[str, Any]:
    return {
        "id": str(descriptor["trial_name"]),
        "jobId": job_id,
        "taskName": str(descriptor["task_name"]),
        "status": status,
        "score": None,
        "retryCount": None,
        "runtimeSeconds": None,
        "costUsd": None,
        "tokenUsageM": None,
        "logPath": descriptor.get("log_path"),
        "error": None,
    }


def pending_trial_dto(job_id: str, task_name: str) -> dict[str, Any]:
    return {
        "id": task_name,
        "jobId": job_id,
        "taskName": task_name,
        "status": "pending",
        "score": None,
        "retryCount": None,
        "runtimeSeconds": None,
        "costUsd": None,
        "tokenUsageM": None,
        "logPath": None,
        "error": None,
    }


def trial_start_epoch(item: dict[str, Any]) -> float:
    for key in ("started_at", "finished_at"):
        value = item.get(key)
        if isinstance(value, str):
            try:
                return datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp()
            except ValueError:
                continue
    return 0.0


def trial_dir_epoch(descriptor: dict[str, Any]) -> float:
    log_path = descriptor.get("log_path")
    if not isinstance(log_path, str):
        return 0.0
    try:
        return Path(log_path).parent.stat().st_mtime
    except OSError:
        return 0.0


def trial_dto(job_id: str, item: dict[str, Any], pricing: dict | None = None) -> dict[str, Any]:
    agent_result = item.get("agent_result") or {}
    token_usage = trial_token_usage(agent_result, item.get("step_results"))
    return {
        "id": str(item.get("id", item.get("trial_name", "unknown"))),
        "jobId": job_id,
        "taskName": str(item.get("task_name", item.get("name", "unknown"))),
        "status": "failed" if item.get("exception_info") else "passed",
        "score": verifier_score(item.get("verifier_result")),
        "retryCount": None,
        "runtimeSeconds": runtime_seconds(item.get("started_at"), item.get("finished_at")),
        "costUsd": calculate_cost(token_usage, pricing),
        "tokenUsageM": token_usage_m(token_usage),
        "logPath": trial_log_path(item),
        "error": _trial_error(item),
    }


def _trial_error(item: dict[str, Any], max_chars: int = 200) -> str | None:
    exception = item.get("exception_info")
    if not isinstance(exception, dict):
        return None
    error_type = exception.get("exception_type")
    message = exception.get("exception_message")
    if not error_type and not message:
        return None
    text = f"{error_type}: {message}" if error_type else str(message)
    return text[:max_chars]


def trial_token_usage(agent_result: object, step_results: object) -> dict[str, Any]:
    contexts = [agent_result] if isinstance(agent_result, dict) else []
    if not contexts and isinstance(step_results, list):
        contexts = [item.get("agent_result") for item in step_results if isinstance(item, dict)]
    result: dict[str, Any] = {}
    for context in contexts:
        if not isinstance(context, dict):
            continue
        for key in ("n_input_tokens", "n_cache_tokens", "n_output_tokens", "cost_usd"):
            value = context.get(key)
            if isinstance(value, int | float):
                result[key] = result.get(key, 0.0) + float(value)
    return result


def token_usage_m(stats: dict[str, Any]) -> float | None:
    values = [stats.get("n_input_tokens"), stats.get("n_output_tokens")]
    if not any(isinstance(value, int | float) for value in values):
        return None
    return sum(float(value) for value in values if isinstance(value, int | float)) / 1_000_000


def verifier_score(value: object) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        return None
    rewards = value.get("rewards")
    if not isinstance(rewards, dict):
        return None
    value = rewards.get("pass")
    if isinstance(value, int | float) and value in {0, 1}:
        return {"kind": "percentage", "value": float(value) * 100}
    return None


def trial_log_path(result: dict[str, Any]) -> str | None:
    trial_uri = result.get("trial_uri")
    if not isinstance(trial_uri, str):
        return None
    parsed = urlparse(trial_uri)
    if parsed.scheme != "file":
        return None
    path = Path(_file_uri_path(parsed.path, parsed.netloc)) / "trial.log"
    return str(path) if path.is_file() else None


def _file_uri_path(path: str, host: str, *, windows: bool | None = None) -> str:
    decoded = unquote(path)
    if host and host != "localhost":
        decoded = f"//{host}{decoded}"
    is_windows = os.name == "nt" if windows is None else windows
    if (
        is_windows
        and len(decoded) >= 3
        and decoded[0] == "/"
        and decoded[1].isalpha()
        and decoded[2] == ":"
    ):
        decoded = decoded[1:]
    return decoded
