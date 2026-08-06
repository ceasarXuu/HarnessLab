from __future__ import annotations

import logging
import subprocess
from subprocess import CompletedProcess

from ornnlab.services.doctor_service import DoctorService
from ornnlab.services.system_health_probe import (
    _disk_state,
    _gpu_component,
    _log_probe_transition,
    _probe_signatures,
    _usage_state,
)

API = "/api/webui/v1"


def test_system_health_distinguishes_installed_docker_cli_from_running_daemon(
    client, monkeypatch
):
    monkeypatch.setattr(
        DoctorService,
        "status",
        lambda _self: {
            "harbor_version": "0.13.2",
            "docker": {
                "available": True,
                "cli": "docker",
                "ornnlab_orphans": {
                    "available": True,
                    "ok": False,
                    "command": ["docker"],
                    "error": "failed to connect to the docker API",
                },
            },
        },
    )
    monkeypatch.setattr(
        "ornnlab.services.system_health_probe._docker_context",
        lambda _command: "colima",
        raising=False,
    )
    monkeypatch.setattr(
        "ornnlab.services.system_health_probe._docker_versions",
        lambda _command: ("28.1.1", None),
        raising=False,
    )

    components = client.get(f"{API}/system/health").json()["data"]["items"]
    docker = next(component for component in components if component["kind"] == "docker")

    assert docker == {
        "kind": "docker",
        "state": "not-running",
        "context": "colima",
        "clientVersion": "28.1.1",
        "serverVersion": None,
        "startCommand": "",
        "executablePath": "docker",
        "error": "failed to connect to the docker API",
        "actions": [],
    }
    assert "status" not in docker
    assert "value" not in docker
    assert "path" not in docker


def test_docker_versions_reads_client_and_server_from_standard_docker_output(monkeypatch):
    from ornnlab.services.system_health_probe import _docker_versions

    monkeypatch.setattr(
        "ornnlab.services.system_health_probe.subprocess.run",
        lambda *_args, **_kwargs: CompletedProcess(
            ["docker", "version"],
            0,
            stdout='{"Client":{"Version":"28.1.1"},"Server":{"Version":"27.5.1"}}',
            stderr="",
        ),
    )

    assert _docker_versions(["docker"]) == ("28.1.1", "27.5.1")


def test_system_health_resource_thresholds_are_stable():
    assert _usage_state(69.9) == "normal"
    assert _usage_state(70) == "elevated"
    assert _usage_state(90) == "high"
    assert _disk_state(21 * 1024**3, 100 * 1024**3) == "normal"
    assert _disk_state(19 * 1024**3, 100 * 1024**3) == "low"
    assert _disk_state(4 * 1024**3, 100 * 1024**3) == "critical"


def test_system_health_logs_probe_failures_only_on_state_change(caplog):
    _probe_signatures.clear()
    caplog.set_level(logging.INFO)

    _log_probe_transition("docker", "not-running", "daemon unavailable")
    _log_probe_transition("docker", "not-running", "daemon unavailable")
    _log_probe_transition("docker", "running")

    failure_logs = [
        record for record in caplog.records if "system_health_probe_failed" in record.message
    ]
    recovery_logs = [
        record for record in caplog.records if "system_health_probe_recovered" in record.message
    ]
    assert len(failure_logs) == 1
    assert len(recovery_logs) == 1


def _run_gpu_probe(monkeypatch, error: Exception) -> dict:
    def _raising_run(*_args, **_kwargs):
        raise error

    monkeypatch.setattr(
        "ornnlab.services.system_health_probe.subprocess.run", _raising_run, raising=False
    )
    return _gpu_component()


def test_gpu_probe_surfaces_nvidia_smi_error_detail(monkeypatch):
    component = _run_gpu_probe(
        monkeypatch,
        subprocess.CalledProcessError(
            18,
            ["nvidia-smi"],
            stderr="Failed to initialize NVML: Driver/library version mismatch",
        ),
    )

    assert component["state"] == "error"
    assert component["usagePercent"] is None
    assert component["deviceCount"] == 0
    assert component["error"] == "Failed to initialize NVML: Driver/library version mismatch"


def test_gpu_probe_falls_back_to_returncode_when_smi_has_no_output(monkeypatch):
    component = _run_gpu_probe(
        monkeypatch, subprocess.CalledProcessError(18, ["nvidia-smi"])
    )

    assert component["state"] == "error"
    assert component["error"] == "nvidia-smi exited with 18"


def test_gpu_probe_not_detected_has_no_error(monkeypatch):
    component = _run_gpu_probe(monkeypatch, FileNotFoundError())

    assert component["state"] == "not-detected"
    assert component["error"] is None


def test_gpu_probe_reads_usage_without_error(monkeypatch):
    monkeypatch.setattr(
        "ornnlab.services.system_health_probe.subprocess.run",
        lambda *_args, **_kwargs: CompletedProcess(
            ["nvidia-smi"], 0, stdout="7\n", stderr=""
        ),
        raising=False,
    )

    component = _gpu_component()

    assert component["state"] == "normal"
    assert component["usagePercent"] == 7.0
    assert component["deviceCount"] == 1
    assert component["error"] is None
