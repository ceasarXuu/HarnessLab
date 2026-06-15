from __future__ import annotations

from fastapi import APIRouter, Request

from ornnlab.services.leaderboard_service import LeaderboardService

router = APIRouter(prefix="/api/leaderboard", tags=["leaderboard"])


@router.get("")
def leaderboard(request: Request, benchmark: str | None = None) -> list[dict]:
    return LeaderboardService(request.app.state.settings).list(benchmark=benchmark)
