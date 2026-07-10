from __future__ import annotations

import asyncio
import logging
import os
import shutil
from pathlib import Path

from harbor.registry.client.factory import RegistryClientFactory

from ornnlab.models.webui import DatasetImportInput
from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


class WebUiDatasetService:
    def __init__(self, settings: Settings):
        self.settings = settings

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
        return sorted(result, key=lambda item: item["ref"].lower())

    async def get_dataset(self, ref: str) -> dict:
        local = self._local_dataset(ref)
        if local:
            return local
        name, version = _split_ref(ref)
        metadata = await RegistryClientFactory.create().get_dataset_metadata(
            _join_ref(name, version)
        )
        return _remote_dto(metadata.name, metadata.version, len(metadata.task_ids))

    async def list_tasks(self, ref: str, query: str | None = None) -> list[dict]:
        local = self._local_dataset(ref)
        if local and local["source"] == "local":
            tasks = _local_tasks(Path(local["download"].get("path", "")), ref)
        else:
            name, version = _split_ref(ref)
            metadata = await RegistryClientFactory.create().get_dataset_metadata(
                _join_ref(name, version)
            )
            tasks = [
                {"datasetRef": ref, "description": "", "name": task_id.get_name()}
                for task_id in metadata.task_ids
            ]
        if query:
            needle = query.lower()
            tasks = [
                item for item in tasks if needle in f"{item['name']} {item['description']}".lower()
            ]
        return tasks

    async def download(self, ref: str, progress) -> None:
        existing = self._local_dataset(ref)
        if existing and existing["download"]["status"] == "downloaded":
            raise ValueError("dataset is already downloaded")
        name, version = _split_ref(ref)
        client = RegistryClientFactory.create()
        output_dir = self.settings.datasets_dir
        total = 0
        completed = 0

        def on_total_known(value: int) -> None:
            nonlocal total
            total = value
            progress(0, f"Downloading {value} tasks")

        def on_complete(_task_id, _result) -> None:
            nonlocal completed
            completed += 1
            percentage = int(completed * 100 / total) if total else None
            progress(percentage, f"Downloaded {completed} of {total} tasks")

        items = await client.download_dataset(
            _join_ref(name, version),
            output_dir=output_dir,
            export=True,
            on_total_known=on_total_known,
            on_task_download_complete=on_complete,
        )
        downloaded_path = _downloaded_path(items, output_dir, name)
        self._upsert_dataset(
            ref=ref,
            name=name,
            version=version or "latest",
            source="harbor registry",
            visibility="public",
            registry_url="https://hub.harborframework.com",
            local_path=str(downloaded_path),
            task_count=len(items),
        )

    def cancel_download(self, ref: str) -> None:
        dataset = self._local_dataset(ref)
        if dataset:
            raise ValueError("dataset download is already complete")
        name, _version = _split_ref(ref)
        partial = self.settings.datasets_dir / name.split("/")[-1]
        if partial.exists():
            shutil.rmtree(partial)

    def delete_local(self, ref: str) -> None:
        dataset = self._local_dataset(ref)
        if not dataset or dataset["download"]["status"] != "downloaded":
            raise KeyError(ref)
        self._remove_managed_files(dataset)
        with sqlite.connect(self.settings) as conn:
            conn.execute("DELETE FROM webui_datasets WHERE ref = ?", (ref,))

    async def import_local(self, payload: DatasetImportInput, progress) -> None:
        source_path = Path(payload.path).expanduser().resolve()
        if not source_path.is_dir():
            raise ValueError("dataset path must be an existing directory")
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
            task_count=len(tasks),
        )

    async def sync(self, ref: str, progress) -> None:
        dataset = self._local_dataset(ref)
        if not dataset:
            raise KeyError(ref)
        path = Path(dataset["download"].get("path", ""))
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
        return [_stored_dto(row) for row in rows]

    def _local_dataset(self, ref: str) -> dict | None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT * FROM webui_datasets WHERE ref = ? AND deleted_at IS NULL",
                (ref,),
            )
        return _stored_dto(rows[0]) if rows else None

    async def _remote_datasets(self) -> list[dict]:
        summaries = await RegistryClientFactory.create().list_datasets()
        return [_remote_dto(item.name, item.version, item.task_count) for item in summaries]

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
        task_count: int,
    ) -> None:
        now = now_iso()
        query = """
            INSERT OR REPLACE INTO webui_datasets(
              ref, name, version, source, visibility, registry_url, local_path,
              task_count, created_at, updated_at, deleted_at
            ) VALUES (
              ?, ?, ?, ?, ?, ?, ?, ?,
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
                    task_count,
                    ref,
                    now,
                    now,
                ),
            )

    def _remove_managed_files(self, dataset: dict) -> None:
        path_value = dataset["download"].get("path")
        if not path_value:
            return
        path = Path(path_value).resolve()
        managed_root = self.settings.datasets_dir.resolve()
        if path.is_relative_to(managed_root) and path.exists():
            shutil.rmtree(path)


def _remote_dto(name: str, version: str | None, task_count: int) -> dict:
    resolved_version = version or "latest"
    return {
        "ref": _join_ref(name, resolved_version),
        "name": name,
        "version": resolved_version,
        "visibility": "public",
        "taskCount": task_count,
        "source": "harbor registry",
        "download": {"status": "not-downloaded"},
        "registryUrl": "https://hub.harborframework.com",
    }


def _stored_dto(row: dict) -> dict:
    path = row.get("local_path")
    downloaded = path is not None and Path(path).exists()
    download = (
        {"status": "downloaded", "path": path, "sizeBytes": _directory_size(Path(path))}
        if downloaded
        else {"status": "not-downloaded"}
    )
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


def _local_tasks(path: Path, ref: str) -> list[dict]:
    if not path.is_dir():
        return []
    from harbor.models.task.task import Task

    tasks = []
    for child in sorted(path.iterdir()):
        if child.is_dir() and Task.is_valid_dir(child, disable_verification=True):
            tasks.append({"datasetRef": ref, "description": "", "name": child.name})
    return tasks


def _downloaded_path(items: list, output_dir: Path, name: str) -> Path:
    expected = output_dir / name.split("/")[-1]
    if expected.exists():
        return expected
    paths = [Path(item.downloaded_path) for item in items]
    if not paths:
        raise ValueError("Harbor completed download without any tasks")
    return Path(os.path.commonpath(paths))


def _directory_size(path: Path) -> int:
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def _split_ref(ref: str) -> tuple[str, str | None]:
    name, separator, version = ref.rpartition("@")
    return (name, version) if separator else (ref, None)


def _join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name
