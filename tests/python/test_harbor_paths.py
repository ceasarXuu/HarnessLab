from pathlib import Path

from ornnlab.services.harbor_paths import (
    resolve_harbor_job_path,
    resolve_harbor_log_path,
    resolve_harbor_result_path,
)


def test_native_harbor_paths_take_precedence_when_present(tmp_path: Path):
    native_dir = tmp_path / "native-job"
    native_dir.mkdir()
    result = native_dir / "result.json"
    log = native_dir / "job.log"
    result.write_text("{}", encoding="utf-8")
    log.write_text("Harbor log\n", encoding="utf-8")

    assert resolve_harbor_result_path(tmp_path, "native-job") == result
    assert resolve_harbor_log_path(tmp_path, "native-job") == log


def test_resume_uses_native_job_directory_when_its_config_exists(tmp_path: Path):
    native_dir = tmp_path / "native-job"
    native_dir.mkdir()
    (native_dir / "config.json").write_text("{}", encoding="utf-8")

    assert resolve_harbor_job_path(tmp_path, "native-job") == native_dir


def test_legacy_paths_remain_available_without_a_native_job(tmp_path: Path):
    assert resolve_harbor_result_path(tmp_path, "missing-job") == tmp_path / "result.json"
    assert resolve_harbor_log_path(tmp_path, "missing-job") == tmp_path / "job.log"
    assert resolve_harbor_job_path(tmp_path, "missing-job") == tmp_path
