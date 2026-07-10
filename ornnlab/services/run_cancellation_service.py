from __future__ import annotations

from ornnlab.services.clock import now_iso
from ornnlab.services.event_service import EventService
from ornnlab.services.report_service import ReportService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class RunCancellationService:
    """Persist cancellation state and its observable Harbor-run evidence."""

    def __init__(self, settings: Settings, events: EventService, reports: ReportService):
        self.settings = settings
        self.events = events
        self.reports = reports

    def cancel(self, run: dict) -> None:
        if run["status"] in {"completed", "failed", "cancelled", "interrupted"}:
            return

        now = now_iso()
        report_path = self._cancellation_report(run)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, "
                "report_path = COALESCE(?, report_path), updated_at = ? WHERE id = ?",
                ("cancelled", now, report_path, now, run["id"]),
            )
            conn.execute(
                "UPDATE queue_items SET state = ?, finished_at = ? WHERE run_id = ?",
                ("cancelled", now, run["id"]),
            )

        event_type = (
            "experiment.cancel_requested" if run["status"] == "running" else "experiment.cancelled"
        )
        self.events.append(
            "run",
            run["id"],
            event_type,
            {"previous_status": run["status"]},
        )

    def _cancellation_report(self, run: dict) -> str | None:
        if run["status"] != "running":
            return None
        job_dir = run["job_dir"] or str(self.settings.experiments_dir / run["id"] / "harbor-job")
        return self.reports.write_summary(run["id"], "cancelled", job_dir, None)
