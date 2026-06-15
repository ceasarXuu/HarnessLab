from __future__ import annotations

import json
import os
import shutil
from dataclasses import dataclass
from pathlib import Path

from ornnlab import __version__

DEFAULT_HOME = Path("~/.ornnlab/data")
LEGACY_HOME = Path("~/.harnesslab")
HOME_MARKER = ".ornnlab-home.json"
MIGRATION_MANIFEST = "ornnlab-home-migration.json"


@dataclass(frozen=True)
class Settings:
    home: Path
    host: str = "127.0.0.1"
    port: int = 8765
    warnings: tuple[str, ...] = ()
    migration: dict[str, str | int | bool | None] | None = None

    @property
    def db_path(self) -> Path:
        return self.home / "ornnlab.sqlite"

    @property
    def legacy_db_path(self) -> Path:
        return self.home / "harnesslab.sqlite"

    @property
    def marker_path(self) -> Path:
        return self.home / HOME_MARKER

    @property
    def logs_dir(self) -> Path:
        return self.home / "logs"

    @property
    def agents_dir(self) -> Path:
        return self.home / "agents"

    @property
    def generated_agents_dir(self) -> Path:
        return self.home / "generated-agents"

    @property
    def experiments_dir(self) -> Path:
        return self.home / "experiments"

    @property
    def exports_dir(self) -> Path:
        return self.home / "exports"

    @property
    def archive_dir(self) -> Path:
        return self.home / "archive"

    @classmethod
    def from_env(cls) -> Settings:
        ornnlab_home = os.environ.get("ORNNLAB_HOME")
        legacy_home = os.environ.get("HARNESSLAB_HOME")
        warnings: list[str] = []
        migration: dict[str, str | int | bool | None] | None = None

        if ornnlab_home:
            home = Path(ornnlab_home).expanduser()
            if legacy_home:
                warnings.append("legacy_home_ignored")
        elif legacy_home:
            home = Path(legacy_home).expanduser()
            warnings.extend(["legacy_env_in_use", "using_legacy_home"])
        else:
            home = DEFAULT_HOME.expanduser()
            legacy_default_home = LEGACY_HOME.expanduser()
            if not (home / HOME_MARKER).exists() and legacy_default_home.exists():
                migration = _migrate_legacy_home(legacy_default_home, home)
                warnings.append("migrated_home" if migration.get("ok") else "migration_error")

        return cls(home=home, warnings=tuple(warnings), migration=migration)

    def ensure_dirs(self) -> None:
        for path in [
            self.home,
            self.logs_dir,
            self.agents_dir,
            self.generated_agents_dir,
            self.experiments_dir,
            self.exports_dir,
            self.archive_dir,
        ]:
            path.mkdir(parents=True, exist_ok=True)
        if self.legacy_db_path.exists() and not self.db_path.exists():
            shutil.copy2(self.legacy_db_path, self.db_path)
        if not self.marker_path.exists():
            self.marker_path.write_text(
                json.dumps(
                    {
                        "schema_version": 1,
                        "product": "OrnnLab",
                        "home": str(self.home),
                        "version": __version__,
                    },
                    indent=2,
                    sort_keys=True,
                ),
                encoding="utf-8",
            )


def _migrate_legacy_home(source: Path, target: Path) -> dict[str, str | int | bool | None]:
    manifest_dir = target / "migration"
    manifest_path = manifest_dir / MIGRATION_MANIFEST
    file_count = sum(1 for path in source.rglob("*") if path.is_file() and not path.is_symlink())
    payload: dict[str, str | int | bool | None] = {
        "schema_version": 1,
        "source_home": str(source),
        "target_home": str(target),
        "version": __version__,
        "source_file_count": file_count,
        "ok": False,
        "error": None,
    }
    try:
        target.mkdir(parents=True, exist_ok=True)
        shutil.copytree(source, target, dirs_exist_ok=True, symlinks=False)
        legacy_db = target / "harnesslab.sqlite"
        new_db = target / "ornnlab.sqlite"
        if legacy_db.exists() and not new_db.exists():
            shutil.copy2(legacy_db, new_db)
        payload["ok"] = True
    except Exception as error:  # pragma: no cover - defensive migration reporting
        payload["error"] = str(error)
    finally:
        manifest_dir.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    return payload
