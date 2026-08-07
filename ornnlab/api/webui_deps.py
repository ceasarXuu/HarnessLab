from __future__ import annotations

from uuid import uuid4

from fastapi import Request

from ornnlab.services.webui_dataset_service import WebUiDatasetService
from ornnlab.services.webui_job_service import WebUiJobService
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.services.webui_system_service import WebUiSystemService


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


def _page_data(request: Request, page: dict) -> dict:
    meta = {"requestId": _request_id(request), "total": page["total"]}
    if page.get("nextCursor"):
        meta["nextCursor"] = page["nextCursor"]
    return {"data": page, "error": None, "meta": meta}


def _require_query(request: Request, allowed: set[str]) -> None:
    unsupported = sorted(set(request.query_params) - allowed)
    if unsupported:
        raise ValueError(f"unsupported query parameters: {', '.join(unsupported)}")


def _request_id(request: Request) -> str:
    if not hasattr(request.state, "request_id"):
        request.state.request_id = uuid4().hex
    return request.state.request_id
