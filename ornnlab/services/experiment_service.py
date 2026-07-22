from __future__ import annotations

import asyncio
import json
from pathlib import Path
from uuid import uuid4

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.agent_config_service import AgentConfigService
from ornnlab.services.clock import now_iso
from ornnlab.services.container_proxy_runtime import (
    ContainerProxyRuntime,
    RuntimeProxyPolicy,
)
from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_utils import (
    derive_experiment_status,
    experiment_kind,
    stable_hash,
    unique_preserving_order,
)
from ornnlab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from ornnlab.services.harbor_event_payloads import harbor_running_event_payload
from ornnlab.services.queue_service import QueueService
from ornnlab.services.report_service import ReportService
from ornnlab.services.run_cancellation_service import RunCancellationService
from ornnlab.services.run_docker_cleanup import cleanup_run_docker_resources
from ornnlab.services.run_failure_service import RunFailureService
from ornnlab.services.template_service import TemplateService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class ExperimentService:
    def __init__(
        self,
        settings: Settings,
        container_proxy: ContainerProxyRuntime | None = None,
    ):
        self.settings = settings
        self.events = EventService(settings)
        self.builder = HarborConfigBuilder(settings)
        self.engine = HarborEngine()
        self.agent_configs = AgentConfigService(settings)
        self.queue = QueueService(settings)
        self.reports = ReportService(settings)
        self.cancellations = RunCancellationService(settings, self.events, self.reports)
        self.failures = RunFailureService(settings, self.events, self.queue, self.reports)
        self.templates = TemplateService(settings)
        self.container_proxy = container_proxy or ContainerProxyRuntime()

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

    def dequeue_next_run(self, experiment_id: str | None = None) -> dict | None:
        return self.queue.dequeue_next(experiment_id)

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
        self.cancellations.cancel(run)
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
        webui_config = self._webui_run_config(run["id"])
        job_dir = _resolve_job_dir(
            webui_config.get("jobs_dir"), self.settings.experiments_dir / run["id"] / "harbor-job"
        )
        proxy_policy = RuntimeProxyPolicy({}, {}, 0)
        try:
            overrides = webui_config.get("harbor_overrides") or {}
            agent_config = self.agent_configs.config(run["agent_id"], webui_config.get("model"))
            if _uses_docker_environment(overrides):
                proxy_policy = await self.container_proxy.prepare_policy(
                    _explicit_proxy_names(agent_config, overrides),
                    automatic_proxy_allowed=_automatic_proxy_allowed(
                        agent_config, overrides
                    ),
                )
            if proxy_policy.container_env_defaults:
                self.events.append(
                    "run",
                    run["id"],
                    "docker.proxy.injected",
                    {
                        "variable_names": sorted(proxy_policy.container_env_defaults),
                        "relay_count": proxy_policy.relay_count,
                        "strategy": proxy_policy.strategy,
                        "target_kind": proxy_policy.target_kind,
                    },
                )
            if agent_config.get("import_path"):
                agent_config.pop("name", None)
            config = self.builder.build(
                agent_config,
                run["benchmark_name"],
                run["benchmark_version"],
                run["n_tasks"],
                run["n_attempts"],
                run["n_concurrent"],
                job_dir,
                job_name=webui_config.get("job_name", run["id"]),
                overrides=overrides,
                runtime_container_env_defaults=proxy_policy.container_env_defaults,
                owner_run_id=run["id"],
            )
            snapshot = self.engine.capability_snapshot()
            artifact_paths = self.builder.write_run_artifacts(config, snapshot)
        except Exception as exc:
            await proxy_policy.close()
            await self.failures.mark_failed(run, job_dir, exc)
            return
        if not self._mark_run_running(run, job_dir, config.job_name, now):
            await proxy_policy.close()
            self.events.append(
                "run",
                run["id"],
                "harbor.job.cancelled",
                {"source": "cancelled_before_mark_running"},
                severity="warning",
            )
            return
        self.events.append(
            "run",
            run["id"],
            "harbor.job.running",
            harbor_running_event_payload(config, snapshot, artifact_paths),
        )
        try:
            result = await self.engine.run(config, runtime_env=proxy_policy.subprocess_env)
        except asyncio.CancelledError:
            if self._is_run_cancelled(run["id"]):
                self.events.append(
                    "run",
                    run["id"],
                    "harbor.job.cancelled",
                    {"source": "cancelled_during_engine_task_cancel"},
                )
                return
            await self.failures.mark_interrupted(
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
            await self.failures.mark_failed(run, job_dir, exc)
            return
        finally:
            await proxy_policy.close()
            await cleanup_run_docker_resources(
                self.settings, self.events, run["id"], config.environment
            )
        report_path = self.reports.write_summary(
            run["id"],
            result["status"],
            job_dir,
            result.get("score"),
        )
        finished = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET "
                "result_path = COALESCE(result_path, ?), "
                "score = COALESCE(score, ?), "
                "harbor_job_id = COALESCE(harbor_job_id, ?), "
                "updated_at = ? "
                "WHERE id = ?",
                (
                    result["result_path"],
                    result.get("score"),
                    result.get("harbor_job_id"),
                    finished,
                    run["id"],
                ),
            )
            cursor = conn.execute(
                "UPDATE runs SET status = ?, "
                "finished_at = COALESCE(finished_at, ?), "
                "report_path = COALESCE(report_path, ?), "
                "updated_at = ? "
                "WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
                (result["status"], finished, report_path, finished, run["id"]),
            )
            updated = cursor.rowcount
        if updated == 0:
            self.events.append(
                "run",
                run["id"],
                "harbor.job.completed_but_cancelled",
                {
                    "score": result.get("score"),
                    "status": result["status"],
                    "result_path": result["result_path"],
                    "report_path": report_path,
                },
                severity="warning",
            )
        else:
            self.queue.finish(run["id"], result["status"])
            self.events.append("run", run["id"], "harbor.job.completed", result)

    def _mark_run_running(
        self,
        run: dict,
        job_dir: str,
        harbor_job_name: str,
        now: str,
    ) -> bool:
        with sqlite.connect(self.settings) as conn:
            cursor = conn.execute(
                "UPDATE runs SET status = ?, started_at = ?, job_dir = ?, "
                "harbor_job_name = ?, updated_at = ? "
                "WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
                ("running", now, job_dir, harbor_job_name, now, run["id"]),
            )
            if cursor.rowcount == 0:
                return False
            conn.execute(
                "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                ("running", now, run["experiment_id"]),
            )
            return True

    def _is_run_cancelled(self, run_id: str) -> bool:
        return self.get_run(run_id)["status"] == "cancelled"

    def _webui_run_config(self, run_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM webui_job_configs WHERE run_id = ?",
                (run_id,),
            )
        return json.loads(rows[0]["config_json"]) if rows else {}


def _resolve_job_dir(configured_path: str | None, default_path) -> str:
    if not configured_path:
        return str(default_path)
    return str(Path(configured_path).expanduser().resolve())


def _uses_docker_environment(overrides: dict) -> bool:
    environment = overrides.get("environment", {"type": "docker"})
    return (
        isinstance(environment, dict)
        and not environment.get("import_path")
        and environment.get("type", "docker") == "docker"
    )


def _explicit_proxy_names(agent_config: dict, overrides: dict) -> set[str]:
    names: set[str] = set()
    agent_env = agent_config.get("env")
    if isinstance(agent_env, dict):
        names.update(str(name) for name in agent_env)
    environment = overrides.get("environment")
    if isinstance(environment, dict) and isinstance(environment.get("env"), dict):
        names.update(str(name) for name in environment["env"])
    return names


def _automatic_proxy_allowed(agent_config: dict, overrides: dict) -> bool:
    if agent_config.get("extra_allowed_hosts"):
        return False
    environment = overrides.get("environment")
    return not (
        isinstance(environment, dict) and environment.get("extra_allowed_hosts")
    )
