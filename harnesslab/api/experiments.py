from __future__ import annotations

import asyncio

from fastapi import APIRouter, HTTPException, Request
from fastapi.responses import StreamingResponse

from harnesslab.models.experiment import ExperimentCreate
from harnesslab.services.event_service import EventService
from harnesslab.services.experiment_service import ExperimentService

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
async def run_experiment(experiment_id: str, request: Request) -> dict:
    try:
        return await ExperimentService(request.app.state.settings).run(experiment_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="experiment not found") from exc


@router.get("/{experiment_id}/events")
def list_events(experiment_id: str, request: Request, after: int = 0) -> list[dict]:
    service = EventService(request.app.state.settings)
    return [event.model_dump() for event in service.list_after(experiment_id, after)]


@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    service = EventService(request.app.state.settings)

    async def stream():
        for event in service.list_after(experiment_id, after):
            yield f"id: {event.id}\nevent: {event.event_type}\ndata: {event.model_dump_json()}\n\n"
        await asyncio.sleep(0.01)

    return StreamingResponse(stream(), media_type="text/event-stream")
