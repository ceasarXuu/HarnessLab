from __future__ import annotations

import sys
from types import SimpleNamespace

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.doctor_service import DoctorService
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.services.webui_system_service import _choose_native_directory
from tests.python.support import create_test_agent


def test_system_status_reports_core_fields(settings):
    payload = DoctorService(settings).status()

    assert payload["db_schema_version"] == 7
    assert payload["data_dir"]
    assert payload["stale_running_runs"] == 0
    assert "docker" in payload
    assert "ornnlab_orphans" in payload["docker"]
    assert "harbor_version" in payload
    assert payload["harbor_engine"]["mode"] == "subprocess"
    assert payload["harbor_engine"]["supports_cancel"] is True


def test_native_directory_picker_uses_finder_and_handles_cancel(tmp_path, monkeypatch):
    calls: list[list[str]] = []

    def successful_picker(command, **_):
        calls.append(command)
        return SimpleNamespace(returncode=0, stderr="", stdout=f"{tmp_path}/")

    monkeypatch.setattr("ornnlab.services.webui_system_service.platform.system", lambda: "Darwin")
    monkeypatch.setattr("ornnlab.services.webui_system_service.subprocess.run", successful_picker)
    assert _choose_native_directory() == str(tmp_path.resolve())
    assert calls[0][0] == "osascript"

    monkeypatch.setattr(
        "ornnlab.services.webui_system_service.subprocess.run",
        lambda *_args, **_kwargs: SimpleNamespace(
            returncode=1, stderr="User canceled (-128)", stdout=""
        ),
    )
    assert _choose_native_directory() is None


def test_system_status_warns_about_ornnlab_docker_orphans(settings, monkeypatch, tmp_path):
    script = _docker_script(tmp_path, "orphan1")
    monkeypatch.setenv("ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    payload = DoctorService(settings).status()

    assert payload["docker"]["available"] is True
    assert payload["docker"]["ornnlab_orphans"]["count"] == 1
    assert "docker_orphans_detected" in payload["warnings"]


def test_system_docker_orphans_returns_cleanup_plan(settings, monkeypatch, tmp_path):
    script = _docker_script(tmp_path, "abc123")
    monkeypatch.setenv("ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    payload = DoctorService(settings).docker_orphans()

    assert payload["count"] == 1
    assert payload["cleanup_plan"][0]["dry_run"] is True
    assert payload["cleanup_plan"][0]["manual_review_required"] is True


def test_system_doctor_logs_reports_latest_failed_run(settings):
    create_test_agent(settings)
    created = ExperimentService(settings).create(
        ExperimentCreate(
            name="Failure",
            agent_ids=["oracle"],
            benchmark_names=["simulated-docker-failure"],
        )
    )
    import asyncio

    asyncio.run(ExperimentService(settings).run(created["experiment"]["id"]))

    logs = DoctorService(settings).logs_report()

    assert logs["latest_failed_run"]["experiment_name"] == "Failure"
    assert logs["latest_failed_run"]["failure_class"] == "docker_resource_failure"
    assert logs["latest_failed_run"]["paths"]["job_log"].endswith("job.log")
    assert "check_docker_resources" in logs["remediation"]


def _docker_script(tmp_path, container_id: str):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import sys",
                "if sys.argv[4] != 'label=ornnlab.run_id':",
                "    raise SystemExit(0)",
                "print(json.dumps({",
                f"    'ID': '{container_id}',",
                "    'Names': 'ornnlab-orphan',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Exited (137)',",
                "    'Labels': 'ornnlab.run_id=run-123',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    return script
