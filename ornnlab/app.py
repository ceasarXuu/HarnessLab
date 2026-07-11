from __future__ import annotations

import asyncio
import logging
from contextlib import asynccontextmanager
from uuid import uuid4

from fastapi import FastAPI
from fastapi.encoders import jsonable_encoder
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse
from starlette.exceptions import HTTPException as StarletteHttpException

from ornnlab.api import webui
from ornnlab.services.queue_service import QueueService
from ornnlab.services.recovery_service import RunRecoveryService
from ornnlab.services.webui_dataset_service import WebUiDatasetService
from ornnlab.services.worker_service import QueueWorkerService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


def create_app(settings: Settings | None = None) -> FastAPI:
    active_settings = settings or Settings.from_env()
    active_settings.ensure_dirs()
    sqlite.initialize(active_settings)
    startup_recovery = RunRecoveryService(active_settings).reconcile_startup()

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        if QueueService(active_settings).queued_count() > 0:
            app.state.worker.start()
        yield
        tasks = list(app.state.operation_tasks.values())
        for task in tasks:
            task.cancel()
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)

    app = FastAPI(title="OrnnLab", version="0.3.0", lifespan=lifespan)
    app.state.settings = active_settings
    app.state.startup_recovery = startup_recovery
    app.state.worker = QueueWorkerService(active_settings)
    app.state.dataset_service = WebUiDatasetService(active_settings)
    app.state.operation_tasks = {}

    @app.middleware("http")
    async def add_request_id(request, call_next):
        request.state.request_id = uuid4().hex
        response = await call_next(request)
        response.headers["X-Request-Id"] = request.state.request_id
        return response

    @app.exception_handler(RequestValidationError)
    async def validation_error(request, exc):
        return _error_response(
            request,
            422,
            "VALIDATION_ERROR",
            "Request validation failed",
            {"errors": jsonable_encoder(exc.errors(), custom_encoder={ValueError: str})},
        )

    @app.exception_handler(StarletteHttpException)
    async def http_error(request, exc):
        return _error_response(
            request,
            exc.status_code,
            "ROUTE_NOT_FOUND" if exc.status_code == 404 else "HTTP_ERROR",
            str(exc.detail),
        )

    @app.exception_handler(KeyError)
    async def missing_resource(request, exc):
        return _error_response(request, 404, "RESOURCE_NOT_FOUND", str(exc.args[0]))

    @app.exception_handler(PermissionError)
    async def forbidden_resource(request, exc):
        return _error_response(request, 403, "RESOURCE_IMMUTABLE", str(exc))

    @app.exception_handler(ValueError)
    async def invalid_resource(request, exc):
        return _error_response(request, 422, "INVALID_REQUEST", str(exc))

    @app.exception_handler(RuntimeError)
    async def conflict_resource(request, exc):
        return _error_response(request, 409, "OPERATION_CONFLICT", str(exc))

    @app.exception_handler(Exception)
    async def unexpected_error(request, exc):
        logger.exception("Unhandled WebUI API exception")
        return _error_response(request, 500, "INTERNAL_ERROR", "Unexpected server error")

    app.include_router(webui.router)
    return app


def _error_response(
    request, status: int, code: str, message: str, details: dict[str, object] | None = None
) -> JSONResponse:
    error: dict[str, object] = {"code": code, "message": message}
    if details:
        error["details"] = details
    return JSONResponse(
        status_code=status,
        content={
            "data": None,
            "error": error,
            "meta": {"requestId": getattr(request.state, "request_id", uuid4().hex)},
        },
    )
