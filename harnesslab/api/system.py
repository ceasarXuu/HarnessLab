from __future__ import annotations

from fastapi import APIRouter, Request

from harnesslab.services.doctor_service import DoctorService

router = APIRouter(prefix="/api/system", tags=["system"])


@router.get("/status")
def status(request: Request) -> dict:
    return DoctorService(request.app.state.settings).status()


@router.post("/doctor")
def doctor(request: Request) -> dict:
    return DoctorService(request.app.state.settings).status()
