import json
from pathlib import Path

from ornnlab.services.harbor_results import (
    _file_uri_path,
    running_trial_descriptors,
    trial_log_path,
    trial_result_payloads,
)


def test_trial_result_payloads_reads_harbor_native_trial_directories(tmp_path: Path):
    job_path = tmp_path / "native-job"
    (job_path / "trial-a").mkdir(parents=True)
    (job_path / "config.json").write_text("{}", encoding="utf-8")
    (job_path / "result.json").write_text('{"n_total_trials": 1}', encoding="utf-8")
    (job_path / "trial-a" / "result.json").write_text(
        json.dumps({"trial_name": "trial-a"}), encoding="utf-8"
    )

    assert trial_result_payloads(tmp_path, "native-job", None) == [{"trial_name": "trial-a"}]


def test_trial_result_payloads_prefers_legacy_embedded_results(tmp_path: Path):
    result_path = tmp_path / "result.json"
    result_path.write_text(json.dumps({"trial_results": [{"trial_name": "embedded"}]}))

    assert trial_result_payloads(tmp_path, None, str(result_path)) == [{"trial_name": "embedded"}]


def test_running_trial_descriptors_reports_dirs_without_results_only(tmp_path: Path):
    job_path = tmp_path / "native-job"
    (job_path / "chess__abc").mkdir(parents=True)
    (job_path / "sqlite__def").mkdir(parents=True)
    (job_path / "config.json").write_text("{}", encoding="utf-8")
    (job_path / "result.json").write_text('{"n_total_trials": 2}', encoding="utf-8")
    (job_path / "chess__abc" / "config.json").write_text(
        json.dumps({"trial_name": "chess__abc", "task": {"path": "sample/chess-best-move"}}),
        encoding="utf-8",
    )
    (job_path / "chess__abc" / "trial.log").write_text("agent output\n", encoding="utf-8")
    (job_path / "sqlite__def" / "config.json").write_text(
        json.dumps({"trial_name": "sqlite__def", "task": {"path": "sample/sqlite-with-gcov"}}),
        encoding="utf-8",
    )
    (job_path / "sqlite__def" / "result.json").write_text('{"trial_name": "sqlite__def"}')

    assert running_trial_descriptors(tmp_path, "native-job", None) == [
        {
            "trial_name": "chess__abc",
            "task_name": "chess-best-move",
            "log_path": str(job_path / "chess__abc" / "trial.log"),
        }
    ]


def test_running_trial_descriptors_falls_back_to_legacy_embedded_layout(tmp_path: Path):
    result_path = tmp_path / "result.json"
    result_path.write_text(json.dumps({"trial_results": [{"trial_name": "embedded"}]}))

    assert running_trial_descriptors(tmp_path, None, str(result_path)) == []


def test_running_trial_descriptors_empty_once_job_result_is_finished(tmp_path: Path):
    job_path = tmp_path / "native-job"
    trial_path = job_path / "chess__abc"
    trial_path.mkdir(parents=True)
    (trial_path / "config.json").write_text(
        json.dumps({"trial_name": "chess__abc", "task": {"path": "sample/chess-best-move"}}),
        encoding="utf-8",
    )
    (trial_path / "trial.log").write_text("agent output\n", encoding="utf-8")
    (job_path / "result.json").write_text(
        json.dumps({"n_total_trials": 1, "finished_at": "2026-08-07T13:14:19Z"}),
        encoding="utf-8",
    )

    assert running_trial_descriptors(tmp_path, "native-job", None) == []


def test_trial_log_path_requires_an_existing_file_uri_path(tmp_path: Path):
    trial_path = tmp_path / "trial-a"
    trial_path.mkdir()
    log_path = trial_path / "trial.log"
    log_path.write_text("trial log\n", encoding="utf-8")

    assert trial_log_path({"trial_uri": trial_path.as_uri()}) == str(log_path)
    assert trial_log_path({"trial_uri": "https://example.com/trial"}) is None


def test_file_uri_path_removes_windows_drive_prefix():
    assert str(_file_uri_path("/C:/work/trial", "", windows=True)) == "C:/work/trial"


def test_file_uri_path_preserves_posix_path():
    assert str(_file_uri_path("/work/trial", "", windows=False)) == "/work/trial"


def test_file_uri_path_handles_remote_host():
    assert str(_file_uri_path("/share/path", "server", windows=False)) == "//server/share/path"


def test_file_uri_path_does_not_strip_non_alpha_drive_prefix():
    assert str(_file_uri_path("/1:/path", "", windows=True)) == "/1:/path"
