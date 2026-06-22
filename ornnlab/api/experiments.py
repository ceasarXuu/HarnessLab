from __future__ import annotations

import asyncio

from fastapi import APIRouter, HTTPException, Request
from fastapi.responses import StreamingResponse

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_service import ExperimentService

router = APIRouter(prefix="/api/experiments", tags=["experiments"])


@router.get("")
def list_experiments(request: Request) -> list[dict]:
    return ExperimentService(request.app.state.settings).list()


@router.post("")
def create_experiment(payload: ExperimentCreate, request: Request) -> dict:
    return ExperimentService(request.app.state.settings).create(payload)


@router.get("/{experiment_id}")
def get_experiment(experiment_id: str, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).get(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.post("/{experiment_id}/run")
async def run_experiment(
    experiment_id: str,
    request: Request,
    wait: bool = False,
) -> dict:
    try:
        service = ExperimentService(request.app.state.settings)
        service.enqueue(experiment_id)
        worker = request.app.state.worker
        worker.start()
        if wait:
            await worker.wait_experiment_terminal(experiment_id)
        return service.get(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.post("/{experiment_id}/cancel")
async def cancel_experiment(experiment_id: str, request: Request) -> dict:
    try:
        result = ExperimentService(request.app.state.settings).cancel_experiment(experiment_id)
        for run in result["runs"]:
            if run["status"] == "cancelled":
                request.app.state.worker.cancel_run(run["id"])
        return result
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.delete("/{experiment_id}")
def delete_experiment(experiment_id: str, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).soft_delete(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc
    except RuntimeError as exc:
        raise HTTPException(status_code=409, detail=str(exc)) from exc


@router.post("/{experiment_id}/clone")
def clone_experiment(experiment_id: str, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).clone(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.post("/{experiment_id}/save-template")
def save_template(experiment_id: str, payload: dict, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).save_template(
            experiment_id,
            payload.get("name"),
        )
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.get("/{experiment_id}/report")
def get_experiment_report(experiment_id: str, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).report(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment report not found") from exc


@router.get("/{experiment_id}/events")
def list_events(experiment_id: str, request: Request, after: int = 0) -> list[dict]:
    service = EventService(request.app.state.settings)
    return [event.model_dump() for event in service.list_after(experiment_id, after)]


TERMINAL_EXPERIMENT_STATUSES = {
    "completed",
    "failed",
    "partially_failed",
    "cancelled",
    "interrupted",
}


def _format_sse(event) -> str:
    return (
        f"id: {event.id}\n"
        f"event: {event.event_type}\n"
        f"data: {event.model_dump_json()}\n\n"
    )


def _experiment_terminal(state: dict) -> bool:
    return state["experiment"]["status"] in TERMINAL_EXPERIMENT_STATUSES


@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    settings = request.app.state.settings

    async def stream():
        cursor = after
        while True:
            if await request.is_disconnected():
                break

            exp_service = ExperimentService(settings)
            try:
                state = exp_service.get(experiment_id)
            except KeyError:
                yield 'event: stream.error\ndata: {"detail":"experiment not found"}\n\n'
                break

            aggregate_ids = [experiment_id, *[run["id"] for run in state["runs"]]]
            service = EventService(settings)
            events = service.list_after_many(aggregate_ids, cursor)
            for event in events:
                cursor = event.id
                yield _format_sse(event)

            if _experiment_terminal(state):
                await asyncio.sleep(0.1)
                service = EventService(settings)
                remaining = service.list_after_many(aggregate_ids, cursor)
                for event in remaining:
                    cursor = event.id
                    yield _format_sse(event)
                yield f"event: stream.end\ndata: {{\"status\": \"{state['experiment']['status']}\"}}\n\n"
                break

            await asyncio.sleep(0.5)

    return StreamingResponse(stream(), media_type="text/event-stream")


@router.get("/{experiment_id}/runs")
def list_experiment_runs(experiment_id: str, request: Request) -> list[dict]:
    return get_experiment(experiment_id, request)["runs"]
