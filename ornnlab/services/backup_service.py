from __future__ import annotations

import io
import json
import tarfile
from pathlib import Path
from typing import Any

from ornnlab import __version__
from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite
from ornnlab.storage.paths import ensure_parent

MANIFEST_NAME = "ornnlab-backup-manifest.json"
LEGACY_MANIFEST_NAME = "harnesslab-backup-manifest.json"
BACKUP_PREFIX = "ornnlab-backup"


class BackupService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def export_home(self, output: Path | None = None) -> dict[str, Any]:
        self.settings.ensure_dirs()
        _checkpoint_sqlite(self.settings)
        archive_path = output or self._default_output_path()
        ensure_parent(archive_path)
        files = self._backup_files(archive_path)
        manifest = {
            "schema_version": 1,
            "ornnlab_version": __version__,
            "created_at": now_iso(),
            "source_home": str(self.settings.home),
            "file_count": len(files),
        }
        with tarfile.open(archive_path, "w:gz") as archive:
            _add_json(archive, MANIFEST_NAME, manifest)
            for path in files:
                archive.add(path, arcname=path.relative_to(self.settings.home).as_posix())
        return {
            "archive_path": str(archive_path),
            "file_count": len(files),
            "manifest": manifest,
        }

    def import_home(self, archive_path: Path) -> dict[str, Any]:
        if self.settings.home.exists() and any(self.settings.home.iterdir()):
            raise ValueError("target_home_not_empty")
        self.settings.home.mkdir(parents=True, exist_ok=True)
        restored: list[str] = []
        manifest: dict[str, Any] | None = None
        with tarfile.open(archive_path, "r:gz") as archive:
            for member in archive.getmembers():
                _validate_member(member)
                if member.name in {MANIFEST_NAME, LEGACY_MANIFEST_NAME}:
                    parsed_manifest = json.loads(_read_member(archive, member).decode("utf-8"))
                    if (
                        "ornnlab_version" not in parsed_manifest
                        and "harnesslab_version" in parsed_manifest
                    ):
                        parsed_manifest["ornnlab_version"] = parsed_manifest[
                            "harnesslab_version"
                        ]
                    manifest = parsed_manifest
                    continue
                target = (self.settings.home / member.name).resolve()
                target.relative_to(self.settings.home.resolve())
                if member.isdir():
                    target.mkdir(parents=True, exist_ok=True)
                    continue
                if not member.isfile():
                    continue
                ensure_parent(target)
                target.write_bytes(_read_member(archive, member))
                restored.append(member.name)
        self.settings.ensure_dirs()
        return {
            "archive_path": str(archive_path),
            "target_home": str(self.settings.home),
            "file_count": len(restored),
            "manifest": manifest,
        }

    def _default_output_path(self) -> Path:
        stamp = now_iso().replace(":", "").replace("-", "")
        return self.settings.exports_dir / f"{BACKUP_PREFIX}-{stamp}.tar.gz"

    def _backup_files(self, archive_path: Path) -> list[Path]:
        home = self.settings.home.resolve()
        archive_resolved = archive_path.resolve()
        files: list[Path] = []
        for path in sorted(home.rglob("*")):
            resolved = path.resolve()
            if path.is_symlink() or not path.is_file():
                continue
            if resolved == archive_resolved:
                continue
            if _is_under(resolved, self.settings.exports_dir.resolve()):
                continue
            if path.name in {
                "ornnlab.sqlite-wal",
                "ornnlab.sqlite-shm",
                "harnesslab.sqlite-wal",
                "harnesslab.sqlite-shm",
            }:
                continue
            files.append(path)
        return files


def _checkpoint_sqlite(settings: Settings) -> None:
    if not settings.db_path.exists():
        return
    with sqlite.connect(settings) as conn:
        conn.execute("PRAGMA wal_checkpoint(FULL)")


def _add_json(archive: tarfile.TarFile, name: str, payload: dict[str, Any]) -> None:
    data = json.dumps(payload, indent=2, sort_keys=True).encode("utf-8")
    info = tarfile.TarInfo(name)
    info.size = len(data)
    archive.addfile(info, io.BytesIO(data))


def _validate_member(member: tarfile.TarInfo) -> None:
    path = Path(member.name)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"unsafe_backup_member:{member.name}")
    if member.issym() or member.islnk() or member.isdev():
        raise ValueError(f"unsupported_backup_member:{member.name}")


def _read_member(archive: tarfile.TarFile, member: tarfile.TarInfo) -> bytes:
    handle = archive.extractfile(member)
    if handle is None:
        return b""
    return handle.read()


def _is_under(path: Path, parent: Path) -> bool:
    try:
        path.relative_to(parent)
    except ValueError:
        return False
    return True
