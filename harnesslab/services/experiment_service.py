from __future__ import annotations

import hashlib
from uuid import uuid4

from harnesslab.models.experiment import ExperimentCreate
from harnesslab.services.clock import now_iso
from harnesslab.services.event_service import EventService
from harnesslab.services.failure_classifier import classify_exception
from harnesslab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from harnesslab.services.queue_service import QueueService
from harnesslab.services.report_service import ReportService
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class ExperimentService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.events = EventService(settings)
        self.builder = HarborConfigBuilder(settings)
        self.engine = HarborEngine()
        self.queue = QueueService(settings)
        self.reports = ReportService(settings)

    def create(self, request: ExperimentCreate) -> dict:
        experiment_id = f"exp-{uuid4().hex[:12]}"
        now = now_iso()
        run_specs = [
            (agent_id, benchmark)
            for agent_id in request.agent_ids
            for benchmark in request.benchmark_names
        ]
        kind = _kind(len(request.agent_ids), len(request.benchmark_names))
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "INSERT INTO experiments VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    experiment_id,
                    request.name,
                    kind,
                    "draft",
                    len(run_specs),
                    request.mode,
                    now,
                    now,
                ),
            )
            for index, (agent_id, benchmark_name) in enumerate(run_specs, start=1):
                run_id = f"run-{uuid4().hex[:12]}"
                conn.execute(
                    "INSERT INTO runs("
                    "id, experiment_id, status, run_order, agent_id, agent_snapshot_hash, "
                    "benchmark_name, benchmark_version, split, task_filter_hash, n_tasks, "
                    "n_attempts, n_concurrent, created_at, updated_at, leaderboard_eligible, "
                    "comparability_key"
                    ") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    (
                        run_id,
                        experiment_id,
                        "draft",
                        index,
                        agent_id,
                        _hash(agent_id),
                        benchmark_name,
                        request.benchmark_version,
                        request.split,
                        _hash(str(request.n_tasks)),
                        request.n_tasks,
                        request.n_attempts,
                        request.n_concurrent,
                        now,
                        now,
                        0 if request.n_tasks == 1 else 1,
                        _hash(f"{benchmark_name}:{request.benchmark_version}:{request.split}"),
                    ),
                )
        self.events.append(
            "experiment",
            experiment_id,
            "experiment.created",
            {"run_count": len(run_specs), "kind": kind},
        )
        return self.get(experiment_id)

    def get(self, experiment_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            exp = sqlite.rows(conn, "SELECT * FROM experiments WHERE id = ?", (experiment_id,))
            runs = sqlite.rows(
                conn,
                "SELECT * FROM runs WHERE experiment_id = ? ORDER BY run_order",
                (experiment_id,),
            )
        if not exp:
            raise KeyError(experiment_id)
        return {"experiment": exp[0], "runs": runs}

    def list(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(conn, "SELECT * FROM experiments ORDER BY created_at DESC")

    def get_run(self, run_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            runs = sqlite.rows(conn, "SELECT * FROM runs WHERE id = ?", (run_id,))
        if not runs:
            raise KeyError(run_id)
        return runs[0]

    async def run(self, experiment_id: str) -> dict:
        self.queue.enqueue_experiment(experiment_id)
        self.events.append("experiment", experiment_id, "experiment.queued", {})
        while True:
            run = self.queue.dequeue_next(experiment_id)
            if run is None:
                break
            await self._run_one(run)
        status = self._derive_experiment_status(experiment_id)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                (status, now_iso(), experiment_id),
            )
        self.events.append("experiment", experiment_id, f"experiment.{status}", {})
        return self.get(experiment_id)

    def cancel_run(self, run_id: str) -> dict:
        run = self.get_run(run_id)
        if run["status"] in {"completed", "failed", "cancelled", "interrupted"}:
            return run
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, updated_at = ? WHERE id = ?",
                ("cancelled", now, now, run_id),
            )
            conn.execute(
                "UPDATE queue_items SET state = ?, finished_at = ? WHERE run_id = ?",
                ("cancelled", now, run_id),
            )
        self.events.append(
            "run",
            run_id,
            "experiment.cancelled",
            {"previous_status": run["status"]},
        )
        return self.get_run(run_id)

    async def _run_one(self, run: dict) -> None:
        now = now_iso()
        job_dir = str(self.settings.experiments_dir / run["id"] / "harbor-job")
        config = self.builder.build(
            {"name": run["agent_id"]},
            run["benchmark_name"],
            run["benchmark_version"],
            run["n_tasks"],
            run["n_attempts"],
            run["n_concurrent"],
            job_dir,
        )
        self._mark_run_running(run, job_dir, now)
        self.events.append("run", run["id"], "harbor.job.running", config.model_dump())
        try:
            result = await self.engine.run(config)
        except Exception as exc:
            await self._mark_run_failed(run, job_dir, exc)
            return
        report_path = self.reports.write_summary(
            run["id"],
            result["status"],
            job_dir,
            result.get("score"),
        )
        finished = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, result_path = ?, report_path = ?, "
                "score = ?, updated_at = ? WHERE id = ?",
                (
                    result["status"],
                    finished,
                    result["result_path"],
                    report_path,
                    result.get("score"),
                    finished,
                    run["id"],
                ),
            )
        self.queue.finish(run["id"], result["status"])
        self.events.append("run", run["id"], "harbor.job.completed", result)

    def _mark_run_running(self, run: dict, job_dir: str, now: str) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, started_at = ?, job_dir = ?, updated_at = ? "
                "WHERE id = ?",
                ("running", now, job_dir, now, run["id"]),
            )
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("running", now, run["experiment_id"]),
            )

    async def _mark_run_failed(self, run: dict, job_dir: str, exc: Exception) -> None:
        failure = classify_exception(exc)
        report_path = self.reports.write_summary(run["id"], "failed", job_dir, None)
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

    def _derive_experiment_status(self, experiment_id: str) -> str:
        runs = self.get(experiment_id)["runs"]
        statuses = {run["status"] for run in runs}
        if statuses == {"completed"}:
            return "completed"
        if "completed" in statuses and "failed" in statuses:
            return "partially_failed"
        if "failed" in statuses:
            return "failed"
        if "cancelled" in statuses:
            return "cancelled"
        if "interrupted" in statuses:
            return "interrupted"
        return "queued"


def _kind(agent_count: int, benchmark_count: int) -> str:
    if agent_count > 1:
        return "comparison"
    if benchmark_count > 1:
        return "batch"
    return "single"


def _hash(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]
