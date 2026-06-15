from __future__ import annotations

from fastapi import APIRouter

router = APIRouter(prefix="/api/benchmarks", tags=["benchmarks"])


@router.get("")
def benchmarks() -> list[dict]:
    return [
        {"name": "terminal-bench", "version": "2.0", "source": "harbor"},
        {"name": "swebench-verified", "version": None, "source": "harbor"},
    ]
