from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from harnesslab.services.template_service import TemplateService

router = APIRouter(prefix="/api/templates", tags=["templates"])


@router.get("")
def list_templates(request: Request) -> list[dict]:
    return TemplateService(request.app.state.settings).list()


@router.post("")
def create_template(payload: dict, request: Request) -> dict:
    name = str(payload.get("name", "")).strip()
    config = payload.get("config")
    if not name or not isinstance(config, dict):
        raise HTTPException(status_code=422, detail="name and config object are required")
    return TemplateService(request.app.state.settings).create(name, config)


@router.delete("/{template_id}")
def delete_template(template_id: str, request: Request) -> dict:
    try:
        return TemplateService(request.app.state.settings).soft_delete(template_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="template not found") from exc
