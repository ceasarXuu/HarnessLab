from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from harnesslab.services.event_service import EventService
from harnesslab.services.experiment_service import ExperimentService

router = APIRouter(prefix="/api/runs", tags=["runs"])


@router.get("/{run_id}")
def get_run(run_id: str, request: Request) -> dict:
    try:
        return ExperimentService(request.app.state.settings).get_run(run_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="run not found") from exc


@router.post("/{run_id}/cancel")
async def cancel_run(run_id: str, request: Request) -> dict:
    try:
        result = ExperimentService(request.app.state.settings).cancel_run(run_id)
        request.app.state.worker.cancel_run(run_id)
        return result
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="run not found") from exc


@router.get("/{run_id}/events")
def list_run_events(run_id: str, request: Request, after: int = 0) -> list[dict]:
    service = EventService(request.app.state.settings)
    return [event.model_dump() for event in service.list_after(run_id, after)]


@router.get("/{run_id}/report")
def get_run_report(run_id: str, request: Request) -> dict:
    service = ExperimentService(request.app.state.settings)
    try:
        run = service.get_run(run_id)
        return {
            "run": run,
            "summary": service.reports.read_summary(run["report_path"]),
        }
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="run report not found") from exc
