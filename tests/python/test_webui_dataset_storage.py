from __future__ import annotations

import asyncio
import shutil
from pathlib import Path
from types import SimpleNamespace

import pytest

from ornnlab.models.webui import DatasetImportInput
from ornnlab.services.webui_dataset_service import WebUiDatasetService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class FakeRegistryClient:
    async def download_dataset(
        self, _ref, *, output_dir, on_task_download_complete, on_total_known, **_
    ):
        on_total_known(1)
        task = output_dir / "task-one"
        task.mkdir()
        (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")
        on_task_download_complete("task-one", None)
        return [SimpleNamespace(downloaded_path=task)]


def test_registry_download_uses_selected_parent_and_remembers_it(tmp_path, monkeypatch):
    settings = _settings(tmp_path)
    parent = tmp_path / "chosen-parent"
    parent.mkdir()
    monkeypatch.setattr(
        "ornnlab.services.webui_dataset_service.RegistryClientFactory.create",
        lambda: FakeRegistryClient(),
    )
    service = WebUiDatasetService(settings)

    asyncio.run(service.download("team/eval@1.0", str(parent), lambda *_: None))

    destination = parent / "team--eval@1.0"
    dataset = asyncio.run(service.get_dataset("team/eval@1.0"))
    assert dataset["download"] == {
        "path": str(destination),
        "sizeBytes": pytest.approx(_directory_size(destination)),
        "status": "downloaded",
        "storageKind": "managed",
    }
    assert (destination / ".ornnlab-dataset.json").is_file()
    assert service.default_download_parent()["parentPath"] == str(parent)


def test_registry_download_rejects_existing_destination(tmp_path, monkeypatch):
    settings = _settings(tmp_path)
    parent = tmp_path / "chosen-parent"
    (parent / "team--eval@1.0").mkdir(parents=True)
    monkeypatch.setattr(
        "ornnlab.services.webui_dataset_service.RegistryClientFactory.create",
        lambda: FakeRegistryClient(),
    )

    with pytest.raises(ValueError, match="already exists"):
        asyncio.run(
            WebUiDatasetService(settings).download("team/eval@1.0", str(parent), lambda *_: None)
        )


def test_managed_dataset_can_move_to_a_new_parent(tmp_path, monkeypatch):
    settings = _settings(tmp_path)
    first_parent = tmp_path / "first"
    second_parent = tmp_path / "second"
    first_parent.mkdir()
    second_parent.mkdir()
    monkeypatch.setattr(
        "ornnlab.services.webui_dataset_service.RegistryClientFactory.create",
        lambda: FakeRegistryClient(),
    )
    service = WebUiDatasetService(settings)
    asyncio.run(service.download("team/eval@1.0", str(first_parent), lambda *_: None))

    service.move("team/eval@1.0", str(second_parent))

    destination = second_parent / "team--eval@1.0"
    assert not (first_parent / "team--eval@1.0").exists()
    assert destination.is_dir()
    assert asyncio.run(service.get_dataset("team/eval@1.0"))["download"]["path"] == str(destination)
    with pytest.raises(ValueError, match="must be deleted"):
        service.remove_registration("team/eval@1.0")


def test_external_import_is_relocated_or_unregistered_without_deleting_files(tmp_path):
    settings = _settings(tmp_path)
    original = _task_dataset(tmp_path / "original")
    replacement = _task_dataset(tmp_path / "replacement")
    service = WebUiDatasetService(settings)
    payload = DatasetImportInput(name="local/demo", path=str(original), taskCount=1, version="v1")

    asyncio.run(service.import_local(payload, lambda *_: None))
    service.relocate("local/demo@v1", str(replacement))
    assert asyncio.run(service.get_dataset("local/demo@v1"))["download"]["path"] == str(replacement)
    assert original.is_dir()
    assert replacement.is_dir()

    service.remove_registration("local/demo@v1")
    assert original.is_dir()
    assert replacement.is_dir()
    assert service._local_dataset("local/demo@v1") is None


def test_missing_path_is_exposed_without_marking_dataset_downloaded(tmp_path):
    settings = _settings(tmp_path)
    dataset = _task_dataset(tmp_path / "external")
    service = WebUiDatasetService(settings)
    asyncio.run(
        service.import_local(
            DatasetImportInput(name="local/demo", path=str(dataset), taskCount=1, version="v1"),
            lambda *_: None,
        )
    )
    shutil.rmtree(dataset)

    response = asyncio.run(service.get_dataset("local/demo@v1"))

    assert response["download"] == {
        "path": str(dataset),
        "status": "path-unavailable",
        "storageKind": "external",
    }


def _settings(tmp_path: Path) -> Settings:
    settings = Settings(home=tmp_path / "ornnlab")
    sqlite.initialize(settings)
    return settings


def _task_dataset(path: Path) -> Path:
    task = path / "task-one"
    (task / "environment").mkdir(parents=True)
    (task / "instruction.md").write_text("Solve it.\n", encoding="utf-8")
    (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")
    return path


def _directory_size(path: Path) -> int:
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())
