from __future__ import annotations

import asyncio
import json
import os
from pathlib import Path

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.models.webui import CreateJobInput
from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.services.harbor_paths import resolve_harbor_job_path
from ornnlab.services.harbor_results import (
    pending_trial_dto,
    running_trial_descriptors,
    running_trial_dto,
    trial_dir_epoch,
    trial_dto,
    trial_result_payloads,
    trial_start_epoch,
)
from ornnlab.services.harbor_subprocess import harbor_cli_executable
from ornnlab.services.model_pricing import pricing_snapshot
from ornnlab.services.queue_service import QueueService
from ornnlab.services.recovery_service import RunRecoveryService
from ornnlab.services.webui_dataset_service import WebUiDatasetService
from ornnlab.services.webui_job_copy import load_job_copy_config
from ornnlab.services.webui_job_dto import (
    dataset_ref,
    event_level,
    exception_list,
    job_dto,
    join_ref,
)
from ornnlab.services.webui_job_leaderboard import leaderboard, leaderboard_datasets
from ornnlab.services.webui_job_logs import job_log_payload
from ornnlab.services.webui_job_progress import TERMINAL_STATUSES
from ornnlab.services.webui_job_query import JOB_SELECT
from ornnlab.services.webui_job_resume import (
    active_resume_operation,
    agent_env,
    cleanup_resume_leftovers,
    clear_stale_job_lock,
    environment_env,
    failed_trial_error_types,
    mark_resume_failed,
    mark_resume_running,
    prepare_resume_proxy,
    restore_sensitive_env,
    resume_error_tail,
)
from ornnlab.services.webui_job_runtime import load_job_result
from ornnlab.services.webui_job_tasks import pending_task_names
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class WebUiJobService:
    def __init__(
        self,
        settings: Settings,
        operations: WebUiOperationService,
        worker,
    ):
        self.settings = settings
        self.operations = operations
        self.worker = worker
        self.experiments = ExperimentService(settings)
        self.profiles = WebUiProfileService(settings)
        self.events = EventService(settings)

    def list_jobs(self, query: str | None = None) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                JOB_SELECT + "WHERE experiments.status != 'deleted' ORDER BY runs.created_at DESC",
            )
        jobs = [job_dto(row) for row in rows]
        if not query:
            return jobs
        needle = query.lower()
        return [
            job for job in jobs if needle in " ".join(str(value) for value in job.values()).lower()
        ]

    def get_job(self, job_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, JOB_SELECT + "WHERE runs.id = ?", (job_id,))
        if not rows:
            raise KeyError(job_id)
        return job_dto(rows[0])

    def copy_job_config(self, job_id: str) -> dict:
        return load_job_copy_config(self.settings, job_id)

    async def create_job(self, request: CreateJobInput) -> tuple[dict, dict]:
        config = request.config
        agent = self.profiles.resolve_agent(config.agent_name)
        if config.model_name not in agent["models"]:
            raise ValueError("selected model is not configured for this Agent")
        pricing = pricing_snapshot(agent, config.model_name)
        environment = self.profiles.get_environment(config.environment_preset_id)
        benchmark_name, benchmark_version = dataset_ref(config.dataset_ref)
        selected_tasks = config.selected_task_names
        dataset_download_dir = await WebUiDatasetService(self.settings).register_dataset_for_job(
            config.dataset_ref
        )
        created = self.experiments.create(
            ExperimentCreate(
                name=config.job_name,
                agent_ids=[agent["id"]],
                benchmark_names=[benchmark_name],
                benchmark_version=benchmark_version,
                n_tasks=len(selected_tasks) if selected_tasks is not None else None,
                n_attempts=config.attempts,
                n_concurrent=config.concurrency,
                mode="webui",
            )
        )
        run = created["runs"][0]
        overrides = {
            "task_names": selected_tasks,
            "timeout_multiplier": config.timeout_multiplier,
            "agent_timeout_multiplier": config.agent_timeout_multiplier,
            "verifier_timeout_multiplier": config.verifier_timeout_multiplier,
            "agent_setup_timeout_multiplier": config.agent_setup_timeout_multiplier,
            "environment_build_timeout_multiplier": config.environment_build_timeout_multiplier,
            "extra_instruction_paths": config.extra_instruction_paths,
            "debug": config.debug,
            "retry": {
                "max_retries": config.max_retries,
                "include_exceptions": exception_list(config.retry_include),
                "exclude_exceptions": exception_list(config.retry_exclude),
                "wait_multiplier": config.retry_wait_multiplier,
                "min_wait_sec": config.retry_min_wait_seconds,
                "max_wait_sec": config.retry_max_wait_seconds,
            },
            "verifier": {"disable": config.verifier_mode == "skip"},
            "metrics": [{"type": config.metric}],
            "environment": self.profiles.environment_harbor_config(environment),
        }
        if dataset_download_dir:
            overrides["dataset_download_dir"] = dataset_download_dir
        stored = {
            "agent_harness": agent["harness"],
            "agent_name": agent["agentName"],
            "environment_name": environment["name"],
            "environment_preset_id": environment["id"],
            "job_name": config.job_name,
            "jobs_dir": config.jobs_dir,
            "harbor_overrides": overrides,
            "model": config.model_name,
            "pricing": pricing,
        }
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "INSERT INTO webui_job_configs("
                "run_id, config_json, notes, environment_preset_id"
                ") VALUES (?, ?, ?, ?)",
                (run["id"], json.dumps(stored), config.notes, environment["id"]),
            )
            conn.execute(
                "UPDATE runs SET leaderboard_eligible = ? WHERE id = ?",
                (int(config.include_in_leaderboard and config.verifier_mode != "skip"), run["id"]),
            )
        self.events.append(
            "run",
            run["id"],
            "webui.job.configured",
            {
                "agent_name": agent["agentName"],
                "model_name": config.model_name,
                "pricing_source": pricing["source"],
            },
        )
        if request.run_immediately:
            QueueService(self.settings).enqueue_experiment(created["experiment"]["id"])
            self.worker.start()
            operation = self.operations.complete("run-job", "job", run["id"], "Job queued")
        else:
            operation = self.operations.complete("create-job", "job", run["id"], "Job created")
        return self.get_job(run["id"]), operation

    def cancel_job(self, job_id: str) -> dict:
        existing = self.experiments.get_run(job_id)
        if existing["status"] in {"completed", "failed", "cancelled", "interrupted"}:
            raise RuntimeError("job is already terminal")
        run = self.experiments.cancel_run(job_id)
        self.worker.cancel_run(job_id)
        return self.operations.complete("cancel-job", "job", run["id"], "Job cancelled")

    def resume_job(self, job_id: str) -> dict:
        run = self.experiments.get_run(job_id)
        if run["status"] not in {"failed", "interrupted"}:
            raise ValueError("only failed or interrupted jobs can be resumed")
        if active_resume_operation(self.settings, job_id):
            raise ValueError("Job resume is already in progress")
        if not run.get("job_dir"):
            raise ValueError("Harbor job directory is unavailable for resume")
        job_path = resolve_harbor_job_path(Path(run["job_dir"]), run.get("harbor_job_name"))
        if not job_path.is_dir() or not (job_path / "config.json").is_file():
            raise ValueError("Harbor job directory is unavailable for resume")
        clear_stale_job_lock(self.settings, job_id, job_path, run.get("harbor_job_name"))
        return self._resume_operation(run, job_path, operation_type="resume-job")

    def rerun_failed_job(self, job_id: str) -> dict:
        run = self.experiments.get_run(job_id)
        if run["status"] not in TERMINAL_STATUSES:
            raise ValueError("only terminal jobs can re-run failed tasks")
        if active_resume_operation(self.settings, job_id):
            raise ValueError("Job resume is already in progress")
        if not run.get("job_dir"):
            raise ValueError("Harbor job directory is unavailable for re-run")
        job_path = resolve_harbor_job_path(Path(run["job_dir"]), run.get("harbor_job_name"))
        if not job_path.is_dir() or not (job_path / "config.json").is_file():
            raise ValueError("Harbor job directory is unavailable for re-run")
        error_types = failed_trial_error_types(job_path)
        if not error_types:
            raise ValueError("no failed tasks to re-run")
        clear_stale_job_lock(self.settings, job_id, job_path, run.get("harbor_job_name"))
        return self._resume_operation(
            run,
            job_path,
            operation_type="rerun-failed-job",
            filter_error_types=error_types,
        )

    def _resume_operation(
        self,
        run: dict,
        job_path: Path,
        *,
        operation_type: str,
        filter_error_types: list[str] | None = None,
    ) -> dict:
        job_id = run["id"]

        async def work(progress) -> None:
            progress(10, "Resuming Harbor job")
            mark_resume_running(self.settings, self.events, run)
            policy = None
            run_agent_env = agent_env(self.profiles, run.get("agent_id"))
            run_environment_env = environment_env(
                self.profiles, self._job_config(job_id).get("environment_preset_id")
            )
            try:
                await cleanup_resume_leftovers(job_path)
                restore_sensitive_env(job_path, run_agent_env, run_environment_env)
                policy = await prepare_resume_proxy(self.experiments.container_proxy, job_path)
                await self._resume_harbor_job(
                    job_path,
                    env={**os.environ, **run_agent_env},
                    filter_error_types=filter_error_types,
                )
            except asyncio.CancelledError as exc:
                mark_resume_failed(self.settings, self.events, run, exc)
                raise
            except Exception as exc:
                mark_resume_failed(self.settings, self.events, run, exc)
                raise
            finally:
                if policy is not None:
                    await policy.close()
            RunRecoveryService(self.settings).reconcile_run(job_id)
            progress(100, "Harbor job resumed")

        return self.operations.submit(operation_type, "job", job_id, work)

    def update_leaderboard(self, job_id: str, include: bool) -> tuple[dict, dict, list[dict]]:
        job = self.get_job(job_id)
        if include:
            config = self._job_config(job_id)
            if config.get("harbor_overrides", {}).get("verifier", {}).get("disable"):
                raise ValueError("jobs without verification cannot enter the leaderboard")
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET leaderboard_eligible = ? WHERE id = ?", (int(include), job_id)
            )
        operation = self.operations.complete(
            "update-job-leaderboard", "job", job_id, "Leaderboard inclusion updated"
        )
        return self.get_job(job_id), operation, leaderboard(self.settings, job["datasetRef"])

    def events_for_job(self, job_id: str) -> list[dict]:
        run = self.experiments.get_run(job_id)
        events = self.events.list_after_many([job_id, run["experiment_id"]], 0)
        return [
            {
                "level": event_level(event.severity),
                "message": event.event_type,
                "occurredAt": event.ts,
            }
            for event in events
        ]

    async def trials_for_job(self, job_id: str) -> list[dict]:
        run = self.experiments.get_run(job_id)
        if not run.get("job_dir"):
            return []
        config = self._job_config(job_id)
        job_path = Path(run["job_dir"])
        job_name = run.get("harbor_job_name") or config.get("job_name")
        result_path = run.get("result_path")
        pricing = config.get("pricing")
        entries: list[tuple[float, dict]] = [
            (trial_start_epoch(item), trial_dto(job_id, item, pricing))
            for item in trial_result_payloads(job_path, job_name, result_path)
        ]
        started = {trial["taskName"] for _, trial in entries}
        in_flight_status = "running" if run.get("status") == "running" else "interrupted"
        for descriptor in running_trial_descriptors(job_path, job_name, result_path):
            entries.append(
                (
                    trial_dir_epoch(descriptor),
                    running_trial_dto(job_id, descriptor, in_flight_status),
                )
            )
            started.add(descriptor["task_name"])
        ref = join_ref(str(run.get("benchmark_name") or ""), run.get("benchmark_version"))
        for task_name in await pending_task_names(
            self.settings, ref, started, load_job_result(run).get("n_total_trials")
        ):
            entries.append((0.0, pending_trial_dto(job_id, task_name)))
        entries.sort(key=lambda entry: (-entry[0], entry[1]["taskName"]))
        return [entry[1] for entry in entries]

    def logs_for_job(self, job_id: str) -> dict:
        return job_log_payload(self.experiments.get_run(job_id), self._job_config(job_id))

    def leaderboard(
        self, dataset_ref: str, query: str | None = None, metric: str | None = None
    ) -> list[dict]:
        return leaderboard(self.settings, dataset_ref, query, metric)

    def leaderboard_datasets(self) -> list[dict]:
        return leaderboard_datasets(self.settings)

    async def _resume_harbor_job(
        self,
        job_path: Path,
        env: dict | None = None,
        filter_error_types: list[str] | None = None,
    ) -> None:
        command = [harbor_cli_executable(), "job", "resume", "--job-path", str(job_path)]
        for error_type in filter_error_types or []:
            command.extend(["--filter-error-type", error_type])
        process = await asyncio.create_subprocess_exec(
            *command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            env=env,
        )
        output, _ = await process.communicate()
        if process.returncode != 0:
            message = output.decode("utf-8", errors="replace").strip()
            raise RuntimeError(
                f"harbor job resume exited with {process.returncode}: {resume_error_tail(message)}"
            )

    def _job_config(self, job_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn, "SELECT config_json FROM webui_job_configs WHERE run_id = ?", (job_id,)
            )
        return json.loads(rows[0]["config_json"]) if rows else {}
