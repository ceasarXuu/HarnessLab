from __future__ import annotations

import asyncio

from ornnlab.services.docker_orphan_service import DockerOrphanService
from ornnlab.services.event_service import EventService
from ornnlab.services.run_docker_cleanup import cleanup_run_docker_resources


def test_run_cleanup_records_success_event(settings, monkeypatch):
    monkeypatch.setattr(
        DockerOrphanService,
        "cleanup_run",
        lambda _self, run_id: {
            "ok": True,
            "run_id": run_id,
            "matched_containers": 1,
            "removed_containers": 1,
            "removed_networks": 1,
            "removed_volumes": 0,
            "projects": ["trial-project"],
            "errors": [],
        },
    )
    events = EventService(settings)

    asyncio.run(
        cleanup_run_docker_resources(
            settings,
            events,
            "run-1",
            {"kwargs": {"ornnlab_cleanup_policy": "auto"}},
        )
    )

    event = events.list_after("run-1")[-1]
    assert event.event_type == "docker.ownership.cleanup_completed"
    assert event.payload["removed_containers"] == 1


def test_run_cleanup_preserves_explicit_retention(settings, monkeypatch):
    monkeypatch.setattr(
        DockerOrphanService,
        "cleanup_run",
        lambda *_args: (_ for _ in ()).throw(AssertionError("cleanup must not run")),
    )
    events = EventService(settings)

    asyncio.run(
        cleanup_run_docker_resources(
            settings,
            events,
            "run-retained",
            {"kwargs": {"ornnlab_cleanup_policy": "retain"}},
        )
    )

    assert events.list_after("run-retained")[-1].event_type == "docker.ownership.retained"
