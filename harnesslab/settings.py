from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Settings:
    home: Path
    host: str = "127.0.0.1"
    port: int = 8765

    @property
    def db_path(self) -> Path:
        return self.home / "harnesslab.sqlite"

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

    @classmethod
    def from_env(cls) -> Settings:
        home = Path(os.environ.get("HARNESSLAB_HOME", "~/.harnesslab")).expanduser()
        return cls(home=home)

    def ensure_dirs(self) -> None:
        for path in [
            self.home,
            self.logs_dir,
            self.agents_dir,
            self.generated_agents_dir,
            self.experiments_dir,
        ]:
            path.mkdir(parents=True, exist_ok=True)
