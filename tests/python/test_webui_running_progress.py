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


def test_interrupted_job_still_reads_native_counts(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Interrupted counts",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    native_dir = jobs_dir / "interrupted-job"
    native_dir.mkdir(parents=True)
    (native_dir / "config.json").write_text("{}", encoding="utf-8")
    (native_dir / "result.json").write_text(
        json.dumps(
            {
                "n_total_trials": 10,
                "stats": {
                    "n_completed_trials": 7,
                    "n_errored_trials": 0,
                    "n_running_trials": 2,
                    "n_pending_trials": 1,
                    "evals": {
                        "claude-code__deepseek-v4-pro__terminal-bench-sample": {
                            "reward_stats": {"reward": {"1.0": ["a", "b", "c"], "0.0": ["d"]}}
                        }
                    },
                },
            }
        ),
        encoding="utf-8",
    )
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'interrupted', started_at = ?, job_dir = ?, "
            "harbor_job_name = ?, result_path = NULL WHERE id = ?",
            ("2026-08-07T13:09:03Z", str(jobs_dir), "interrupted-job", run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}")

    assert response.status_code == 200
    assert response.json()["data"]["trial"] == {
        "total": 10,
        "completed": 7,
        "passed": 3,
        "notPassed": 4,
        "errored": 0,
    }


def test_interrupted_job_trials_do_not_claim_running(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Interrupted trials",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    native_dir = jobs_dir / "interrupted-job"
    native_dir.mkdir(parents=True)
    (native_dir / "config.json").write_text("{}", encoding="utf-8")
    (native_dir / "result.json").write_text(
        json.dumps({"n_total_trials": 2, "stats": {}}), encoding="utf-8"
    )

    completed = native_dir / "sqlite__def"
    completed.mkdir()
    (completed / "config.json").write_text(
        json.dumps({"trial_name": "sqlite__def", "task": {"path": "sample/sqlite-with-gcov"}}),
        encoding="utf-8",
    )
    (completed / "result.json").write_text(
        json.dumps({"id": "trial-1", "task_name": "sqlite-with-gcov", "agent_result": {}}),
        encoding="utf-8",
    )

    stale = native_dir / "build-cython-ext__abc"
    stale.mkdir()
    (stale / "config.json").write_text(
        json.dumps(
            {"trial_name": "build-cython-ext__abc", "task": {"path": "sample/build-cython-ext"}}
        ),
        encoding="utf-8",
    )
    (stale / "trial.log").write_text("output\n", encoding="utf-8")

    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'interrupted', started_at = ?, job_dir = ?, "
            "harbor_job_name = ?, result_path = NULL WHERE id = ?",
            ("2026-08-07T13:09:03Z", str(jobs_dir), "interrupted-job", run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}/trials")

    assert response.status_code == 200
    trials = response.json()["data"]
    assert [trial["taskName"] for trial in trials] == ["sqlite-with-gcov"]
    assert all(trial["status"] != "running" for trial in trials)


def test_running_job_trials_list_includes_in_progress_trials(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Live trials",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    native_dir = jobs_dir / "live-trials"
    native_dir.mkdir(parents=True)
    (native_dir / "config.json").write_text("{}", encoding="utf-8")
    (native_dir / "result.json").write_text(
        json.dumps({"n_total_trials": 2, "stats": {}}), encoding="utf-8"
    )

    completed = native_dir / "sqlite__def"
    completed.mkdir()
    (completed / "config.json").write_text(
        json.dumps({"trial_name": "sqlite__def", "task": {"path": "sample/sqlite-with-gcov"}}),
        encoding="utf-8",
    )
    (completed / "result.json").write_text(
        json.dumps(
            {
                "id": "trial-1",
                "task_name": "sqlite-with-gcov",
                "started_at": "2026-08-07T13:09:07Z",
                "finished_at": "2026-08-07T13:14:19Z",
                "agent_result": {},
            }
        ),
        encoding="utf-8",
    )

    running = native_dir / "chess__abc"
    running.mkdir()
    (running / "config.json").write_text(
        json.dumps({"trial_name": "chess__abc", "task": {"path": "sample/chess-best-move"}}),
        encoding="utf-8",
    )
    (running / "trial.log").write_text("agent output\n", encoding="utf-8")

    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'running', started_at = ?, job_dir = ?, "
            "harbor_job_name = ?, result_path = NULL WHERE id = ?",
            ("2026-08-07T13:09:03Z", str(jobs_dir), "live-trials", run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}/trials")

    assert response.status_code == 200
    by_task = {trial["taskName"]: trial for trial in response.json()["data"]}
    assert set(by_task) == {"sqlite-with-gcov", "chess-best-move"}
    assert by_task["sqlite-with-gcov"]["status"] == "passed"
    assert by_task["chess-best-move"]["status"] == "running"
    assert by_task["chess-best-move"]["logPath"] == str(running / "trial.log")
    assert by_task["chess-best-move"]["score"] is None
    assert by_task["chess-best-move"]["runtimeSeconds"] is None


def test_job_logs_reads_the_native_job_log(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Live logs",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    jobs_dir.mkdir()
    (jobs_dir / "job.log").write_text("line1\nline2\nagent output\n", encoding="utf-8")
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'running', job_dir = ?, result_path = NULL WHERE id = ?",
            (str(jobs_dir), run_id),
        )

    response = client.get(f"{API}/jobs/{run_id}/logs")

    assert response.status_code == 200
    data = response.json()["data"]
    assert data["logPath"] == str(jobs_dir / "job.log")
    assert data["content"] == "line1\nline2\nagent output\n"


def test_job_logs_tail_large_files_and_return_empty_without_dir(client, settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Large logs",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench-sample"],
            n_tasks=None,
        )
    )
    run_id = created["runs"][0]["id"]
    jobs_dir = settings.home / "shared-jobs"
    jobs_dir.mkdir()
    (jobs_dir / "job.log").write_text("z" * 200_000 + "\nTAIL", encoding="utf-8")
    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'running', job_dir = ?, result_path = NULL WHERE id = ?",
            (str(jobs_dir), run_id),
        )

    data = client.get(f"{API}/jobs/{run_id}/logs").json()["data"]
    assert data["content"].endswith("\nTAIL")
    assert len(data["content"]) == 200_000

    with sqlite.connect(settings) as conn:
        conn.execute(
            "UPDATE runs SET job_dir = NULL WHERE id = ?",
            (run_id,),
        )
    empty = client.get(f"{API}/jobs/{run_id}/logs").json()["data"]
    assert empty["logPath"] is None
    assert empty["content"] == ""
