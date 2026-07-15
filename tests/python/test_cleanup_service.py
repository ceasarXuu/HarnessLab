from __future__ import annotations

from pathlib import Path

from ornnlab.services.cleanup_service import CleanupService
from ornnlab.services.clock import now_iso
from ornnlab.storage import sqlite


def test_cleanup_plan_finds_only_unreferenced_artifacts(settings):
    _seed_references(settings)
    (settings.experiments_dir / "exp-active").mkdir(parents=True)
    (settings.experiments_dir / "run-active").mkdir()
    (settings.experiments_dir / "orphan-artifact").mkdir()

    plan = CleanupService(settings).plan()

    assert plan["mode"] == "archive-only"
    assert {(item["type"], item["reason"]) for item in plan["candidates"]} == {
        ("experiment-artifact", "no_experiment_or_run_row"),
    }
    assert {Path(item["path"]).name for item in plan["candidates"]} == {
        "orphan-artifact",
    }


def test_cleanup_archive_moves_candidates_to_recoverable_archive(settings):
    _seed_references(settings)
    orphan = settings.experiments_dir / "orphan-artifact"
    orphan.mkdir(parents=True)
    (orphan / "result.json").write_text("{}", encoding="utf-8")

    result = CleanupService(settings).archive()

    assert result["archived_count"] == 1
    assert not orphan.exists()
    archived_names = {Path(item["archived_path"]).name for item in result["archived"]}
    assert archived_names == {"orphan-artifact"}


def _seed_references(settings) -> None:
    sqlite.initialize(settings)
    now = now_iso()
    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO experiments VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            ("exp-active", "Active", "single", "completed", 1, "manual", now, now),
        )
        conn.execute(
            "INSERT INTO runs("
            "id, experiment_id, status, run_order, agent_id, agent_snapshot_hash, "
            "benchmark_name, n_attempts, n_concurrent, created_at, updated_at, "
            "leaderboard_eligible, comparability_key"
            ") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (
                "run-active",
                "exp-active",
                "completed",
                1,
                "active-agent",
                "hash",
                "terminal-bench",
                1,
                1,
                now,
                now,
                0,
                "key",
            ),
        )
