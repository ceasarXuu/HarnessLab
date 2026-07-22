from __future__ import annotations

import asyncio

from ornnlab.services.docker_orphan_service import DockerOrphanService
from ornnlab.services.event_service import EventService
from ornnlab.settings import Settings


async def cleanup_run_docker_resources(
    settings: Settings,
    events: EventService,
    run_id: str,
    environment: dict,
) -> None:
    kwargs = environment.get("kwargs") if isinstance(environment, dict) else None
    if isinstance(kwargs, dict) and kwargs.get("ornnlab_cleanup_policy") == "retain":
        events.append(
            "run",
            run_id,
            "docker.ownership.retained",
            {"policy": "retain"},
        )
        return
    result = await asyncio.to_thread(
        DockerOrphanService(instance_id=settings.instance_id).cleanup_run,
        run_id,
    )
    events.append(
        "run",
        run_id,
        "docker.ownership.cleanup_completed"
        if result["ok"]
        else "docker.ownership.cleanup_failed",
        result,
        severity="info" if result["ok"] else "warning",
    )
