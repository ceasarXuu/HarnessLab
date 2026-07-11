from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlparse

from ornnlab.services.harbor_paths import resolve_harbor_job_path, resolve_harbor_result_path


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
