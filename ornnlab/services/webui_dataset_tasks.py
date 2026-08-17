from __future__ import annotations

import logging
import time
from pathlib import Path

from ornnlab.services.container_image_platforms import resolve_local_task
from ornnlab.services.dataset_directory import join_ref, split_ref
from ornnlab.services.dataset_download_state import stored_dataset_runtime
from ornnlab.services.dataset_task_catalog import LocalDatasetTaskCatalog, page_offset
from ornnlab.settings import Settings

logger = logging.getLogger(__name__)
_task_catalog = LocalDatasetTaskCatalog()


async def list_tasks(
    settings: Settings,
    ref: str,
    query: str | None = None,
    cursor: str | None = None,
    limit: int = 20,
) -> dict:
    offset = page_offset(cursor, limit)
    local = stored_dataset_runtime(_dataset_row(settings, ref))
    if local and local["download"]["status"] == "downloaded":
        started_at = time.perf_counter()
        page = _task_catalog.list_page(Path(local["download"]["path"]), ref, query, offset, limit)
        logger.info(
            "Loaded Dataset Task page ref=%s offset=%s limit=%s total=%s returned=%s "
            "index_cache_hit=%s elapsed_ms=%.1f",
            ref,
            offset,
            limit,
            page.total,
            len(page.items),
            page.cache_hit,
            (time.perf_counter() - started_at) * 1000,
        )
        return {
            "items": page.items,
            "nextCursor": page.next_cursor,
            "total": page.total,
        }
    elif local and local["source"] == "local":
        task_names: list[str] = []
    else:
        name, version = split_ref(ref)
        metadata = (
            await _registry_client_factory().create().get_dataset_metadata(join_ref(name, version))
        )
        task_names = [task_id.get_name() for task_id in metadata.task_ids]
    if query:
        needle = query.casefold()
        task_names = [task_name for task_name in task_names if needle in task_name.casefold()]
    selected = task_names[offset : offset + limit]
    next_cursor = str(offset + limit) if offset + limit < len(task_names) else None
    return {
        "items": [
            {
                "datasetRef": ref,
                "description": "",
                "environment": None,
                "name": task_name,
            }
            for task_name in selected
        ],
        "nextCursor": next_cursor,
        "total": len(task_names),
    }


async def get_task(settings: Settings, ref: str, task_name: str) -> dict | None:
    return await resolve_local_task(
        stored_dataset_runtime(_dataset_row(settings, ref)), ref, task_name
    )


def _dataset_row(settings: Settings, ref: str) -> dict | None:
    from ornnlab.storage import sqlite

    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn, "SELECT * FROM webui_datasets WHERE ref = ? AND deleted_at IS NULL", (ref,)
        )
    return rows[0] if rows else None


def _registry_client_factory():
    from harbor.registry.client.factory import RegistryClientFactory

    return RegistryClientFactory
