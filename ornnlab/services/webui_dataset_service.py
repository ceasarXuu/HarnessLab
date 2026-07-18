from __future__ import annotations

import asyncio
import json
import logging
import os
import re
import shutil
import time
from pathlib import Path

from ornnlab.models.webui import DatasetImportInput
from ornnlab.services.clock import now_iso
from ornnlab.services.container_image_platforms import resolve_local_task
from ornnlab.services.dataset_download_state import (
    merge_active_downloads,
    remote_dataset_dto,
    stored_dataset_dto,
    stored_dataset_runtime,
)
from ornnlab.services.dataset_environment import parse_local_tasks
from ornnlab.services.dataset_task_catalog import LocalDatasetTaskCatalog, page_offset
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)
_MARKER_FILE = ".ornnlab-dataset.json"
_LAST_PARENT_KEY = "last_dataset_parent_path"
_REGISTRY_CACHE_TTL_SECONDS = 60


class WebUiDatasetService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self._registry_cache: list[dict] | None = None
        self._registry_cache_lock = asyncio.Lock()
        self._registry_cache_updated_at = 0.0
        self._task_catalog = LocalDatasetTaskCatalog()

    async def list_datasets(self, query: str | None = None) -> list[dict]:
        local = self._local_datasets()
        try:
            remote = await self._remote_datasets()
        except Exception:
            if not local:
                raise
            logger.warning("Harbor registry is unavailable; returning local datasets only")
            remote = []
        records = {item["ref"]: item for item in remote}
        records.update({item["ref"]: item for item in local})
        result = list(records.values())
        if query:
            needle = query.lower()
            result = [
                item
                for item in result
                if needle in " ".join(str(value) for value in item.values()).lower()
            ]
        return merge_active_downloads(
            self.settings, sorted(result, key=lambda item: item["ref"].lower())
        )

    async def get_dataset(self, ref: str) -> dict:
        local = self._local_dataset(ref)
        if local:
            return merge_active_downloads(self.settings, [local])[0]
        name, version = _split_ref(ref)
        metadata = await _registry_client_factory().create().get_dataset_metadata(
            _join_ref(name, version)
        )
        remote = remote_dataset_dto(metadata.name, metadata.version, len(metadata.task_ids))
        return merge_active_downloads(self.settings, [remote])[0]

    def default_download_parent(self) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT value FROM webui_dataset_preferences WHERE key = ?",
                (_LAST_PARENT_KEY,),
            )
        return {"parentPath": rows[0]["value"] if rows else str(self.settings.datasets_dir)}

    async def list_tasks(
        self,
        ref: str,
        query: str | None = None,
        cursor: str | None = None,
        limit: int = 20,
    ) -> dict:
        offset = page_offset(cursor, limit)
        local = stored_dataset_runtime(self._dataset_row(ref))
        if local and local["download"]["status"] == "downloaded":
            started_at = time.perf_counter()
            page = self._task_catalog.list_page(
                Path(local["download"]["path"]), ref, query, offset, limit
            )
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
            name, version = _split_ref(ref)
            metadata = await _registry_client_factory().create().get_dataset_metadata(
                _join_ref(name, version)
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

    async def get_task(self, ref: str, task_name: str) -> dict | None:
        return await resolve_local_task(
            stored_dataset_runtime(self._dataset_row(ref)), ref, task_name
        )

    async def download(self, ref: str, parent_path: str, progress) -> None:
        if self._dataset_row(ref):
            raise ValueError(
                "dataset is already registered; relocate it or remove its registration first"
            )
        name, version = _split_ref(ref)
        parent = _require_parent_directory(parent_path)
        destination = parent / _managed_directory_name(ref)
        if destination.exists():
            raise ValueError(f"dataset destination already exists: {destination}")

        self._set_last_parent(parent)
        destination.mkdir()
        _write_marker(destination, ref)
        progress(5, "Preparing dataset directory")
        self._record_pending_download(ref, parent, destination)
        progress(10, "Starting dataset download")
        logger.info("Preparing Dataset download ref=%s destination=%s", ref, destination)
        total = 0
        completed = 0

        def on_total_known(value: int) -> None:
            nonlocal total
            total = value
            progress(20, f"Downloading {value} tasks")

        def on_complete(_task_id, _result) -> None:
            nonlocal completed
            completed += 1
            percentage = 20 + int(completed * 75 / total) if total else None
            progress(percentage, f"Downloaded {completed} of {total} tasks")

        try:
            items = await _registry_client_factory().create().download_dataset(
                _join_ref(name, version),
                output_dir=destination,
                export=True,
                on_total_known=on_total_known,
                on_task_download_complete=on_complete,
            )
            self._upsert_dataset(
                ref=ref,
                name=name,
                version=version or "latest",
                source="harbor registry",
                visibility="public",
                registry_url="https://hub.harborframework.com",
                local_path=str(destination),
                storage_kind="managed",
                task_count=len(items),
            )
            logger.info(
                "Downloaded Dataset ref=%s destination=%s tasks=%s", ref, destination, len(items)
            )
        except BaseException:
            _remove_marked_directory(destination, ref, allow_legacy=False)
            raise
        finally:
            self._clear_pending_download(ref)

    def cancel_download(self, ref: str) -> None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn, "SELECT destination_path FROM webui_dataset_downloads WHERE ref = ?", (ref,)
            )
        if not rows:
            raise ValueError("dataset download is already complete")
        destination = Path(rows[0]["destination_path"])
        _remove_marked_directory(destination, ref, allow_legacy=False)
        self._clear_pending_download(ref)
        logger.info("Cancelled Dataset download ref=%s destination=%s", ref, destination)

    def move(self, ref: str, parent_path: str) -> None:
        row = self._require_dataset_row(ref)
        if row.get("storage_kind") != "managed":
            raise ValueError("only OrnnLab-managed datasets can be moved")
        source = _require_existing_directory(row.get("local_path"))
        _assert_managed_directory(source, ref, self.settings.datasets_dir)
        parent = _require_parent_directory(parent_path)
        destination = parent / _managed_directory_name(ref)
        if destination.exists():
            raise ValueError(f"dataset destination already exists: {destination}")
        shutil.move(str(source), str(destination))
        _write_marker(destination, ref)
        self._update_path(ref, destination)
        self._set_last_parent(parent)
        logger.info("Moved Dataset ref=%s source=%s destination=%s", ref, source, destination)

    def relocate(self, ref: str, path_value: str) -> None:
        row = self._require_dataset_row(ref)
        path = _require_existing_directory(path_value)
        if row.get("storage_kind") == "managed":
            _assert_managed_directory(path, ref, self.settings.datasets_dir, allow_legacy=False)
        elif not _local_tasks(path, ref):
            raise ValueError("external Dataset directory contains no valid Harbor tasks")
        self._update_path(ref, path)
        logger.info("Relocated Dataset registration ref=%s path=%s", ref, path)

    def delete_local(self, ref: str) -> None:
        row = self._require_dataset_row(ref)
        if row.get("storage_kind") != "managed":
            raise ValueError("external Dataset files cannot be deleted by OrnnLab")
        path = _require_existing_directory(row.get("local_path"))
        _assert_managed_directory(path, ref, self.settings.datasets_dir)
        _remove_marked_directory(
            path, ref, allow_legacy=True, legacy_root=self.settings.datasets_dir
        )
        self.remove_registration(ref)
        logger.info("Deleted managed Dataset ref=%s path=%s", ref, path)

    def remove_registration(self, ref: str) -> None:
        row = self._require_dataset_row(ref)
        local_path = row.get("local_path")
        if row.get("storage_kind") == "managed" and local_path and Path(local_path).is_dir():
            raise ValueError("managed Dataset must be deleted before removing its registration")
        with sqlite.connect(self.settings) as conn:
            conn.execute("DELETE FROM webui_datasets WHERE ref = ?", (ref,))
        logger.info("Removed Dataset registration ref=%s", ref)

    async def import_local(self, payload: DatasetImportInput, progress) -> None:
        source_path = _require_existing_directory(payload.path)
        tasks = _local_tasks(source_path, _join_ref(payload.name, payload.version))
        if not tasks:
            raise ValueError("dataset directory contains no valid Harbor task directories")
        if payload.task_count not in {0, len(tasks)}:
            raise ValueError("taskCount must match the discovered Harbor task count")
        progress(75, "Registering local dataset")
        self._upsert_dataset(
            ref=_join_ref(payload.name, payload.version),
            name=payload.name,
            version=payload.version,
            source="local",
            visibility="private",
            registry_url=None,
            local_path=str(source_path),
            storage_kind="external",
            task_count=len(tasks),
        )
        logger.info(
            "Registered external Dataset ref=%s path=%s",
            _join_ref(payload.name, payload.version),
            source_path,
        )

    async def sync(self, ref: str, progress) -> None:
        dataset = self._local_dataset(ref)
        if not dataset or dataset["download"]["status"] != "downloaded":
            raise KeyError(ref)
        path = Path(dataset["download"]["path"])
        manifest = path / "dataset.toml"
        if not manifest.is_file():
            raise ValueError("sync requires a local dataset.toml manifest")
        progress(25, "Updating dataset manifest")
        from harbor.cli.sync import sync_dataset

        await asyncio.to_thread(sync_dataset, path)
        progress(90, "Manifest updated")

    def _local_datasets(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT * FROM webui_datasets WHERE deleted_at IS NULL")
        return [stored_dataset_dto(row) for row in rows]

    def _local_dataset(self, ref: str) -> dict | None:
        row = self._dataset_row(ref)
        return stored_dataset_dto(row) if row else None

    def _dataset_row(self, ref: str) -> dict | None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn, "SELECT * FROM webui_datasets WHERE ref = ? AND deleted_at IS NULL", (ref,)
            )
        return rows[0] if rows else None

    def _require_dataset_row(self, ref: str) -> dict:
        row = self._dataset_row(ref)
        if not row:
            raise KeyError(ref)
        return row

    async def _remote_datasets(self) -> list[dict]:
        if self._registry_cache_is_fresh():
            logger.debug("Using cached Harbor registry Dataset catalog")
            return list(self._registry_cache or [])

        async with self._registry_cache_lock:
            if self._registry_cache_is_fresh():
                logger.debug("Using cached Harbor registry Dataset catalog after lock")
                return list(self._registry_cache or [])
            summaries = await _registry_client_factory().create().list_datasets()
            self._registry_cache = [
                remote_dataset_dto(item.name, item.version, item.task_count)
                for item in summaries
            ]
            self._registry_cache_updated_at = time.monotonic()
            logger.info(
                "Refreshed Harbor registry Dataset catalog count=%s", len(self._registry_cache)
            )
            return list(self._registry_cache)

    def _registry_cache_is_fresh(self) -> bool:
        return (
            self._registry_cache is not None
            and time.monotonic() - self._registry_cache_updated_at < _REGISTRY_CACHE_TTL_SECONDS
        )

    def _set_last_parent(self, parent: Path) -> None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                (
                    "INSERT OR REPLACE INTO webui_dataset_preferences"
                    "(key, value, updated_at) VALUES (?, ?, ?)"
                ),
                (_LAST_PARENT_KEY, str(parent), now),
            )

    def _record_pending_download(self, ref: str, parent: Path, destination: Path) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                (
                    "INSERT OR REPLACE INTO webui_dataset_downloads"
                    "(ref, destination_path, parent_path, created_at) VALUES (?, ?, ?, ?)"
                ),
                (ref, str(destination), str(parent), now_iso()),
            )

    def _clear_pending_download(self, ref: str) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute("DELETE FROM webui_dataset_downloads WHERE ref = ?", (ref,))

    def _update_path(self, ref: str, path: Path) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE webui_datasets SET local_path = ?, updated_at = ? WHERE ref = ?",
                (str(path), now_iso(), ref),
            )

    def _upsert_dataset(
        self,
        *,
        ref: str,
        name: str,
        version: str,
        source: str,
        visibility: str,
        registry_url: str | None,
        local_path: str,
        storage_kind: str,
        task_count: int,
    ) -> None:
        now = now_iso()
        query = """
            INSERT OR REPLACE INTO webui_datasets(
              ref, name, version, source, visibility, registry_url, local_path, storage_kind,
              task_count, created_at, updated_at, deleted_at
            ) VALUES (
              ?, ?, ?, ?, ?, ?, ?, ?, ?,
              COALESCE((SELECT created_at FROM webui_datasets WHERE ref = ?), ?), ?, NULL
            )
        """
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                query,
                (
                    ref,
                    name,
                    version,
                    source,
                    visibility,
                    registry_url,
                    local_path,
                    storage_kind,
                    task_count,
                    ref,
                    now,
                    now,
                ),
            )


def _local_tasks(path: Path, ref: str) -> list[dict]:
    return parse_local_tasks(path, ref)


def _registry_client_factory():
    from harbor.registry.client.factory import RegistryClientFactory

    return RegistryClientFactory


def _require_parent_directory(value: str) -> Path:
    path = Path(value).expanduser().resolve()
    if not path.is_dir():
        raise ValueError("dataset parent directory must exist")
    if not os.access(path, os.W_OK | os.X_OK):
        raise ValueError("dataset parent directory is not writable")
    return path


def _require_existing_directory(value: str | None) -> Path:
    if not value:
        raise ValueError("dataset path is not available")
    path = Path(value).expanduser().resolve()
    if not path.is_dir():
        raise ValueError("dataset path must be an existing directory")
    return path


def _managed_directory_name(ref: str) -> str:
    name = re.sub(r"[^A-Za-z0-9._@-]+", "-", ref.replace("/", "--")).strip(".-")
    if not name:
        raise ValueError("dataset reference cannot produce a directory name")
    return name


def _write_marker(path: Path, ref: str) -> None:
    (path / _MARKER_FILE).write_text(json.dumps({"ref": ref}, sort_keys=True), encoding="utf-8")


def _assert_managed_directory(
    path: Path, ref: str, legacy_root: Path, *, allow_legacy: bool = True
) -> None:
    marker = path / _MARKER_FILE
    if marker.is_file():
        try:
            if json.loads(marker.read_text(encoding="utf-8")).get("ref") == ref:
                return
        except json.JSONDecodeError:
            pass
    if allow_legacy and path.parent.resolve() == legacy_root.resolve():
        return
    raise ValueError("dataset directory is not managed by OrnnLab")


def _remove_marked_directory(
    path: Path, ref: str, *, allow_legacy: bool, legacy_root: Path | None = None
) -> None:
    if not path.exists():
        return
    root = legacy_root or path.parent
    _assert_managed_directory(path, ref, root, allow_legacy=allow_legacy)
    shutil.rmtree(path)




def _split_ref(ref: str) -> tuple[str, str | None]:
    name, separator, version = ref.rpartition("@")
    return (name, version) if separator else (ref, None)


def _join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name
