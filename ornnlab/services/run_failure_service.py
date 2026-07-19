from __future__ import annotations

from ornnlab.services.clock import now_iso
from ornnlab.services.event_service import EventService
from ornnlab.services.failure_classifier import classify_exception
from ornnlab.services.queue_service import QueueService
from ornnlab.services.report_service import ReportService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class RunFailureService:
    def __init__(
        self,
        settings: Settings,
        events: EventService,
        queue: QueueService,
        reports: ReportService,
    ) -> None:
        self.settings = settings
        self.events = events
        self.queue = queue
        self.reports = reports

    async def mark_failed(self, run: dict, job_dir: str, exc: Exception) -> None:
        failure = classify_exception(exc)
        report_path = self.reports.write_summary(
            run["id"],
            "failed",
            job_dir,
            None,
            failure["failure_class"],
            failure["failure_code"],
        )
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, job_dir = ?, report_path = ?, "
                "failure_class = ?, failure_code = ?, failure_summary = ?, updated_at = ? "
                "WHERE id = ?",
                (
                    "failed",
                    now,
                    job_dir,
                    report_path,
                    failure["failure_class"],
                    failure["failure_code"],
                    failure["failure_summary"],
                    now,
                    run["id"],
                ),
            )
        self.queue.finish(run["id"], "failed")
        self.events.append("run", run["id"], "harbor.job.failed", failure, severity="error")

    async def mark_interrupted(
        self,
        run: dict,
        job_dir: str,
        failure_code: str,
        summary: str,
    ) -> None:
        report_path = self.reports.write_summary(
            run["id"],
            "interrupted",
            job_dir,
            None,
            "worker_lifecycle",
            failure_code,
        )
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, job_dir = ?, report_path = ?, "
                "failure_class = ?, failure_code = ?, failure_summary = ?, updated_at = ? "
                "WHERE id = ?",
                (
                    "interrupted",
                    now,
                    job_dir,
                    report_path,
                    "worker_lifecycle",
                    failure_code,
                    summary,
                    now,
                    run["id"],
                ),
            )
        self.queue.finish(run["id"], "interrupted")
        self.events.append(
            "run",
            run["id"],
            "experiment.interrupted",
            {"failure_code": failure_code, "failure_summary": summary},
            severity="warning",
        )
