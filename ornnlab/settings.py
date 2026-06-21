from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path

from ornnlab import __version__

DEFAULT_HOME = Path("~/.ornnlab/data")
HOME_MARKER = ".ornnlab-home.json"


@dataclass(frozen=True)
class Settings:
    home: Path
    host: str = "127.0.0.1"
    port: int = 8765

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
        home = Path(ornnlab_home).expanduser() if ornnlab_home else DEFAULT_HOME.expanduser()
        return cls(home=home)

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
