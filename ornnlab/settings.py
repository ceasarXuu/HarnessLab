from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path
from uuid import uuid4

from ornnlab import __version__

DEFAULT_HOME = Path("~/.ornnlab/data")
HOME_MARKER = ".ornnlab-home.json"


@dataclass(frozen=True)
class Settings:
    home: Path
    host: str = "127.0.0.1"
    port: int = 8765
    worker_max_concurrent: int = 2

    @property
    def db_path(self) -> Path:
        return self.home / "ornnlab.sqlite"

    @property
    def marker_path(self) -> Path:
        return self.home / HOME_MARKER

    @property
    def logs_dir(self) -> Path:
        return self.home / "logs"

    @property
    def experiments_dir(self) -> Path:
        return self.home / "experiments"

    @property
    def datasets_dir(self) -> Path:
        return self.home / "datasets"

    @property
    def exports_dir(self) -> Path:
        return self.home / "exports"

    @property
    def archive_dir(self) -> Path:
        return self.home / "archive"

    @classmethod
    def from_env(cls) -> Settings:
        ornnlab_home = os.environ.get("ORNNLAB_HOME")
        home = Path(ornnlab_home).expanduser() if ornnlab_home else DEFAULT_HOME.expanduser()
        max_concurrent = int(os.environ.get("ORNNLAB_WORKER_MAX_CONCURRENT", "2"))
        if max_concurrent < 1:
            raise ValueError("ORNNLAB_WORKER_MAX_CONCURRENT must be >= 1")
        return cls(home=home, worker_max_concurrent=max_concurrent)

    def ensure_dirs(self) -> None:
        for path in [
            self.home,
            self.logs_dir,
            self.experiments_dir,
            self.datasets_dir,
            self.exports_dir,
            self.archive_dir,
        ]:
            path.mkdir(parents=True, exist_ok=True)
        marker = self._home_marker()
        original = dict(marker)
        if "instance_id" not in marker:
            marker["instance_id"] = uuid4().hex
        marker.update(
            {
                "schema_version": 2,
                "product": "OrnnLab",
                "home": str(self.home),
                "version": __version__,
            }
        )
        if marker != original:
            self.marker_path.write_text(
                json.dumps(marker, indent=2, sort_keys=True),
                encoding="utf-8",
            )

    @property
    def instance_id(self) -> str:
        self.ensure_dirs()
        value = self._home_marker().get("instance_id")
        if not isinstance(value, str) or not value:
            raise RuntimeError("OrnnLab home marker is missing instance_id")
        return value

    def _home_marker(self) -> dict[str, object]:
        if not self.marker_path.exists():
            return {}
        try:
            payload = json.loads(self.marker_path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as error:
            raise RuntimeError(f"Invalid OrnnLab home marker: {self.marker_path}") from error
        if not isinstance(payload, dict):
            raise RuntimeError(f"Invalid OrnnLab home marker: {self.marker_path}")
        return payload
