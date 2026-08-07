from __future__ import annotations

import time
from pathlib import Path
from typing import Any

from ornnlab.services.dataset_download_state import stored_dataset_runtime
from ornnlab.services.dataset_task_catalog import LocalDatasetTaskCatalog
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

_TASK_CATALOG = LocalDatasetTaskCatalog()
_TASK_NAMES_CACHE: dict[str, tuple[float, list[str] | None]] = {}
_CACHE_TTL_SECONDS = 60.0


async def pending_task_names(
    settings: Settings,
    ref: str,
    started: set[str],
    n_total: int | None,
) -> list[str]:
    all_tasks = _all_task_names(settings, ref)
    if all_tasks is None:
        all_tasks = await _registry_task_names(ref)
    if all_tasks is None:
        return _numbered_pending(started, n_total)
    remaining = max(0, (n_total if isinstance(n_total, int) else len(all_tasks)) - len(started))
    return [name for name in all_tasks if name not in started][:remaining]


def _all_task_names(settings: Settings, ref: str) -> list[str] | None:
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn, "SELECT * FROM webui_datasets WHERE ref = ? AND deleted_at IS NULL", (ref,)
        )
    if not rows:
        return None
    runtime = stored_dataset_runtime(rows[0])
    if not runtime or runtime["download"]["status"] != "downloaded":
        return None
    page = _TASK_CATALOG.list_page(Path(runtime["download"]["path"]), ref, None, 0, 10_000)
    return [item["name"] for item in page.items]


async def _registry_task_names(ref: str) -> list[str] | None:
    cached = _TASK_NAMES_CACHE.get(ref)
    if cached is not None and time.monotonic() - cached[0] < _CACHE_TTL_SECONDS:
        return cached[1]
    try:
        metadata = await _registry_client_factory().create().get_dataset_metadata(ref)
        names = [task_id.get_name() for task_id in metadata.task_ids]
    except Exception:
        names = None
    _TASK_NAMES_CACHE[ref] = (time.monotonic(), names)
    return names


def _registry_client_factory() -> Any:
    from harbor.registry.client.factory import RegistryClientFactory

    return RegistryClientFactory


def _numbered_pending(started: set[str], n_total: int | None) -> list[str]:
    if not isinstance(n_total, int):
        return []
    return [f"Task {index}" for index in range(len(started) + 1, n_total + 1)]
