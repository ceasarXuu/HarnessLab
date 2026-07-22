from __future__ import annotations

import json
import sys

from ornnlab.services.docker_orphan_service import DockerOrphanService


def test_scan_finds_only_inactive_auto_cleanup_containers(tmp_path):
    script = _docker_fixture(
        tmp_path,
        [
            _container("orphan", "run-orphan", "instance-1"),
            _container("active", "run-active", "instance-1"),
            _container("retained", "run-retained", "instance-1", cleanup="retain"),
        ],
    )

    result = DockerOrphanService(
        command=[sys.executable, str(script)], instance_id="instance-1"
    ).scan_ornnlab_containers({"run-active"})

    assert result["ok"] is True
    assert result["owned_count"] == 3
    assert result["count"] == 1
    assert result["containers"][0]["id"] == "orphan"
    assert result["cleanup_plan"] == [
        {
            "container_id": "orphan",
            "name": "ornnlab-orphan",
            "run_id": "run-orphan",
            "instance_id": "instance-1",
            "command": [sys.executable, str(script), "rm", "-f", "orphan"],
            "dry_run": True,
            "manual_review_required": False,
        }
    ]


def test_cleanup_run_removes_containers_and_compose_resources(tmp_path):
    calls = tmp_path / "calls.jsonl"
    script = _docker_fixture(
        tmp_path,
        [
            _container(
                "abc123",
                "run-1",
                "instance-1",
                project="harbor-project",
            )
        ],
        calls=calls,
    )

    result = DockerOrphanService(
        command=[sys.executable, str(script)], instance_id="instance-1"
    ).cleanup_run("run-1")

    assert result == {
        "ok": True,
        "run_id": "run-1",
        "instance_id": "instance-1",
        "matched_containers": 1,
        "removed_containers": 1,
        "removed_networks": 1,
        "removed_volumes": 1,
        "projects": ["harbor-project"],
        "errors": [],
    }
    commands = [json.loads(line) for line in calls.read_text().splitlines()]
    assert ["rm", "-f", "abc123"] in commands
    assert ["network", "rm", "network-1"] in commands
    assert ["volume", "rm", "volume-1"] in commands


def test_cleanup_run_does_not_remove_retained_container(tmp_path):
    script = _docker_fixture(
        tmp_path,
        [_container("retained", "run-1", "instance-1", cleanup="retain")],
    )

    result = DockerOrphanService(
        command=[sys.executable, str(script)], instance_id="instance-1"
    ).cleanup_run("run-1")

    assert result["ok"] is True
    assert result["matched_containers"] == 0
    assert result["removed_containers"] == 0


def test_cleanup_run_removes_owned_network_and_volume_after_container_is_gone(tmp_path):
    script = _docker_fixture(tmp_path, [])

    result = DockerOrphanService(
        command=[sys.executable, str(script)], instance_id="instance-1"
    ).cleanup_run("run-partial")

    assert result["matched_containers"] == 0
    assert result["removed_networks"] == 1
    assert result["removed_volumes"] == 1


def test_cleanup_run_requires_instance_boundary(tmp_path):
    script = _docker_fixture(tmp_path, [_container("abc", "run-1", "instance-1")])

    result = DockerOrphanService(command=[sys.executable, str(script)]).cleanup_run("run-1")

    assert result["ok"] is False
    assert result["errors"] == ["docker_cleanup_instance_id_required"]


def test_docker_orphan_scan_reports_cli_failure(tmp_path):
    script = tmp_path / "fake_docker_failure.py"
    script.write_text(
        "import sys\nprint('daemon unavailable', file=sys.stderr)\nsys.exit(1)\n",
        encoding="utf-8",
    )

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["available"] is True
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


def test_docker_orphan_scan_reports_parse_failure(tmp_path):
    script = tmp_path / "fake_docker_bad_json.py"
    script.write_text("print('not-json')", encoding="utf-8")

    result = DockerOrphanService(command=[sys.executable, str(script)]).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["error"].startswith("docker_ps_parse_failed")


def test_docker_orphan_scan_reports_timeout(tmp_path):
    script = tmp_path / "fake_docker_slow.py"
    script.write_text("import time; time.sleep(1)", encoding="utf-8")

    result = DockerOrphanService(
        command=[sys.executable, str(script)], timeout_sec=0.01
    ).scan_ornnlab_containers()

    assert result["ok"] is False
    assert result["error"] == "docker_ps_timeout"


def _container(
    container_id: str,
    run_id: str,
    instance_id: str,
    *,
    cleanup: str = "auto",
    project: str = "",
) -> dict[str, str]:
    labels = [
        "ornnlab.managed=true",
        f"ornnlab.instance_id={instance_id}",
        f"ornnlab.run_id={run_id}",
        f"ornnlab.cleanup={cleanup}",
    ]
    if project:
        labels.append(f"com.docker.compose.project={project}")
    return {
        "ID": container_id,
        "Names": f"ornnlab-{container_id}",
        "Image": "harbor-runner:latest",
        "Status": "Exited (137)",
        "Labels": ",".join(labels),
    }


def _docker_fixture(tmp_path, containers, calls=None):
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import pathlib",
                "import sys",
                f"containers = {containers!r}",
                f"calls = pathlib.Path({str(calls)!r}) if {calls is not None!r} else None",
                "args = sys.argv[1:]",
                "if calls is not None:",
                "    with calls.open('a') as handle:",
                "        handle.write(json.dumps(args) + '\\n')",
                "if args[:2] == ['ps', '-a']:",
                "    for container in containers:",
                "        print(json.dumps(container))",
                "elif args[:2] == ['network', 'ls']:",
                "    print('network-1')",
                "elif args[:2] == ['volume', 'ls']:",
                "    print('volume-1')",
            ]
        ),
        encoding="utf-8",
    )
    return script
