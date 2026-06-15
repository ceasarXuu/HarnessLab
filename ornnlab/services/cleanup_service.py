from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any

from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class CleanupService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def plan(self) -> dict[str, Any]:
        sqlite.initialize(self.settings)
        candidates = [
            *self._stale_generated_agents(),
            *self._orphan_experiment_artifacts(),
        ]
        return {
            "candidate_count": len(candidates),
            "candidates": candidates,
            "mode": "archive-only",
        }

    def archive(self) -> dict[str, Any]:
        plan = self.plan()
        destination_root = self._destination_root()
        archived: list[dict[str, Any]] = []
        for candidate in plan["candidates"]:
            source = Path(candidate["path"])
            if not source.exists():
                continue
            destination = destination_root / candidate["type"] / source.name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(source), str(destination))
            archived.append({**candidate, "archived_path": str(destination)})
        return {
            "archived_count": len(archived),
            "archive_dir": str(destination_root),
            "archived": archived,
        }

    def _stale_generated_agents(self) -> list[dict[str, Any]]:
        self.settings.generated_agents_dir.mkdir(parents=True, exist_ok=True)
        with sqlite.connect(self.settings) as conn:
            active_ids = {
                row["id"]
                for row in sqlite.rows(conn, "SELECT id FROM agents WHERE status != 'deleted'")
            }
        candidates: list[dict[str, Any]] = []
        for path in sorted(self.settings.generated_agents_dir.iterdir()):
            if not path.is_dir() or path.name in active_ids:
                continue
            candidates.append(
                {
                    "type": "generated-agent",
                    "path": str(path),
                    "reason": "no_active_agent_row",
                    "recoverable": True,
                }
            )
        return candidates

    def _orphan_experiment_artifacts(self) -> list[dict[str, Any]]:
        self.settings.experiments_dir.mkdir(parents=True, exist_ok=True)
        with sqlite.connect(self.settings) as conn:
            referenced = {
                row["id"] for row in sqlite.rows(conn, "SELECT id FROM experiments")
            }
            referenced.update(row["id"] for row in sqlite.rows(conn, "SELECT id FROM runs"))
        candidates: list[dict[str, Any]] = []
        for path in sorted(self.settings.experiments_dir.iterdir()):
            if not path.is_dir() or path.name in referenced:
                continue
            candidates.append(
                {
                    "type": "experiment-artifact",
                    "path": str(path),
                    "reason": "no_experiment_or_run_row",
                    "recoverable": True,
                }
            )
        return candidates

    def _destination_root(self) -> Path:
        stamp = now_iso().replace(":", "").replace("-", "")
        return self.settings.archive_dir / f"cleanup-{stamp}"
