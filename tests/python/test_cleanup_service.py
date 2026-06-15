from __future__ import annotations

from harnesslab.services.cleanup_service import CleanupService
from harnesslab.services.clock import now_iso
from harnesslab.storage import sqlite


def test_cleanup_plan_finds_only_unreferenced_artifacts(settings):
    _seed_references(settings)
    (settings.generated_agents_dir / "active-agent").mkdir(parents=True)
    (settings.generated_agents_dir / "stale-agent").mkdir()
    (settings.experiments_dir / "exp-active").mkdir(parents=True)
    (settings.experiments_dir / "run-active").mkdir()
    (settings.experiments_dir / "orphan-artifact").mkdir()

    plan = CleanupService(settings).plan()

    assert plan["mode"] == "archive-only"
    assert {(item["type"], item["reason"]) for item in plan["candidates"]} == {
        ("generated-agent", "no_active_agent_row"),
        ("experiment-artifact", "no_experiment_or_run_row"),
    }
    assert {item["path"].split("/")[-1] for item in plan["candidates"]} == {
        "stale-agent",
        "orphan-artifact",
    }


def test_cleanup_archive_moves_candidates_to_recoverable_archive(settings):
    _seed_references(settings)
    stale_agent = settings.generated_agents_dir / "stale-agent"
    stale_agent.mkdir(parents=True)
    (stale_agent / "manifest.json").write_text("{}", encoding="utf-8")
    orphan = settings.experiments_dir / "orphan-artifact"
    orphan.mkdir(parents=True)
    (orphan / "result.json").write_text("{}", encoding="utf-8")

    result = CleanupService(settings).archive()

    assert result["archived_count"] == 2
    assert not stale_agent.exists()
    assert not orphan.exists()
    archived_names = {item["archived_path"].split("/")[-1] for item in result["archived"]}
    assert archived_names == {"stale-agent", "orphan-artifact"}


def _seed_references(settings) -> None:
    sqlite.initialize(settings)
    now = now_iso()
    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO agents("
            "id, name, kind, harbor_agent_name, status, profile_path, created_at, updated_at"
            ") VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                "active-agent",
                "Active",
                "custom-command",
                None,
                "compiled",
                str(settings.agents_dir / "active-agent.json"),
                now,
                now,
            ),
        )
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
