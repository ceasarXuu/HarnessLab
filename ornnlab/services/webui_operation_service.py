from __future__ import annotations

import asyncio
import json
import logging
from collections.abc import Awaitable, Callable
from uuid import uuid4

from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

OperationWork = Callable[[Callable[[int | None, str | None], None]], Awaitable[None]]
logger = logging.getLogger(__name__)


class WebUiOperationService:
    def __init__(self, settings: Settings, tasks: dict[str, asyncio.Task[None]]):
        self.settings = settings
        self.tasks = tasks

    def create(
        self, operation_type: str, resource_type: str, resource_id: str | None = None
    ) -> dict:
        operation = {
            "id": f"op-{uuid4().hex[:12]}",
            "type": operation_type,
            "status": "queued",
            "resourceType": resource_type,
            "resourceId": resource_id,
            "progress": 0,
            "message": "Queued",
            "startedAt": None,
            "completedAt": None,
        }
        query = (
            "INSERT INTO webui_operations("
            "id, operation_type, status, resource_type, resource_id, progress, "
            "message, created_at"
            ") VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                query,
                (
                    operation["id"],
                    operation_type,
                    operation["status"],
                    resource_type,
                    resource_id,
                    operation["progress"],
                    operation["message"],
                    now_iso(),
                ),
            )
        return operation

    def reconcile_interrupted(self) -> int:
        """Fail persisted operations that no longer have an in-process task."""
        completed_at = now_iso()
        with sqlite.connect(self.settings) as conn:
            cursor = conn.execute(
                "UPDATE webui_operations SET status = 'failed', message = ?, "
                "completed_at = ?, error_code = ?, error_message = ? "
                "WHERE status IN ('queued', 'running')",
                (
                    "Interrupted by service restart",
                    completed_at,
                    "OPERATION_INTERRUPTED",
                    "The operation was interrupted by a service restart",
                ),
            )
        reconciled = cursor.rowcount
        if reconciled:
            logger.warning("Reconciled interrupted WebUI operations count=%s", reconciled)
        return reconciled

    def get(self, operation_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT * FROM webui_operations WHERE id = ?", (operation_id,))
        if not rows:
            raise KeyError(operation_id)
        return _to_dto(rows[0])

    def submit(
        self, operation_type: str, resource_type: str, resource_id: str | None, work: OperationWork
    ) -> dict:
        operation = self.create(operation_type, resource_type, resource_id)
        logger.info(
            "WebUI operation submitted: id=%s type=%s resource=%s",
            operation["id"],
            operation_type,
            resource_id,
        )
        task = asyncio.create_task(self._execute(operation["id"], work), name=operation["id"])
        self.tasks[operation["id"]] = task
        task.add_done_callback(lambda _: self.tasks.pop(operation["id"], None))
        return operation

    def complete(
        self, operation_type: str, resource_type: str, resource_id: str | None, message: str
    ) -> dict:
        operation = self.create(operation_type, resource_type, resource_id)
        self._set_status(operation["id"], "completed", progress=100, message=message)
        return self.get(operation["id"])

    def fail(
        self,
        operation_type: str,
        resource_type: str,
        resource_id: str | None,
        code: str,
        message: str,
    ) -> dict:
        operation = self.create(operation_type, resource_type, resource_id)
        self._set_status(operation["id"], "failed", message=message, error=(code, message, None))
        return self.get(operation["id"])

    def cancel(self, operation_id: str) -> dict:
        operation = self.get(operation_id)
        if operation["status"] in {"completed", "failed", "cancelled"}:
            raise RuntimeError("operation is already terminal")
        task = self.tasks.get(operation_id)
        if task is not None and not task.done():
            task.cancel()
        self._set_status(operation_id, "cancelled", message="Cancelled")
        return self.get(operation_id)

    def cancel_active(self, operation_type: str, resource_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT id FROM webui_operations WHERE operation_type = ? AND resource_id = ? "
                "AND status IN ('queued', 'running') ORDER BY created_at DESC LIMIT 1",
                (operation_type, resource_id),
            )
        if not rows:
            raise KeyError(resource_id)
        return self.cancel(rows[0]["id"])

    async def _execute(self, operation_id: str, work: OperationWork) -> None:
        self._set_status(operation_id, "running", progress=0, message="Running")

        def progress(value: int | None, message: str | None = None) -> None:
            self._set_progress(operation_id, value, message)

        try:
            await work(progress)
        except asyncio.CancelledError:
            self._set_status(operation_id, "cancelled", message="Cancelled")
            logger.info("WebUI operation cancelled: id=%s", operation_id)
            raise
        except Exception as exc:
            self._set_status(
                operation_id,
                "failed",
                message=str(exc),
                error=("OPERATION_FAILED", str(exc), {"exception": type(exc).__name__}),
            )
            logger.exception("WebUI operation failed: id=%s", operation_id)
        else:
            self._set_status(operation_id, "completed", progress=100, message="Completed")
            logger.info("WebUI operation completed: id=%s", operation_id)

    def _set_progress(self, operation_id: str, progress: int | None, message: str | None) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE webui_operations SET progress = COALESCE(?, progress), "
                "message = COALESCE(?, message) WHERE id = ? AND status IN ('queued', 'running')",
                (progress, message, operation_id),
            )

    def _set_status(
        self,
        operation_id: str,
        status: str,
        progress: int | None = None,
        message: str | None = None,
        error: tuple[str, str, dict | None] | None = None,
    ) -> None:
        started_at = now_iso() if status == "running" else None
        completed_at = now_iso() if status in {"completed", "failed", "cancelled"} else None
        error_code, error_message, error_details = error or (None, None, None)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE webui_operations SET status = ?, progress = COALESCE(?, progress), "
                "message = COALESCE(?, message), started_at = COALESCE(?, started_at), "
                "completed_at = COALESCE(?, completed_at), error_code = ?, error_message = ?, "
                "error_details_json = ? WHERE id = ?",
                (
                    status,
                    progress,
                    message,
                    started_at,
                    completed_at,
                    error_code,
                    error_message,
                    json.dumps(error_details) if error_details else None,
                    operation_id,
                ),
            )


def _to_dto(row: dict) -> dict:
    result = {
        "id": row["id"],
        "type": row["operation_type"],
        "status": row["status"],
        "resourceType": row["resource_type"],
    }
    for source, target in [
        ("resource_id", "resourceId"),
        ("progress", "progress"),
        ("message", "message"),
        ("started_at", "startedAt"),
        ("completed_at", "completedAt"),
    ]:
        if row.get(source) is not None:
            result[target] = row[source]
    if row.get("error_code"):
        result["error"] = {
            "code": row["error_code"],
            "message": row["error_message"],
            "details": json.loads(row["error_details_json"])
            if row.get("error_details_json")
            else None,
        }
    return result
