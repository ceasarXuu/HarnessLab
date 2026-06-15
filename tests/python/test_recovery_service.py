import json

from fastapi.testclient import TestClient

from ornnlab.app import create_app
from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def _create_running_run(
    settings: Settings,
    client: TestClient,
    with_result: bool,
) -> tuple[str, str]:
    client.post(
        "/api/agents",
        json={
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
        },
    )
    created = client.post(
        "/api/experiments",
        json={
            "name": "Recovery",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
            "n_tasks": 1,
        },
    ).json()
    experiment_id = created["experiment"]["id"]
    run_id = created["runs"][0]["id"]
    job_dir = settings.experiments_dir / run_id / "harbor-job"
    job_dir.mkdir(parents=True, exist_ok=True)
    result_path = job_dir / "result.json"
    if with_result:
        result_path.write_text(
            json.dumps({"status": "completed", "score": 0.75}),
            encoding="utf-8",
        )
    now = now_iso()
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
            ("running", now, experiment_id),
        )
        conn.execute(
            "UPDATE runs SET status = ?, started_at = ?, job_dir = ?, updated_at = ? "
            "WHERE id = ?",
            ("running", now, str(job_dir), now, run_id),
        )
        conn.execute(
            "INSERT OR REPLACE INTO queue_items("
            "run_id, queue_position, state, enqueued_at, dequeued_at"
            ") VALUES (?, ?, ?, ?, ?)",
            (run_id, 1, "running", now, now),
        )
    return experiment_id, run_id


def test_startup_recovery_interrupts_stale_running_run(settings):
    experiment_id, run_id = _create_running_run(
        settings,
        TestClient(create_app(settings)),
        with_result=False,
    )

    recovered_app = create_app(settings)
    recovered = TestClient(recovered_app)
    run = recovered.get(f"/api/runs/{run_id}").json()
    queue = QueueRows(settings).by_run_id(run_id)

    assert recovered_app.state.startup_recovery == {"recovered": 0, "interrupted": 1}
    assert recovered.get(f"/api/experiments/{experiment_id}").json()["experiment"][
        "status"
    ] == "interrupted"
    assert run["status"] == "interrupted"
    assert run["failure_class"] == "harbor_recovery"
    assert run["failure_code"] == "stale_running_without_result"
    assert queue["state"] == "interrupted"
    report = recovered.get(f"/api/runs/{run_id}/report").json()
    assert report["summary"]["status"] == "interrupted"
    events = recovered.get(f"/api/runs/{run_id}/events").json()
    assert events[-1]["event_type"] == "experiment.reconcile_decision"


def test_startup_recovery_uses_existing_result_artifact(settings):
    experiment_id, run_id = _create_running_run(
        settings,
        TestClient(create_app(settings)),
        with_result=True,
    )

    recovered_app = create_app(settings)
    recovered = TestClient(recovered_app)
    run = recovered.get(f"/api/runs/{run_id}").json()
    queue = QueueRows(settings).by_run_id(run_id)

    assert recovered_app.state.startup_recovery == {"recovered": 1, "interrupted": 0}
    assert recovered.get(f"/api/experiments/{experiment_id}").json()["experiment"][
        "status"
    ] == "completed"
    assert run["status"] == "completed"
    assert run["score"] == 0.75
    assert queue["state"] == "completed"
    report = recovered.get(f"/api/runs/{run_id}/report").json()
    assert report["summary"]["score"] == 0.75
    events = recovered.get(f"/api/experiments/{experiment_id}/events").json()
    assert events[-1]["event_type"] == "experiment.reconciled"


class QueueRows:
    def __init__(self, settings: Settings):
        self.settings = settings

    def by_run_id(self, run_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT * FROM queue_items WHERE run_id = ?", (run_id,))
        assert rows
        return rows[0]
