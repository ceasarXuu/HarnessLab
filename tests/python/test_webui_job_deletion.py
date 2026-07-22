from __future__ import annotations

import json
from pathlib import Path
from typing import TypedDict

from ornnlab.storage import sqlite

API = "/api/webui/v1"


class SeededJob(TypedDict):
    experiment_id: str
    experiment_root: Path
    job_id: str
    native_root: Path
    run_root: Path
    shared_sibling: Path


def test_delete_job_removes_all_records_leaderboard_and_owned_artifacts(client, settings):
    seeded = _seed_job(settings, status="completed")

    response = client.delete(f"{API}/jobs/{seeded['job_id']}")

    assert response.status_code == 200
    assert response.json()["data"] == {"deletedJobId": seeded["job_id"]}
    with sqlite.connect(settings) as conn:
        assert _count(conn, "runs", "id", seeded["job_id"]) == 0
        assert _count(conn, "experiments", "id", seeded["experiment_id"]) == 0
        assert _count(conn, "queue_items", "run_id", seeded["job_id"]) == 0
        assert _count(conn, "webui_job_configs", "run_id", seeded["job_id"]) == 0
        assert _count(conn, "webui_operations", "resource_id", seeded["job_id"]) == 0
        assert _count(conn, "experiment_events", "aggregate_id", seeded["job_id"]) == 0
        assert _count(conn, "experiment_events", "aggregate_id", seeded["experiment_id"]) == 0
    assert not seeded["native_root"].exists()
    assert not seeded["run_root"].exists()
    assert not seeded["experiment_root"].exists()
    assert seeded["shared_sibling"].is_file()
    assert client.get(f"{API}/jobs/{seeded['job_id']}").status_code == 404
    leaderboard = client.get(
        f"{API}/leaderboard", params={"dataset": "example@1.0"}
    ).json()["data"]
    assert all(entry["jobId"] != seeded["job_id"] for entry in leaderboard["items"])


def test_delete_job_rejects_active_runs_without_removing_data(client, settings):
    seeded = _seed_job(settings, status="running")

    response = client.delete(f"{API}/jobs/{seeded['job_id']}")

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "OPERATION_CONFLICT"
    with sqlite.connect(settings) as conn:
        assert _count(conn, "runs", "id", seeded["job_id"]) == 1
    assert seeded["native_root"].is_dir()


def test_delete_job_refuses_unowned_external_artifacts(client, settings, tmp_path):
    seeded = _seed_job(settings, status="failed", create_native_root=False)
    shared_result = tmp_path / "shared-jobs" / "result.json"
    shared_result.write_text("{}", encoding="utf-8")
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET result_path = ?, harbor_job_name = NULL WHERE id = ?",
            (str(shared_result), seeded["job_id"]),
        )

    response = client.delete(f"{API}/jobs/{seeded['job_id']}")

    assert response.status_code == 422
    assert "ownership cannot be proven" in response.json()["error"]["message"]
    assert shared_result.is_file()
    with sqlite.connect(settings) as conn:
        assert _count(conn, "runs", "id", seeded["job_id"]) == 1


def _seed_job(settings, *, status: str, create_native_root: bool = True) -> SeededJob:
    job_id = "run-delete-target"
    experiment_id = "exp-delete-target"
    shared_jobs = settings.home / "shared-jobs"
    native_root = shared_jobs / "owned-job"
    shared_jobs.mkdir(parents=True)
    if create_native_root:
        native_root.mkdir()
        (native_root / "config.json").write_text("{}", encoding="utf-8")
        (native_root / "result.json").write_text("{}", encoding="utf-8")
    shared_sibling = shared_jobs / "keep.txt"
    shared_sibling.write_text("keep", encoding="utf-8")
    run_root = settings.experiments_dir / job_id
    report_dir = run_root / "report"
    report_dir.mkdir(parents=True)
    report_path = report_dir / "index.html"
    report_path.write_text("report", encoding="utf-8")
    run_mirror = run_root / "ornnlab-events.jsonl"
    run_mirror.write_text("event", encoding="utf-8")
    experiment_root = settings.experiments_dir / experiment_id
    experiment_root.mkdir()
    experiment_mirror = experiment_root / "ornnlab-events.jsonl"
    experiment_mirror.write_text("event", encoding="utf-8")
    result_path = native_root / "result.json" if create_native_root else None

    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO experiments(id, name, kind, status, requested_run_count, mode, "
            "created_at, updated_at) VALUES (?, ?, 'benchmark', ?, 1, 'webui', ?, ?)",
            (
                experiment_id,
                "delete target",
                status,
                "2026-07-22T00:00:00Z",
                "2026-07-22T00:00:00Z",
            ),
        )
        conn.execute(
            "INSERT INTO runs(id, experiment_id, status, run_order, agent_id, "
            "agent_snapshot_hash, benchmark_name, benchmark_version, n_attempts, "
            "n_concurrent, harbor_job_name, created_at, updated_at, job_dir, result_path, "
            "report_path, leaderboard_eligible) VALUES (?, ?, ?, 1, 'agent', 'hash', "
            "'example', '1.0', 1, 1, 'owned-job', ?, ?, ?, ?, ?, 1)",
            (
                job_id,
                experiment_id,
                status,
                "2026-07-22T00:00:00Z",
                "2026-07-22T00:00:00Z",
                str(shared_jobs),
                str(result_path) if result_path else None,
                str(report_path),
            ),
        )
        conn.execute(
            "INSERT INTO queue_items(run_id, queue_position, state, enqueued_at) "
            "VALUES (?, 1, ?, '2026-07-22T00:00:00Z')",
            (job_id, status),
        )
        conn.execute(
            "INSERT INTO webui_job_configs(run_id, config_json, notes, environment_preset_id) "
            "VALUES (?, ?, '', 'docker-default')",
            (job_id, json.dumps({"job_name": "owned-job"})),
        )
        conn.execute(
            "INSERT INTO webui_operations(id, operation_type, status, resource_type, "
            "resource_id, created_at) VALUES ('op-job', 'cancel-job', 'completed', "
            "'job', ?, '2026-07-22T00:00:00Z')",
            (job_id,),
        )
        _insert_event(conn, job_id, str(run_mirror))
        _insert_event(conn, experiment_id, str(experiment_mirror))
    return {
        "experiment_id": experiment_id,
        "experiment_root": experiment_root,
        "job_id": job_id,
        "native_root": native_root,
        "run_root": run_root,
        "shared_sibling": shared_sibling,
    }


def _insert_event(conn, aggregate_id: str, mirror_file: str) -> None:
    conn.execute(
        "INSERT INTO experiment_events(aggregate_type, aggregate_id, ts, event_type, "
        "severity, payload_json, mirror_file) VALUES ('run', ?, "
        "'2026-07-22T00:00:00Z', 'test.event', 'info', '{}', ?)",
        (aggregate_id, mirror_file),
    )


def _count(conn, table: str, field: str, value: str) -> int:
    return conn.execute(f"SELECT COUNT(*) FROM {table} WHERE {field} = ?", (value,)).fetchone()[0]
