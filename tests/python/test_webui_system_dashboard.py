from __future__ import annotations

import logging

from ornnlab.services.doctor_service import DoctorService
from ornnlab.services.system_health_probe import (
    _disk_state,
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

    components = client.get(f"{API}/system/health").json()["data"]["items"]
    docker = next(component for component in components if component["kind"] == "docker")

    assert docker == {
        "kind": "docker",
        "state": "not-running",
        "context": "colima",
        "executablePath": "docker",
        "error": "failed to connect to the docker API",
        "actions": [],
    }
    assert "status" not in docker
    assert "value" not in docker
    assert "path" not in docker


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
