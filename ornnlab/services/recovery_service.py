from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from ornnlab.services.clock import now_iso
from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_utils import derive_experiment_status
from ornnlab.services.report_service import ReportService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

TERMINAL_STATUSES = {"completed", "failed", "cancelled", "interrupted"}
RECOVERY_FAILURE_CLASS = "harbor_recovery"
STALE_RUNNING_CODE = "stale_running_without_result"


class RunRecoveryService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.events = EventService(settings)
        self.reports = ReportService(settings)

    def reconcile_startup(self) -> dict[str, int]:
        running = self._running_runs()
        orphaned = self._orphaned_queue_items()
        counts = {"recovered": 0, "interrupted": 0}
        experiment_ids: set[str] = set()
        for run in [*running, *orphaned]:
            experiment_ids.add(run["experiment_id"])
            decision = self._reconcile_run(run)
            counts[decision] += 1
        for experiment_id in experiment_ids:
            self._update_experiment_status(experiment_id)
        return counts

    def stale_running_count(self) -> int:
        with sqlite.connect(self.settings) as conn:
            row = conn.execute("SELECT COUNT(*) AS count FROM runs WHERE status = 'running'")
            return int(row.fetchone()["count"])

    def _running_runs(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(
                conn,
                "SELECT * FROM runs WHERE status = 'running' ORDER BY started_at, id",
            )

    def _orphaned_queue_items(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(
                conn,
                "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
                "WHERE q.state = 'running' "
                "AND r.status NOT IN "
                "('running', 'completed', 'failed', 'cancelled', 'interrupted') "
                "ORDER BY q.dequeued_at, r.id",
            )

    def _reconcile_run(self, run: dict) -> str:
        job_dir = self._job_dir(run)
        result_path = self._result_path(run, job_dir)
        if result_path.exists():
            result = self._read_result(result_path)
            status = _status_from_result_payload(result)
            score = _score_from_result_payload(result)
            report_path = self.reports.write_summary(run["id"], status, str(job_dir), score)
            self._mark_terminal(run, status, result_path, report_path, score, None, None)
            self._emit_recovered(run, status, result_path, report_path)
            return "recovered"
        report_path = self.reports.write_summary(
            run["id"],
            "interrupted",
            str(job_dir),
            None,
            RECOVERY_FAILURE_CLASS,
            STALE_RUNNING_CODE,
        )
        self._mark_terminal(
            run,
            "interrupted",
            None,
            report_path,
            None,
            RECOVERY_FAILURE_CLASS,
            STALE_RUNNING_CODE,
        )
        self._emit_interrupted(run, job_dir, report_path)
        return "interrupted"

    def _mark_terminal(
        self,
        run: dict,
        status: str,
        result_path: Path | None,
        report_path: str,
        score: float | None,
        failure_class: str | None,
        failure_code: str | None,
    ) -> None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = COALESCE(finished_at, ?), "
                "result_path = COALESCE(?, result_path), report_path = ?, score = ?, "
                "failure_class = ?, failure_code = ?, updated_at = ? WHERE id = ?",
                (
                    status,
                    now,
                    str(result_path) if result_path else None,
                    report_path,
                    score,
                    failure_class,
                    failure_code,
                    now,
                    run["id"],
                ),
            )
            conn.execute(
                "UPDATE queue_items SET state = ?, finished_at = COALESCE(finished_at, ?) "
                "WHERE run_id = ?",
                (status, now, run["id"]),
            )

    def _update_experiment_status(self, experiment_id: str) -> None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT status FROM runs WHERE experiment_id = ?",
                (experiment_id,),
            )
            status = derive_experiment_status(row["status"] for row in rows)
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                (status, now_iso(), experiment_id),
            )

    def _emit_recovered(
        self,
        run: dict,
        status: str,
        result_path: Path,
        report_path: str,
    ) -> None:
        payload = {
            "decision": "recovered_from_result",
            "status": status,
            "result_path": str(result_path),
            "report_path": report_path,
        }
        self.events.append("run", run["id"], "experiment.reconcile_decision", payload)
        self.events.append(
            "experiment",
            run["experiment_id"],
            "experiment.reconciled",
            {"run_id": run["id"], **payload},
        )

    def _emit_interrupted(self, run: dict, job_dir: Path, report_path: str) -> None:
        payload = {
            "decision": "interrupted_missing_result",
            "status": "interrupted",
            "job_dir": str(job_dir),
            "report_path": report_path,
            "failure_code": STALE_RUNNING_CODE,
        }
        self.events.append(
            "run",
            run["id"],
            "experiment.reconcile_decision",
            payload,
            severity="warning",
        )
        self.events.append(
            "experiment",
            run["experiment_id"],
            "experiment.interrupted",
            {"run_id": run["id"], **payload},
            severity="warning",
        )

    def _job_dir(self, run: dict) -> Path:
        if run["job_dir"]:
            return Path(run["job_dir"])
        return self.settings.experiments_dir / run["id"] / "harbor-job"

    def _result_path(self, run: dict, job_dir: Path) -> Path:
        if run["result_path"]:
            return Path(run["result_path"])
        return job_dir / "result.json"

    @staticmethod
    def _read_result(path: Path) -> dict[str, Any]:
        return json.loads(path.read_text(encoding="utf-8"))


def _status_from_result_payload(result: dict[str, Any]) -> str:
    status = result.get("status")
    if isinstance(status, str) and status in TERMINAL_STATUSES:
        return status
    stats_value = result.get("stats")
    stats: dict[str, Any] = stats_value if isinstance(stats_value, dict) else {}
    if int(stats.get("n_cancelled_trials") or 0) > 0:
        return "cancelled"
    if int(stats.get("n_errored_trials") or 0) > 0:
        return "failed"
    return "completed"


def _score_from_result_payload(result: dict[str, Any]) -> float | None:
    score = result.get("score")
    if isinstance(score, int | float):
        return float(score)
    stats_value = result.get("stats")
    stats: dict[str, Any] = stats_value if isinstance(stats_value, dict) else {}
    evals_value = stats.get("evals")
    evals: dict[str, Any] = evals_value if isinstance(evals_value, dict) else {}
    for dataset_stats in evals.values():
        if not isinstance(dataset_stats, dict):
            continue
        pass_at_k = dataset_stats.get("pass_at_k")
        if isinstance(pass_at_k, dict):
            score = pass_at_k.get("1", pass_at_k.get(1))
            if isinstance(score, int | float):
                return float(score)
    return None


