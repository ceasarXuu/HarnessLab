from __future__ import annotations

import hashlib
from uuid import uuid4

from harnesslab.models.experiment import ExperimentCreate
from harnesslab.services.clock import now_iso
from harnesslab.services.event_service import EventService
from harnesslab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from harnesslab.services.report_service import ReportService
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class ExperimentService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.events = EventService(settings)
        self.builder = HarborConfigBuilder(settings)
        self.engine = HarborEngine()
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

    async def run(self, experiment_id: str) -> dict:
        state = self.get(experiment_id)
        for run in state["runs"]:
            await self._run_one(run)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("completed", now_iso(), experiment_id),
            )
        self.events.append("experiment", experiment_id, "experiment.completed", {})
        return self.get(experiment_id)

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
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, started_at = ?, job_dir = ?, updated_at = ? "
                "WHERE id = ?",
                ("running", now, job_dir, now, run["id"]),
            )
        self.events.append("run", run["id"], "harbor.job.running", config.model_dump())
        result = await self.engine.run(config)
        report_path = self.reports.write_summary(
            run_id=run["id"],
            status=result["status"],
            job_dir=job_dir,
            score=result.get("score"),
        )
        finished = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, result_path = ?, report_path = ?, "
                "updated_at = ? WHERE id = ?",
                (
                    result["status"],
                    finished,
                    result["result_path"],
                    report_path,
                    finished,
                    run["id"],
                ),
            )
        self.events.append("run", run["id"], "harbor.job.completed", result)


def _kind(agent_count: int, benchmark_count: int) -> str:
    if agent_count > 1:
        return "comparison"
    if benchmark_count > 1:
        return "batch"
    return "single"


def _hash(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]
