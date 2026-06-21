from __future__ import annotations

import sys


def test_system_status_reports_core_fields(client):
    response = client.get("/api/system/status")

    assert response.status_code == 200
    payload = response.json()
    assert payload["db_schema_version"] == 3
    assert payload["data_dir"]
    assert payload["stale_running_runs"] == 0
    assert "docker" in payload
    assert "ornnlab_orphans" in payload["docker"]
    assert "harbor_version" in payload


def test_system_status_warns_about_ornnlab_docker_orphans(client, monkeypatch, tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import sys",
                "if sys.argv[4] != 'label=ornnlab.run_id':",
                "    raise SystemExit(0)",
                "print(json.dumps({",
                "    'ID': 'orphan1',",
                "    'Names': 'ornnlab-orphan',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Exited (137)',",
                "    'Labels': 'ornnlab.run_id=run-orphan',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    response = client.get("/api/system/status")

    assert response.status_code == 200
    payload = response.json()
    assert payload["docker"]["available"] is True
    assert payload["docker"]["ornnlab_orphans"]["count"] == 1
    assert "docker_orphans_detected" in payload["warnings"]


def test_system_docker_orphans_endpoint_returns_cleanup_plan(client, monkeypatch, tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import sys",
                "if sys.argv[4] != 'label=ornnlab.run_id':",
                "    raise SystemExit(0)",
                "print(json.dumps({",
                "    'ID': 'abc123',",
                "    'Names': 'ornnlab-orphan',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Exited (137)',",
                "    'Labels': 'ornnlab.run_id=run-123',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    response = client.get("/api/system/docker-orphans")

    assert response.status_code == 200
    payload = response.json()
    assert payload["count"] == 1
    assert payload["cleanup_plan"][0]["dry_run"] is True
    assert payload["cleanup_plan"][0]["manual_review_required"] is True


def test_system_status_reports_legacy_runtime_env_warnings(client, monkeypatch, tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text("", encoding="utf-8")
    monkeypatch.delenv("ORNNLAB_DOCKER_COMMAND", raising=False)
    monkeypatch.setenv("HARNESSLAB_DOCKER_COMMAND", f"{sys.executable} {script}")
    monkeypatch.setenv("HARNESSLAB_HARBOR_ENGINE", "subprocess")

    response = client.get("/api/system/status")

    assert response.status_code == 200
    payload = response.json()
    assert "legacy_docker_command_in_use" in payload["runtime_env_warnings"]
    assert "legacy_harbor_engine_in_use" in payload["runtime_env_warnings"]
    assert "legacy_docker_command_in_use" in payload["warnings"]
    assert "legacy_harbor_engine_in_use" in payload["warnings"]


def test_system_doctor_logs_reports_latest_failed_run(client):
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
            "name": "Failure",
            "agent_ids": ["oracle"],
            "benchmark_names": ["fake-docker-failure"],
        },
    ).json()
    client.post(f"/api/experiments/{created['experiment']['id']}/run?wait=true")

    response = client.post("/api/system/doctor?logs=true")

    assert response.status_code == 200
    logs = response.json()["logs"]
    assert logs["latest_failed_run"]["experiment_name"] == "Failure"
    assert logs["latest_failed_run"]["failure_class"] == "docker_resource_failure"
    assert logs["latest_failed_run"]["paths"]["job_log"].endswith("job.log")
    assert "check_docker_resources" in logs["remediation"]
