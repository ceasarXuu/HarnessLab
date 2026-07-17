from __future__ import annotations

import asyncio
from uuid import uuid4

from fastapi import APIRouter, Request

from ornnlab.models.webui import (
    AgentInput,
    CreateJobInput,
    DatasetImportInput,
    DatasetParentPathInput,
    DatasetPathInput,
    DockerStartCommandInput,
    EnvironmentInput,
    LeaderboardUpdateInput,
)
from ornnlab.services.webui_dataset_service import WebUiDatasetService
from ornnlab.services.webui_job_service import WebUiJobService
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.services.webui_system_service import WebUiSystemService

router = APIRouter(prefix="/api/webui/v1", tags=["webui"])

@router.get("/operations/{operation_id}")
async def get_operation(operation_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _operations(request).get(operation_id))


@router.post("/operations/{operation_id}/cancel")
async def cancel_operation(operation_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _operations(request).cancel(operation_id)})


@router.get("/agents")
async def list_agents(
    request: Request,
    q: str | None = None,
    status: str | None = None,
    cursor: str | None = None,
    limit: int = 50,
) -> dict:
    _require_query(request, {"q", "status", "cursor", "limit"})
    return _page(request, _profiles(request).list_agents(q, status), cursor, limit)


@router.get("/harnesses")
async def list_harnesses(
    request: Request,
    q: str | None = None,
    cursor: str | None = None,
    limit: int = 50,
) -> dict:
    _require_query(request, {"q", "cursor", "limit"})
    return _page(request, _profiles(request).list_harnesses(q), cursor, limit)


@router.get("/agents/{agent_id}")
async def get_agent(agent_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _profiles(request).get_agent(agent_id))


@router.post("/agents")
async def create_agent(payload: AgentInput, request: Request) -> dict:
    _require_query(request, set())
    agent = _profiles(request).create_agent(payload)
    operation = _operations(request).complete("create-agent", "agent", agent["id"], "Agent created")
    return _data(request, {"operation": operation})


@router.patch("/agents/{agent_id}")
async def update_agent(agent_id: str, payload: AgentInput, request: Request) -> dict:
    _require_query(request, set())
    agent = _profiles(request).update_agent(agent_id, payload)
    operation = _operations(request).complete("update-agent", "agent", agent["id"], "Agent updated")
    return _data(request, {"operation": operation})


@router.delete("/agents/{agent_id}")
async def delete_agent(agent_id: str, request: Request) -> dict:
    _require_query(request, set())
    _profiles(request).delete_agent(agent_id)
    operation = _operations(request).complete("delete-agent", "agent", agent_id, "Agent deleted")
    return _data(request, {"operation": operation})


@router.get("/environments")
async def list_environments(
    request: Request,
    q: str | None = None,
    type: str | None = None,
    cursor: str | None = None,
    limit: int = 50,
) -> dict:
    _require_query(request, {"q", "type", "cursor", "limit"})
    return _page(request, _profiles(request).list_environments(q, type), cursor, limit)


@router.get("/environments/{environment_id}")
async def get_environment(environment_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _profiles(request).get_environment(environment_id))


@router.post("/environments")
async def create_environment(payload: EnvironmentInput, request: Request) -> dict:
    _require_query(request, set())
    environment = _profiles(request).create_environment(payload)
    operation = _operations(request).complete(
        "create-environment", "environment", environment["id"], "Environment created"
    )
    return _data(request, {"operation": operation})


@router.patch("/environments/{environment_id}")
async def update_environment(
    environment_id: str, payload: EnvironmentInput, request: Request
) -> dict:
    _require_query(request, set())
    environment = _profiles(request).update_environment(environment_id, payload)
    operation = _operations(request).complete(
        "update-environment", "environment", environment["id"], "Environment updated"
    )
    return _data(request, {"operation": operation})


@router.delete("/environments/{environment_id}")
async def delete_environment(environment_id: str, request: Request) -> dict:
    _require_query(request, set())
    _profiles(request).delete_environment(environment_id)
    operation = _operations(request).complete(
        "delete-environment", "environment", environment_id, "Environment deleted"
    )
    return _data(request, {"operation": operation})


@router.post("/environments/{environment_id}/copy")
async def copy_environment(environment_id: str, request: Request) -> dict:
    _require_query(request, set())
    copied = _profiles(request).copy_environment(environment_id)
    operation = _operations(request).complete(
        "copy-environment", "environment", copied["id"], "Environment copied"
    )
    return _data(request, {"operation": operation})


@router.get("/jobs")
async def list_jobs(
    request: Request, q: str | None = None, cursor: str | None = None, limit: int = 50
) -> dict:
    _require_query(request, {"q", "cursor", "limit"})
    return _page(request, _jobs(request).list_jobs(q), cursor, limit)


@router.get("/jobs/{job_id}")
async def get_job(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _jobs(request).get_job(job_id))


@router.get("/jobs/{job_id}/copy-config")
async def get_job_copy_config(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _jobs(request).copy_job_config(job_id))


@router.post("/jobs")
async def create_job(payload: CreateJobInput, request: Request) -> dict:
    _require_query(request, set())
    job, operation = _jobs(request).create_job(payload)
    return _data(request, {"job": job, "operation": operation})


@router.post("/jobs/{job_id}/cancel")
async def cancel_job(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _jobs(request).cancel_job(job_id)})


@router.post("/jobs/{job_id}/resume")
async def resume_job(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _jobs(request).resume_job(job_id)})


@router.get("/jobs/{job_id}/events")
async def list_job_events(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _jobs(request).events_for_job(job_id))


@router.get("/jobs/{job_id}/trials")
async def list_job_trials(job_id: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _jobs(request).trials_for_job(job_id))


@router.patch("/jobs/{job_id}/leaderboard")
async def update_job_leaderboard(
    job_id: str, payload: LeaderboardUpdateInput, request: Request
) -> dict:
    _require_query(request, set())
    job, operation, entries = _jobs(request).update_leaderboard(
        job_id, payload.include_in_leaderboard
    )
    return _data(request, {"job": job, "operation": operation, "leaderboard": entries})


@router.get("/datasets")
async def list_datasets(
    request: Request, q: str | None = None, cursor: str | None = None, limit: int = 50
) -> dict:
    _require_query(request, {"q", "cursor", "limit"})
    return _page(request, await _datasets(request).list_datasets(q), cursor, limit)


@router.get("/datasets/{dataset_ref:path}/tasks")
async def list_dataset_tasks(
    dataset_ref: str,
    request: Request,
    q: str | None = None,
    cursor: str | None = None,
    limit: int = 50,
) -> dict:
    _require_query(request, {"q", "cursor", "limit"})
    return _page(request, await _datasets(request).list_tasks(dataset_ref, q), cursor, limit)


@router.get("/datasets/{dataset_ref:path}/task-environment")
async def get_dataset_task_environment(dataset_ref: str, task: str, request: Request) -> dict:
    _require_query(request, {"task"})
    return _data(request, await _datasets(request).get_task_environment(dataset_ref, task))


@router.post("/datasets/import")
async def import_dataset(payload: DatasetImportInput, request: Request) -> dict:
    _require_query(request, set())
    datasets = _datasets(request)

    async def work(progress) -> None:
        await datasets.import_local(payload, progress)

    operation = _operations(request).submit(
        "import-dataset", "dataset", f"{payload.name}@{payload.version}", work
    )
    return _data(request, {"operation": operation})


@router.get("/datasets/storage/default-parent")
async def dataset_default_parent(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, _datasets(request).default_download_parent())


@router.post("/datasets/{dataset_ref:path}/download")
async def download_dataset(
    dataset_ref: str, payload: DatasetParentPathInput, request: Request
) -> dict:
    _require_query(request, set())
    datasets = _datasets(request)

    async def work(progress) -> None:
        await datasets.download(dataset_ref, payload.parent_path, progress)

    operation = _operations(request).submit("download-dataset", "dataset", dataset_ref, work)
    return _data(request, {"operation": operation})


@router.post("/datasets/{dataset_ref:path}/download/cancel")
async def cancel_dataset_download(dataset_ref: str, request: Request) -> dict:
    _require_query(request, set())
    try:
        operation = _operations(request).cancel_active("download-dataset", dataset_ref)
    except KeyError:
        raise ValueError("no active dataset download") from None
    _datasets(request).cancel_download(dataset_ref)
    return _data(request, {"operation": operation})


@router.delete("/datasets/{dataset_ref:path}/local")
async def delete_local_dataset(dataset_ref: str, request: Request) -> dict:
    _require_query(request, set())
    _datasets(request).delete_local(dataset_ref)
    operation = _operations(request).complete(
        "delete-local-dataset", "dataset", dataset_ref, "Local dataset removed"
    )
    return _data(request, {"operation": operation})


@router.delete("/datasets/{dataset_ref:path}/registration")
async def remove_dataset_registration(dataset_ref: str, request: Request) -> dict:
    _require_query(request, set())
    _datasets(request).remove_registration(dataset_ref)
    operation = _operations(request).complete(
        "remove-dataset-registration", "dataset", dataset_ref, "Dataset registration removed"
    )
    return _data(request, {"operation": operation})


@router.post("/datasets/{dataset_ref:path}/move")
async def move_dataset(
    dataset_ref: str, payload: DatasetParentPathInput, request: Request
) -> dict:
    _require_query(request, set())
    datasets = _datasets(request)

    async def work(progress) -> None:
        progress(10, "Moving Dataset")
        await asyncio.to_thread(datasets.move, dataset_ref, payload.parent_path)
        progress(100, "Dataset moved")

    operation = _operations(request).submit("move-dataset", "dataset", dataset_ref, work)
    return _data(request, {"operation": operation})


@router.post("/datasets/{dataset_ref:path}/relocate")
async def relocate_dataset(dataset_ref: str, payload: DatasetPathInput, request: Request) -> dict:
    _require_query(request, set())
    _datasets(request).relocate(dataset_ref, payload.path)
    operation = _operations(request).complete(
        "relocate-dataset", "dataset", dataset_ref, "Dataset path updated"
    )
    return _data(request, {"operation": operation})


@router.post("/datasets/{dataset_ref:path}/sync")
async def sync_dataset(dataset_ref: str, request: Request) -> dict:
    _require_query(request, set())
    datasets = _datasets(request)

    async def work(progress) -> None:
        await datasets.sync(dataset_ref, progress)

    operation = _operations(request).submit("sync-dataset", "dataset", dataset_ref, work)
    return _data(request, {"operation": operation})


@router.get("/datasets/{dataset_ref:path}")
async def get_dataset(dataset_ref: str, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, await _datasets(request).get_dataset(dataset_ref))


@router.get("/leaderboard/datasets")
async def leaderboard_datasets(
    request: Request, q: str | None = None, cursor: str | None = None, limit: int = 50
) -> dict:
    _require_query(request, {"q", "cursor", "limit"})
    values = _jobs(request).leaderboard_datasets()
    if q:
        values = [item for item in values if q.lower() in item["ref"].lower()]
    return _page(request, values, cursor, limit)


@router.get("/leaderboard")
async def leaderboard(
    request: Request,
    dataset: str,
    q: str | None = None,
    metric: str | None = None,
    cursor: str | None = None,
    limit: int = 50,
) -> dict:
    _require_query(request, {"dataset", "q", "metric", "cursor", "limit"})
    return _page(request, _jobs(request).leaderboard(dataset, q, metric), cursor, limit)


@router.get("/system/health")
async def system_health(request: Request) -> dict:
    _require_query(request, set())
    items = await asyncio.to_thread(_system(request).health)
    return _page(request, items, None, 100)


@router.get("/system/live")
async def system_live(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"status": "ok"})


@router.get("/system/hub-connection")
async def hub_connection(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, await _system(request).hub_connection())


@router.post("/system/service/update/check")
async def check_update(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, await _system(request).check_update())


@router.post("/system/service/update")
async def install_update(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _system(request).install_update()})


@router.post("/system/service/restart")
async def restart_service(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _system(request).restart()})


@router.post("/system/cache/docker/clean")
async def clean_docker_cache(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _system(request).clean_docker_cache()})


@router.post("/system/cache/storage/clean")
async def clean_storage_cache(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _system(request).clean_storage_cache()})


@router.put("/system/docker/start-command")
async def save_docker_start_command(
    payload: DockerStartCommandInput, request: Request
) -> dict:
    _require_query(request, set())
    return _data(request, _system(request).save_docker_start_command(payload.command))


@router.post("/system/docker/start")
async def start_docker(payload: DockerStartCommandInput, request: Request) -> dict:
    _require_query(request, set())
    return _data(request, {"operation": _system(request).start_docker(payload.command)})


@router.post("/system/directory-picker")
async def choose_directory(request: Request) -> dict:
    _require_query(request, set())
    return _data(request, await asyncio.to_thread(_system(request).choose_directory))


def _operations(request: Request) -> WebUiOperationService:
    return WebUiOperationService(request.app.state.settings, request.app.state.operation_tasks)


def _profiles(request: Request) -> WebUiProfileService:
    return WebUiProfileService(request.app.state.settings)


def _jobs(request: Request) -> WebUiJobService:
    return WebUiJobService(
        request.app.state.settings, _operations(request), request.app.state.worker
    )


def _datasets(request: Request) -> WebUiDatasetService:
    return request.app.state.dataset_service


def _system(request: Request) -> WebUiSystemService:
    return WebUiSystemService(request.app.state.settings, _operations(request))


def _data(request: Request, data: object) -> dict:
    return {"data": data, "error": None, "meta": {"requestId": _request_id(request)}}


def _page(request: Request, items: list[dict], cursor: str | None, limit: int) -> dict:
    offset = int(cursor or "0")
    page = items[offset : offset + limit]
    next_cursor = str(offset + limit) if offset + limit < len(items) else None
    meta = {"requestId": _request_id(request), "total": len(items)}
    if next_cursor:
        meta["nextCursor"] = next_cursor
    return {
        "data": {"items": page, "total": len(items), "nextCursor": next_cursor},
        "error": None,
        "meta": meta,
    }


def _require_query(request: Request, allowed: set[str]) -> None:
    unsupported = sorted(set(request.query_params) - allowed)
    if unsupported:
        raise ValueError(f"unsupported query parameters: {', '.join(unsupported)}")


def _request_id(request: Request) -> str:
    if not hasattr(request.state, "request_id"):
        request.state.request_id = uuid4().hex
    return request.state.request_id
