from __future__ import annotations

from pathlib import Path

from ornnlab.services.harbor_paths import resolve_harbor_log_path


def event_log_path(row: dict, config: dict) -> str | None:
    if not row.get("job_dir"):
        return None
    return str(resolve_harbor_log_path(Path(row["job_dir"]), config.get("job_name")))


def job_log_payload(row: dict, config: dict) -> dict:
    path = event_log_path(row, config)
    if not path:
        return {"logPath": None, "content": ""}
    return {"logPath": path, "content": read_log_tail(Path(path))}


def read_log_tail(path: Path, max_chars: int = 200_000) -> str:
    try:
        size = path.stat().st_size
    except OSError:
        return ""
    if size <= max_chars:
        try:
            return path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            return ""
    try:
        with path.open("rb") as handle:
            handle.seek(size - max_chars)
            return handle.read().decode("utf-8", errors="replace").lstrip("\n")
    except OSError:
        return ""
