from __future__ import annotations

from uuid import uuid4

from fastapi import APIRouter, Request

from ornnlab.services.webui_job_deletion import WebUiJobDeletionService

router = APIRouter(prefix="/jobs")


@router.delete("/{job_id}")
async def delete_job(job_id: str, request: Request) -> dict:
    if request.query_params:
        raise ValueError("unsupported query parameters")
    result = WebUiJobDeletionService(request.app.state.settings).delete(job_id)
    return {
        "data": result,
        "error": None,
        "meta": {"requestId": getattr(request.state, "request_id", uuid4().hex)},
    }
