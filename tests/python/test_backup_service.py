from __future__ import annotations

import io
import json
import tarfile
from pathlib import Path

import pytest

from ornnlab.services.backup_service import MANIFEST_NAME, BackupService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def test_backup_export_excludes_exports_and_restores_into_empty_home(settings, tmp_path):
    sqlite.initialize(settings)
    (settings.logs_dir / "ornnlab.log").write_text("started\n", encoding="utf-8")
    (settings.exports_dir / "old-backup.tar.gz").write_text("skip me", encoding="utf-8")
    archive_path = settings.exports_dir / "backup.tar.gz"

    exported = BackupService(settings).export_home(archive_path)

    assert exported["archive_path"] == str(archive_path)
    with tarfile.open(archive_path, "r:gz") as archive:
        names = archive.getnames()
        manifest_handle = archive.extractfile(MANIFEST_NAME)
        assert manifest_handle is not None
        manifest = json.loads(manifest_handle.read().decode("utf-8"))
    assert MANIFEST_NAME in names
    assert "logs/ornnlab.log" in names
    assert "exports/old-backup.tar.gz" not in names
    assert manifest["schema_version"] == 1

    restored_home = tmp_path / "restored"
    imported = BackupService(Settings(home=restored_home)).import_home(archive_path)

    assert imported["file_count"] == exported["file_count"]
    assert (restored_home / "logs" / "ornnlab.log").read_text(encoding="utf-8") == "started\n"
    assert (restored_home / "ornnlab.sqlite").exists()


def test_backup_import_rejects_non_empty_target(settings, tmp_path):
    archive_path = BackupService(settings).export_home(settings.exports_dir / "backup.tar.gz")
    target = tmp_path / "target"
    target.mkdir()
    (target / "existing.txt").write_text("keep", encoding="utf-8")

    with pytest.raises(ValueError, match="target_home_not_empty"):
        BackupService(Settings(home=target)).import_home(Path(archive_path["archive_path"]))

    assert (target / "existing.txt").read_text(encoding="utf-8") == "keep"


def test_backup_import_rejects_path_traversal(tmp_path):
    archive_path = tmp_path / "bad.tar.gz"
    payload = b"bad"
    with tarfile.open(archive_path, "w:gz") as archive:
        info = tarfile.TarInfo("../escape.txt")
        info.size = len(payload)
        archive.addfile(info, io.BytesIO(payload))

    with pytest.raises(ValueError, match="unsafe_backup_member"):
        BackupService(Settings(home=tmp_path / "target")).import_home(archive_path)

    assert not (tmp_path / "escape.txt").exists()
