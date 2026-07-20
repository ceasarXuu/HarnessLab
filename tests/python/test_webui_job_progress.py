from datetime import datetime

from ornnlab.services.webui_job_progress import job_trial_progress, runtime_seconds


def test_job_trial_progress_separates_scored_completion_from_exceptions():
    result = {
        "n_total_trials": 10,
        "stats": {
            "n_completed_trials": 10,
            "n_errored_trials": 2,
            "evals": {
                "terminal-bench": {
                    "reward_stats": {
                        "reward": {
                            "0.0": ["failed-a", "failed-b", "failed-c", "failed-d"],
                            "1.0": ["passed-a", "passed-b", "passed-c", "passed-d"],
                        }
                    }
                }
            },
        },
    }

    assert job_trial_progress(result, expected_total=10) == {
        "total": 10,
        "completed": 8,
        "passed": 4,
        "notPassed": 4,
        "errored": 2,
    }


def test_job_trial_progress_keeps_unfinished_work_out_of_terminal_buckets():
    result = {
        "n_total_trials": 10,
        "stats": {"n_completed_trials": 3, "n_errored_trials": 1, "evals": {}},
    }

    assert job_trial_progress(result, expected_total=10) == {
        "total": 10,
        "completed": 2,
        "passed": 0,
        "notPassed": 2,
        "errored": 1,
    }


def test_runtime_seconds_uses_now_for_a_running_job_and_finish_for_a_terminal_job():
    started = "2026-07-21T00:00:00+00:00"
    now = datetime.fromisoformat("2026-07-21T00:00:42+00:00")

    assert runtime_seconds(started, None, now=now) == 42
    assert runtime_seconds(started, "2026-07-21T00:01:00+00:00", now=now) == 60

