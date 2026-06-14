import asyncio
import time

import pytest
from fastapi.testclient import TestClient

from harnesslab.app import create_app
from harnesslab.models.experiment import ExperimentCreate
from harnesslab.services.agent_service import AgentService
from harnesslab.services.experiment_service import ExperimentService
from harnesslab.services.queue_service import QueueService
from harnesslab.services.worker_service import QueueWorkerService
from harnesslab.storage import sqlite


@pytest.fixture
def anyio_backend():
    return "asyncio"


def test_app_startup_drains_persisted_queue(settings):
    sqlite.initialize(settings)
    AgentService(settings).create(
        {
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
        }
    )
    service = ExperimentService(settings)
    created = service.create(
        ExperimentCreate(
            name="Startup queued",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench"],
            n_tasks=2,
        )
    )
    experiment_id = created["experiment"]["id"]
    service.enqueue(experiment_id)

    with TestClient(create_app(settings)) as client:
        for _ in range(20):
            state = client.get(f"/api/experiments/{experiment_id}").json()
            if state["experiment"]["status"] == "completed":
                break
            time.sleep(0.01)

    assert service.get(experiment_id)["experiment"]["status"] == "completed"


def test_api_cancel_stops_active_worker_task(client, settings):
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
            "name": "API cancel",
            "agent_ids": ["oracle"],
            "benchmark_names": ["fake-slow-cancel"],
            "n_tasks": 2,
        },
    ).json()
    experiment_id = created["experiment"]["id"]
    run_id = created["runs"][0]["id"]

    client.post(f"/api/experiments/{experiment_id}/run")
    run = client.get(f"/api/runs/{run_id}").json()
    for _ in range(50):
        run = client.get(f"/api/runs/{run_id}").json()
        if run["status"] == "running":
            break
        time.sleep(0.01)

    cancelled = client.post(f"/api/runs/{run_id}/cancel")
    for _ in range(50):
        run = client.get(f"/api/runs/{run_id}").json()
        if run["status"] == "cancelled":
            break
        time.sleep(0.01)

    job_result = settings.experiments_dir / run_id / "harbor-job" / "result.json"
    events = client.get(f"/api/runs/{run_id}/events").json()

    assert cancelled.status_code == 200
    assert run["status"] == "cancelled"
    assert not job_result.exists()
    assert any(
        event["event_type"] == "harbor.job.cancelled"
        and event["payload"]["source"] == "cancelled_during_engine_task_cancel"
        for event in events
    )


@pytest.mark.anyio
async def test_queue_worker_drains_persisted_fifo(settings):
    sqlite.initialize(settings)
    AgentService(settings).create(
        {
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
        }
    )
    service = ExperimentService(settings)
    first = service.create(
        ExperimentCreate(
            name="First",
            agent_ids=["oracle"],
            benchmark_names=["terminal-bench"],
            n_tasks=2,
        )
    )
    second = service.create(
        ExperimentCreate(
            name="Second",
            agent_ids=["oracle"],
            benchmark_names=["swebench-verified"],
            n_tasks=2,
        )
    )
    service.enqueue(first["experiment"]["id"])
    service.enqueue(second["experiment"]["id"])

    worker = QueueWorkerService(settings)
    worker.start()
    await worker.wait_until_idle()

    assert service.get(first["experiment"]["id"])["experiment"]["status"] == "completed"
    assert service.get(second["experiment"]["id"])["experiment"]["status"] == "completed"
    queue = QueueService(settings).list_queue()
    assert [item["state"] for item in queue] == ["completed", "completed"]


@pytest.mark.anyio
async def test_running_cancel_is_not_overwritten_by_worker(settings):
    sqlite.initialize(settings)
    AgentService(settings).create(
        {
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
        }
    )
    service = ExperimentService(settings)
    created = service.create(
        ExperimentCreate(
            name="Cancelable running",
            agent_ids=["oracle"],
            benchmark_names=["fake-slow-cancel"],
            n_tasks=2,
        )
    )
    experiment_id = created["experiment"]["id"]
    run_id = created["runs"][0]["id"]
    service.enqueue(experiment_id)

    worker = QueueWorkerService(settings)
    worker.start()
    for _ in range(50):
        if service.get_run(run_id)["status"] == "running":
            break
        await asyncio.sleep(0.01)

    cancelled = service.cancel_run(run_id)
    assert worker.cancel_run(run_id) is True
    await worker.wait_until_idle()
    final_run = service.get_run(run_id)
    job_result = settings.experiments_dir / run_id / "harbor-job" / "result.json"

    assert cancelled["status"] == "cancelled"
    assert final_run["status"] == "cancelled"
    assert final_run["report_path"].endswith("index.html")
    assert not job_result.exists()
    assert service.get(experiment_id)["experiment"]["status"] == "cancelled"
    events = service.events.list_after(run_id)
    event_types = [event.event_type for event in events]
    assert "experiment.cancel_requested" in event_types
    assert any(
        event.event_type == "harbor.job.cancelled"
        and event.payload["source"] == "cancelled_during_engine_task_cancel"
        for event in events
    )
