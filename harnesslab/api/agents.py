from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request

from harnesslab.services.agent_service import AgentService

router = APIRouter(prefix="/api/agents", tags=["agents"])


@router.get("")
def list_agents(request: Request) -> list[dict]:
    return AgentService(request.app.state.settings).list()


@router.post("")
def create_agent(payload: dict, request: Request) -> dict:
    return AgentService(request.app.state.settings).create(payload)


@router.get("/{agent_id}")
def get_agent(agent_id: str, request: Request) -> dict:
    try:
        return AgentService(request.app.state.settings).get(agent_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="agent not found") from exc


@router.post("/{agent_id}/compile")
def compile_agent(agent_id: str, request: Request) -> dict:
    try:
        return AgentService(request.app.state.settings).compile(agent_id)
    except KeyError as exc:
        raise HTTPException(status_code=404, detail="agent not found") from exc


@router.post("/validate")
def validate_agent(payload: dict, request: Request) -> dict:
    return AgentService(request.app.state.settings).validate(payload)
