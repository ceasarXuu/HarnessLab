from __future__ import annotations

import json

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.storage import sqlite
from tests.python.support import create_test_agent

API = "/api/webui/v1"


def test_running_job_reads_native_harbor_progress_before_result_path_is_persisted(
    client, settings
):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Live progress",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            benchmark_version="2.0",
            n_tasks=None,
            n_attempts=1,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    native_dir = jobs_dir / "live-progress"
    native_dir.mkdir(parents=True)
    (jobs_dir / "result.json").write_text(
        json.dumps({"n_total_trials": 99}), encoding="utf-8"
    )
    (native_dir / "result.json").write_text(
        json.dumps(
            {
                "n_total_trials": 10,
                "stats": {
                    "n_completed_trials": 3,
                    "n_errored_trials": 1,
                    "n_running_trials": 2,
                    "n_pending_trials": 5,
                    "evals": {},
                },
            }
        ),
        encoding="utf-8",
    )
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'running', started_at = ?, job_dir = ?, "
            "harbor_job_name = ?, result_path = NULL WHERE id = ?",
            ("2026-07-22T12:50:34+00:00", str(jobs_dir), "live-progress", run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}")

    assert response.status_code == 200
    assert response.json()["data"]["trial"] == {
        "total": 10,
        "completed": 2,
        "passed": 0,
        "notPassed": 2,
        "errored": 1,
    }


def test_running_job_rejects_unsafe_native_result_name(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Unsafe live progress",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    jobs_dir.mkdir()
    outside_result = settings.home / "result.json"
    outside_result.write_text(json.dumps({"n_total_trials": 99}), encoding="utf-8")
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'running', started_at = ?, job_dir = ?, "
            "harbor_job_name = '../', result_path = NULL WHERE id = ?",
            ("2026-07-22T12:50:34+00:00", str(jobs_dir), run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}")

    assert response.status_code == 200
    assert response.json()["data"]["trial"]["total"] == 0
