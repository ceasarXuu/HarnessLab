from __future__ import annotations

import sys

from ornnlab.services.docker_orphan_service import DockerOrphanService


def test_docker_orphan_scan_finds_ornnlab_labelled_containers(tmp_path):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import sys",
                "assert sys.argv[1:4] == ['ps', '-a', '--filter']",
                "assert sys.argv[5:] == ['--format', '{{json .}}']",
                "if sys.argv[4] != 'label=ornnlab.run_id':",
                "    raise SystemExit(0)",
                "print(json.dumps({",
                "    'ID': 'abc123',",
                "    'Names': 'ornnlab-task-1',",
                "    'Image': 'harbor-runner:latest',",
                "    'Status': 'Up 3 minutes',",
                "    'Labels': 'ornnlab.run_id=run-1,com.docker.compose.project=harbor',",
                "}))",
            ]
        ),
        encoding="utf-8",
    )

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is True
    assert result["available"] is True
    assert result["count"] == 1
    assert result["containers"][0]["labels"]["ornnlab.run_id"] == "run-1"
    assert result["cleanup_plan"] == [
        {
            "container_id": "abc123",
            "name": "ornnlab-task-1",
            "run_id": "run-1",
            "command": [sys.executable, str(script), "rm", "-f", "abc123"],
            "dry_run": True,
            "manual_review_required": True,
        }
    ]


def test_docker_orphan_scan_reports_cli_failure(tmp_path):
    script = tmp_path / "fake_docker_failure.py"
    script.write_text(
        "\n".join(
            [
                "import sys",
                "print('daemon unavailable', file=sys.stderr)",
                "sys.exit(1)",
            ]
        ),
        encoding="utf-8",
    )

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["available"] is True
    assert result["count"] == 0
    assert result["cleanup_plan"] == []
    assert result["error"] == "daemon unavailable"


def test_docker_orphan_scan_reports_missing_cli():
    result = DockerOrphanService(command=["/definitely/missing/docker"]).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["available"] is False
    assert result["error"] == "docker_cli_missing"


def test_docker_orphan_scan_handles_empty_output(tmp_path):
    script = tmp_path / "fake_docker_empty.py"
    script.write_text("", encoding="utf-8")

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is True
    assert result["count"] == 0
    assert result["containers"] == []
    assert result["cleanup_plan"] == []


def test_docker_orphan_scan_reports_parse_failure(tmp_path):
    script = tmp_path / "fake_docker_bad_json.py"
    script.write_text("print('not-json')", encoding="utf-8")

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["available"] is True
    assert result["error"].startswith("docker_ps_parse_failed")


def test_docker_orphan_scan_reports_timeout(tmp_path):
    script = tmp_path / "fake_docker_slow.py"
    script.write_text("import time; time.sleep(1)", encoding="utf-8")

    result = DockerOrphanService(
        command=[sys.executable, str(script)],
        timeout_sec=0.01,
    ).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["available"] is True
    assert result["error"] == "docker_ps_timeout"
