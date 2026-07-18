from __future__ import annotations

import logging
from pathlib import Path

from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


def merge_active_downloads(settings: Settings, datasets: list[dict]) -> list[dict]:
    active = _active_downloads(settings)
    if not active:
        return datasets
    merged = []
    for dataset in datasets:
        operation = active.get(dataset["ref"])
        if operation and dataset["download"]["status"] != "downloaded":
            dataset = {
                **dataset,
                "download": {
                    "status": "downloading",
                    "progress": operation.get("progress") or 0,
                },
            }
        merged.append(dataset)
    logger.debug("Merged active Dataset downloads count=%s", len(active))
    return merged


def remote_dataset_dto(name: str, version: str | None, task_count: int) -> dict:
    resolved_version = version or "latest"
    return {
        "ref": f"{name}@{resolved_version}",
        "name": name,
        "version": resolved_version,
        "visibility": "public",
        "taskCount": task_count,
        "source": "harbor registry",
        "download": {"status": "not-downloaded"},
        "registryUrl": "https://hub.harborframework.com",
    }


def stored_dataset_dto(row: dict) -> dict:
    local_path = row.get("local_path")
    storage_kind = row.get("storage_kind") or (
        "external" if row.get("source") == "local" else "managed"
    )
    path = Path(local_path) if isinstance(local_path, str) else None
    if path and path.is_dir():
        download = {
            "path": local_path,
            "sizeBytes": sum(item.stat().st_size for item in path.rglob("*") if item.is_file()),
            "status": "downloaded",
            "storageKind": storage_kind,
            "updatedAt": row.get("updated_at"),
        }
    elif path:
        download = {"path": local_path, "status": "path-unavailable", "storageKind": storage_kind}
    else:
        download = {"status": "not-downloaded"}
    return {
        "ref": row["ref"],
        "name": row["name"],
        "version": row["version"],
        "visibility": row["visibility"],
        "taskCount": row["task_count"],
        "source": row["source"],
        "download": download,
        "registryUrl": row.get("registry_url"),
    }


def stored_dataset_runtime(row: dict | None) -> dict | None:
    """Return only the local-path state needed by Task reads, without a size scan."""
    if row is None:
        return None
    local_path = row.get("local_path")
    path = Path(local_path) if isinstance(local_path, str) else None
    if path and path.is_dir():
        download = {"path": local_path, "status": "downloaded"}
    elif path:
        download = {"path": local_path, "status": "path-unavailable"}
    else:
        download = {"status": "not-downloaded"}
    return {"download": download, "source": row["source"]}


def _active_downloads(settings: Settings) -> dict[str, dict]:
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn,
            "SELECT resource_id, progress FROM webui_operations "
            "WHERE operation_type = 'download-dataset' AND resource_type = 'dataset' "
            "AND status IN ('queued', 'running') ORDER BY created_at DESC",
        )
    active: dict[str, dict] = {}
    for row in rows:
        if resource_id := row.get("resource_id"):
            active.setdefault(resource_id, row)
    return active
