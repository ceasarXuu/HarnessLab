from __future__ import annotations

from uuid import uuid4

from fastapi import APIRouter, Query, Request

from ornnlab.services.model_pricing import catalog_pricing

router = APIRouter(prefix="/model-pricing")


@router.get("/preview")
async def get_model_pricing_preview(
    request: Request, model_name: str = Query(alias="modelName", min_length=1)
) -> dict:
    unexpected = set(request.query_params) - {"modelName"}
    if unexpected:
        raise ValueError(f"unsupported query parameters: {sorted(unexpected)}")
    return {
        "data": catalog_pricing(model_name),
        "error": None,
        "meta": {"requestId": getattr(request.state, "request_id", uuid4().hex)},
    }
