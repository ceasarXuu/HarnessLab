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
    assert "harnesslab_orphans" in payload["docker"]
    assert "harbor_version" in payload


def test_system_status_warns_about_harnesslab_docker_orphans(client, monkeypatch, tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "print(json.dumps({",
                "    'ID': 'orphan1',",
                "    'Names': 'harnesslab-orphan',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Exited (137)',",
                "    'Labels': 'harnesslab.run_id=run-orphan',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("HARNESSLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    response = client.get("/api/system/status")

    assert response.status_code == 200
    payload = response.json()
    assert payload["docker"]["available"] is True
    assert payload["docker"]["harnesslab_orphans"]["count"] == 1
    assert "docker_orphans_detected" in payload["warnings"]


def test_system_docker_orphans_endpoint_returns_cleanup_plan(client, monkeypatch, tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "print(json.dumps({",
                "    'ID': 'abc123',",
                "    'Names': 'harnesslab-orphan',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Exited (137)',",
                "    'Labels': 'harnesslab.run_id=run-123',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("HARNESSLAB_DOCKER_COMMAND", f"{sys.executable} {script}")

    response = client.get("/api/system/docker-orphans")

    assert response.status_code == 200
    payload = response.json()
    assert payload["count"] == 1
    assert payload["cleanup_plan"][0]["dry_run"] is True
    assert payload["cleanup_plan"][0]["manual_review_required"] is True
