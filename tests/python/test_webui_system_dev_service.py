from __future__ import annotations

import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer
from threading import Thread

from ornnlab.services.webui_system_service import _health_endpoint_ok

API = "/api/webui/v1"


def test_system_restart_reports_real_supervisor_requirement(client):
    response = client.post(f"{API}/system/service/restart")

    assert response.status_code == 200
    operation = response.json()["data"]["operation"]
    assert operation["status"] == "failed"
    assert operation["error"]["code"] == "SERVICE_RESTART_UNAVAILABLE"
    persisted = client.get(f"{API}/operations/{operation['id']}").json()["data"]
    assert persisted["status"] == "failed"


def test_system_health_reports_app_level_dev_service_state(client, tmp_path, monkeypatch):
    service_home = tmp_path / "dev-service"
    logs_dir = service_home / "logs"
    logs_dir.mkdir(parents=True)
    state = {
        "serviceId": "ornnlab-dev-service",
        "status": "running",
        "daemonPid": os.getpid(),
        "daemonToken": "daemon-token",
        "backendPid": os.getpid(),
        "backendToken": "backend-token",
        "frontendPid": os.getpid(),
        "frontendToken": "frontend-token",
        "backendUrl": "http://127.0.0.1:8765",
        "frontendUrl": "http://127.0.0.1:5173",
        "updatedAt": "2026-07-13T00:00:00Z",
    }
    (service_home / "state.json").write_text(json.dumps(state), encoding="utf-8")
    monkeypatch.setenv("ORNNLAB_DEV_SERVICE_HOME", str(service_home))
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._process_command",
        lambda _pid: "daemon-token backend-token frontend-token",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._health_endpoint_ok",
        lambda _url: True,
    )

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    service = next(component for component in components if component["kind"] == "ornnlab-service")
    assert service["status"] == "healthy"
    assert service["value"] == "running http://127.0.0.1:5173"
    assert service["path"] == str(logs_dir)
    assert service["actions"] == ["check-update", "restart-service"]


def test_system_health_reports_dev_service_stopped_without_state(client, tmp_path, monkeypatch):
    service_home = tmp_path / "dev-service"
    monkeypatch.setenv("ORNNLAB_DEV_SERVICE_HOME", str(service_home))

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    service = next(component for component in components if component["kind"] == "ornnlab-service")
    assert service["status"] == "unavailable"
    assert service["value"] == "stopped"
    assert service["path"] == str(service_home / "logs")


def test_system_health_degrades_when_dev_service_child_is_not_trusted(
    client, tmp_path, monkeypatch
):
    service_home = tmp_path / "dev-service"
    (service_home / "logs").mkdir(parents=True)
    state = {
        "serviceId": "ornnlab-dev-service",
        "status": "running",
        "daemonPid": os.getpid(),
        "daemonToken": "daemon-token",
        "backendPid": os.getpid(),
        "backendToken": "backend-token",
        "frontendPid": os.getpid(),
        "frontendToken": "frontend-token",
        "frontendUrl": "http://127.0.0.1:5173",
    }
    (service_home / "state.json").write_text(json.dumps(state), encoding="utf-8")
    monkeypatch.setenv("ORNNLAB_DEV_SERVICE_HOME", str(service_home))
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._process_command",
        lambda _pid: "daemon-token backend-token",
    )

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    service = next(component for component in components if component["kind"] == "ornnlab-service")
    assert service["status"] == "unavailable"
    assert service["value"] == "degraded http://127.0.0.1:5173"


def test_system_health_degrades_when_dev_service_health_probe_fails(
    client, tmp_path, monkeypatch
):
    service_home = tmp_path / "dev-service"
    (service_home / "logs").mkdir(parents=True)
    state = {
        "serviceId": "ornnlab-dev-service",
        "status": "running",
        "daemonPid": os.getpid(),
        "daemonToken": "daemon-token",
        "backendPid": os.getpid(),
        "backendToken": "backend-token",
        "frontendPid": os.getpid(),
        "frontendToken": "frontend-token",
        "frontendUrl": "http://127.0.0.1:5173",
        "backendUrl": "http://127.0.0.1:8765",
    }
    (service_home / "state.json").write_text(json.dumps(state), encoding="utf-8")
    monkeypatch.setenv("ORNNLAB_DEV_SERVICE_HOME", str(service_home))
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._process_command",
        lambda _pid: "daemon-token backend-token frontend-token",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._health_endpoint_ok",
        lambda _url: False,
    )

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    service = next(component for component in components if component["kind"] == "ornnlab-service")
    assert service["status"] == "unavailable"
    assert service["value"] == "degraded http://127.0.0.1:5173"


def test_system_health_dev_service_probe_does_not_call_system_health(
    client, tmp_path, monkeypatch
):
    service_home = tmp_path / "dev-service"
    (service_home / "logs").mkdir(parents=True)
    state = {
        "serviceId": "ornnlab-dev-service",
        "status": "running",
        "daemonPid": os.getpid(),
        "daemonToken": "daemon-token",
        "backendPid": os.getpid(),
        "backendToken": "backend-token",
        "frontendPid": os.getpid(),
        "frontendToken": "frontend-token",
        "frontendUrl": "http://127.0.0.1:5173",
        "backendUrl": "http://127.0.0.1:8765",
    }
    (service_home / "state.json").write_text(json.dumps(state), encoding="utf-8")
    monkeypatch.setenv("ORNNLAB_DEV_SERVICE_HOME", str(service_home))
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._process_command",
        lambda _pid: "daemon-token backend-token frontend-token",
    )
    probed_urls = []

    def record_probe(url):
        probed_urls.append(url)
        return True

    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._health_endpoint_ok",
        record_probe,
    )

    response = client.get(f"{API}/system/health")

    assert response.status_code == 200
    assert probed_urls == ["http://127.0.0.1:5173"]


def test_health_endpoint_probe_bypasses_system_proxy(monkeypatch):
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b"ok")

        def log_message(self, format: str, *args: object) -> None:
            del format, args

    server = HTTPServer(("127.0.0.1", 0), Handler)
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    monkeypatch.setenv("http_proxy", "http://127.0.0.1:9")
    monkeypatch.setenv("https_proxy", "http://127.0.0.1:9")
    monkeypatch.delenv("NO_PROXY", raising=False)
    monkeypatch.delenv("no_proxy", raising=False)
    try:
        assert _health_endpoint_ok(f"http://127.0.0.1:{server.server_port}") is True
    finally:
        server.shutdown()
        server.server_close()
