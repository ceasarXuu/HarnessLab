from __future__ import annotations

import asyncio
from pathlib import Path
from uuid import uuid4

from harnesslab.models.agent import AgentProfile
from harnesslab.models.experiment import ExperimentCreate
from harnesslab.services.clock import now_iso
from harnesslab.services.event_service import EventService
from harnesslab.services.experiment_utils import (
    derive_experiment_status,
    experiment_kind,
    stable_hash,
    unique_preserving_order,
)
from harnesslab.services.failure_classifier import classify_exception
from harnesslab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from harnesslab.services.profile_compiler import ProfileCompiler
from harnesslab.services.queue_service import QueueService
from harnesslab.services.report_service import ReportService
from harnesslab.services.template_service import TemplateService
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class ExperimentService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.events = EventService(settings)
        self.builder = HarborConfigBuilder(settings)
        self.engine = HarborEngine()
        self.compiler = ProfileCompiler(settings)
        self.queue = QueueService(settings)
        self.reports = ReportService(settings)
        self.templates = TemplateService(settings)

    def create(self, request: ExperimentCreate) -> dict:
        experiment_id = f"exp-{uuid4().hex[:12]}"
        now = now_iso()
        run_specs = [
            (agent_id, benchmark)
            for agent_id in request.agent_ids
            for benchmark in request.benchmark_names
        ]
        kind = experiment_kind(len(request.agent_ids), len(request.benchmark_names))
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
                        stable_hash(agent_id),
                        benchmark_name,
                        request.benchmark_version,
                        request.split,
                        stable_hash(str(request.n_tasks)),
                        request.n_tasks,
                        request.n_attempts,
                        request.n_concurrent,
                        now,
                        now,
                        0 if request.n_tasks == 1 else 1,
                        stable_hash(
                            f"{benchmark_name}:{request.benchmark_version}:{request.split}"
                        ),
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
            return sqlite.rows(
                conn,
                "SELECT * FROM experiments WHERE status != 'deleted' ORDER BY created_at DESC",
            )

    def get_run(self, run_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            runs = sqlite.rows(conn, "SELECT * FROM runs WHERE id = ?", (run_id,))
        if not runs:
            raise KeyError(run_id)
        return runs[0]

    async def run(self, experiment_id: str) -> dict:
        self.enqueue(experiment_id)
        await self.run_queued_until_idle(experiment_id)
        return self.get(experiment_id)

    def enqueue(self, experiment_id: str) -> dict:
        self.get(experiment_id)
        queued = self.queue.enqueue_experiment(experiment_id)
        self.events.append("experiment", experiment_id, "experiment.queued", {})
        return {"experiment": self.get(experiment_id)["experiment"], "queue": queued}

    def dequeue_next_run(self) -> dict | None:
        return self.queue.dequeue_next()

    async def run_queued_until_idle(self, experiment_id: str | None = None) -> None:
        while True:
            run = self.queue.dequeue_next(experiment_id)
            if run is None:
                break
            await self.execute_dequeued_run(run)

    async def execute_dequeued_run(self, run: dict) -> None:
        await self._run_one(run)
        self.finalize_experiment_if_terminal(run["experiment_id"])

    def finalize_experiment_if_terminal(self, experiment_id: str) -> None:
        state = self.get(experiment_id)
        statuses = {run["status"] for run in state["runs"]}
        if statuses.intersection({"draft", "queued", "running"}):
            return
        status = derive_experiment_status(run["status"] for run in state["runs"])
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                (status, now_iso(), experiment_id),
            )
        self.events.append("experiment", experiment_id, f"experiment.{status}", {})

    def cancel_run(self, run_id: str) -> dict:
        run = self.get_run(run_id)
        if run["status"] in {"completed", "failed", "cancelled", "interrupted"}:
            return run
        now = now_iso()
        report_path = None
        if run["status"] == "running":
            job_dir = run["job_dir"] or str(self.settings.experiments_dir / run_id / "harbor-job")
            report_path = self.reports.write_summary(run_id, "cancelled", job_dir, None)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, "
                "report_path = COALESCE(?, report_path), updated_at = ? WHERE id = ?",
                ("cancelled", now, report_path, now, run_id),
            )
            conn.execute(
                "UPDATE queue_items SET state = ?, finished_at = ? WHERE run_id = ?",
                ("cancelled", now, run_id),
            )
        event_type = (
            "experiment.cancel_requested"
            if run["status"] == "running"
            else "experiment.cancelled"
        )
        self.events.append(
            "run",
            run_id,
            event_type,
            {"previous_status": run["status"]},
        )
        self.finalize_experiment_if_terminal(run["experiment_id"])
        return self.get_run(run_id)

    def cancel_experiment(self, experiment_id: str) -> dict:
        state = self.get(experiment_id)
        for run in state["runs"]:
            if run["status"] not in {"completed", "failed", "cancelled", "interrupted"}:
                self.cancel_run(run["id"])
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("cancelled", now_iso(), experiment_id),
            )
        self.events.append("experiment", experiment_id, "experiment.cancelled", {})
        return self.get(experiment_id)

    def soft_delete(self, experiment_id: str) -> dict:
        state = self.get(experiment_id)
        active = [run for run in state["runs"] if run["status"] in {"queued", "running"}]
        if active:
            raise RuntimeError("experiment has queued or running runs")
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("deleted", now_iso(), experiment_id),
            )
        self.events.append("experiment", experiment_id, "experiment.deleted", {})
        return state

    def clone(self, experiment_id: str) -> dict:
        state = self.get(experiment_id)
        runs = state["runs"]
        if not runs:
            raise RuntimeError("experiment has no runs to clone")
        request = ExperimentCreate(
            name=f"{state['experiment']['name']} copy",
            agent_ids=unique_preserving_order(run["agent_id"] for run in runs),
            benchmark_names=unique_preserving_order(run["benchmark_name"] for run in runs),
            benchmark_version=runs[0]["benchmark_version"],
            split=runs[0]["split"],
            n_tasks=runs[0]["n_tasks"],
            n_attempts=runs[0]["n_attempts"],
            n_concurrent=runs[0]["n_concurrent"],
            mode="clone",
        )
        cloned = self.create(request)
        self.events.append(
            "experiment",
            cloned["experiment"]["id"],
            "experiment.cloned",
            {"source_experiment_id": experiment_id},
        )
        return cloned

    def save_template(self, experiment_id: str, name: str | None = None) -> dict:
        state = self.get(experiment_id)
        runs = state["runs"]
        if not runs:
            raise RuntimeError("experiment has no runs to save")
        config = {
            "agent_ids": unique_preserving_order(run["agent_id"] for run in runs),
            "benchmark_names": unique_preserving_order(run["benchmark_name"] for run in runs),
            "benchmark_version": runs[0]["benchmark_version"],
            "split": runs[0]["split"],
            "n_tasks": runs[0]["n_tasks"],
            "n_attempts": runs[0]["n_attempts"],
            "n_concurrent": runs[0]["n_concurrent"],
            "source_experiment_id": experiment_id,
        }
        template = self.templates.create(name or state["experiment"]["name"], config)
        self.events.append(
            "experiment",
            experiment_id,
            "experiment.template_saved",
            {"template_id": template["id"]},
        )
        return template

    def report(self, experiment_id: str) -> dict:
        state = self.get(experiment_id)
        reports = []
        for run in state["runs"]:
            if run["report_path"] is None:
                continue
            reports.append(
                {
                    "run_id": run["id"],
                    "report_path": run["report_path"],
                    "summary": self.reports.read_summary(run["report_path"]),
                }
            )
        if not reports:
            raise KeyError("experiment has no reports")
        return {"experiment": state["experiment"], "reports": reports}

    async def _run_one(self, run: dict) -> None:
        now = now_iso()
        job_dir = str(self.settings.experiments_dir / run["id"] / "harbor-job")
        try:
            config = self.builder.build(
                self._agent_config(run["agent_id"]),
                run["benchmark_name"],
                run["benchmark_version"],
                run["n_tasks"],
                run["n_attempts"],
                run["n_concurrent"],
                job_dir,
                job_name=run["id"],
            )
            snapshot = self.engine.capability_snapshot()
            artifact_paths = self.builder.write_run_artifacts(config, snapshot)
        except Exception as exc:
            await self._mark_run_failed(run, job_dir, exc)
            return
        self._mark_run_running(run, job_dir, config.job_name, now)
        self.events.append(
            "run",
            run["id"],
            "harbor.job.running",
            {
                "config": config.model_dump(),
                "capability": snapshot.model_dump(),
                "artifacts": artifact_paths,
            },
        )
        try:
            result = await self.engine.run(config)
        except asyncio.CancelledError:
            if self._is_run_cancelled(run["id"]):
                self.events.append(
                    "run",
                    run["id"],
                    "harbor.job.cancelled",
                    {"source": "cancelled_during_engine_task_cancel"},
                )
                return
            await self._mark_run_interrupted(
                run,
                job_dir,
                "worker_task_cancelled",
                "worker task was cancelled before Harbor returned",
            )
            return
        except Exception as exc:
            if self._is_run_cancelled(run["id"]):
                self.events.append(
                    "run",
                    run["id"],
                    "harbor.job.cancelled",
                    {"source": "cancelled_during_engine_failure"},
                )
                return
            await self._mark_run_failed(run, job_dir, exc)
            return
        if self._is_run_cancelled(run["id"]):
            self.events.append(
                "run",
                run["id"],
                "harbor.job.cancelled",
                {"source": "cancelled_during_engine_run"},
            )
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
                "score = ?, harbor_job_id = ?, updated_at = ? WHERE id = ?",
                (
                    result["status"],
                    finished,
                    result["result_path"],
                    report_path,
                    result.get("score"),
                    result.get("harbor_job_id"),
                    finished,
                    run["id"],
                ),
            )
        self.queue.finish(run["id"], result["status"])
        self.events.append("run", run["id"], "harbor.job.completed", result)

    def _mark_run_running(
        self,
        run: dict,
        job_dir: str,
        harbor_job_name: str,
        now: str,
    ) -> None:
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, started_at = ?, job_dir = ?, "
                "harbor_job_name = ?, updated_at = ? "
                "WHERE id = ?",
                ("running", now, job_dir, harbor_job_name, now, run["id"]),
            )
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("running", now, run["experiment_id"]),
            )

    async def _mark_run_failed(self, run: dict, job_dir: str, exc: Exception) -> None:
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

    async def _mark_run_interrupted(
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

    def _agent_config(self, agent_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT profile_path FROM agents WHERE id = ?", (agent_id,))
        if not rows:
            raise KeyError(agent_id)
        profile = AgentProfile.model_validate_json(Path(rows[0]["profile_path"]).read_text())
        return self.compiler.compile(profile)["agent_config"]

    def _is_run_cancelled(self, run_id: str) -> bool:
        return self.get_run(run_id)["status"] == "cancelled"
