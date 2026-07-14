from __future__ import annotations

import asyncio
import json
from datetime import datetime
from pathlib import Path

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.models.webui import CreateJobInput
from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.services.harbor_paths import (
    resolve_harbor_job_path,
    resolve_harbor_log_path,
)
from ornnlab.services.harbor_results import (
    load_result_payload,
    trial_log_path,
    trial_result_payloads,
)
from ornnlab.services.harbor_score import result_pass_at_one
from ornnlab.services.harbor_subprocess import harbor_cli_executable
from ornnlab.services.queue_service import QueueService
from ornnlab.services.recovery_service import RunRecoveryService
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

_JOB_SELECT = """
    SELECT runs.*, experiments.name AS experiment_name,
           agents.name AS agent_profile_name, webui_job_configs.config_json
    FROM runs
    JOIN experiments ON experiments.id = runs.experiment_id
    JOIN agents ON agents.id = runs.agent_id
    LEFT JOIN webui_job_configs ON webui_job_configs.run_id = runs.id
"""


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
                _JOB_SELECT + "WHERE experiments.status != 'deleted' ORDER BY runs.created_at DESC",
            )
        jobs = [_job_dto(row) for row in rows]
        if not query:
            return jobs
        needle = query.lower()
        return [
            job for job in jobs if needle in " ".join(str(value) for value in job.values()).lower()
        ]

    def get_job(self, job_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                _JOB_SELECT + "WHERE runs.id = ?",
                (job_id,),
            )
        if not rows:
            raise KeyError(job_id)
        return _job_dto(rows[0])

    def create_job(self, request: CreateJobInput) -> tuple[dict, dict]:
        config = request.config
        agent = self.profiles.resolve_agent(config.agent_name)
        agent = self.profiles.ensure_agent_persisted(agent)
        environment = self.profiles.get_environment(config.environment_preset_id)
        benchmark_name, benchmark_version = _dataset_ref(config.dataset_ref)
        selected_tasks = config.selected_task_names
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
                "include_exceptions": _exception_list(config.retry_include),
                "exclude_exceptions": _exception_list(config.retry_exclude),
                "wait_multiplier": config.retry_wait_multiplier,
                "min_wait_sec": config.retry_min_wait_seconds,
                "max_wait_sec": config.retry_max_wait_seconds,
            },
            "verifier": {"disable": config.verifier_mode == "skip"},
            "metrics": [{"type": config.metric}],
            "environment": self.profiles.environment_harbor_config(environment),
        }
        stored = {
            "agent_harness": agent["harness"],
            "agent_name": agent["agentName"],
            "environment_name": environment["name"],
            "environment_preset_id": environment["id"],
            "job_name": config.job_name,
            "jobs_dir": config.jobs_dir,
            "harbor_overrides": overrides,
            "model": _first(agent["models"]),
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
        if not run.get("job_dir"):
            raise ValueError("Harbor job directory is unavailable for resume")
        job_path = resolve_harbor_job_path(Path(run["job_dir"]), run.get("harbor_job_name"))
        if not job_path.is_dir():
            raise ValueError("Harbor job directory is unavailable for resume")

        async def work(progress) -> None:
            progress(10, "Resuming Harbor job")
            self._mark_resume_running(run)
            try:
                await self._resume_harbor_job(job_path)
            except Exception as exc:
                self._mark_resume_failed(run, exc)
                raise
            RunRecoveryService(self.settings).reconcile_run(job_id)
            progress(100, "Harbor job resumed")

        return self.operations.submit("resume-job", "job", job_id, work)

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
        return self.get_job(job_id), operation, self.leaderboard(job["datasetRef"])

    def events_for_job(self, job_id: str) -> list[dict]:
        run = self.experiments.get_run(job_id)
        events = self.events.list_after_many([job_id, run["experiment_id"]], 0)
        return [
            {
                "level": _event_level(event.severity),
                "message": event.event_type,
                "occurredAt": event.ts,
            }
            for event in events
        ]

    def trials_for_job(self, job_id: str) -> list[dict]:
        run = self.experiments.get_run(job_id)
        if not run.get("job_dir"):
            return []
        config = self._job_config(job_id)
        results = trial_result_payloads(
            Path(run["job_dir"]),
            run.get("harbor_job_name") or config.get("job_name"),
            run.get("result_path"),
        )
        return [
            _trial_dto(job_id, item)
            for item in results
        ]

    def leaderboard(
        self, dataset_ref: str, query: str | None = None, metric: str | None = None
    ) -> list[dict]:
        benchmark, version = _dataset_ref(dataset_ref)
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT runs.*, webui_job_configs.config_json FROM runs "
                "LEFT JOIN webui_job_configs ON webui_job_configs.run_id = runs.id "
                "WHERE runs.status = 'completed' AND runs.leaderboard_eligible = 1 "
                f"AND runs.benchmark_name = ? AND {_version_filter(version)} "
                "ORDER BY runs.score DESC, runs.finished_at DESC",
                (benchmark,) if version is None else (benchmark, version),
            )
        entries = []
        for rank, row in enumerate(rows, start=1):
            job = _job_dto(row)
            config = _config(row)
            if (
                metric
                and config.get("harbor_overrides", {}).get("metrics", [{}])[0].get("type") != metric
            ):
                continue
            entry = {
                "agentName": job["agentName"],
                "comparabilityKey": row.get("comparability_key") or "",
                "costUsd": job["costUsd"],
                "datasetRef": job["datasetRef"],
                "harness": job["harness"],
                "jobId": job["id"],
                "metric": config.get("harbor_overrides", {})
                .get("metrics", [{}])[0]
                .get("type", "mean"),
                "model": job["model"],
                "rank": rank,
                "reportPath": row.get("report_path"),
                "runtimeSeconds": job["runtimeSeconds"],
                "score": job["score"],
                "submittedAt": row.get("finished_at") or row["created_at"],
                "tokenUsageM": job["tokenUsageM"],
                "trial": job["trial"],
            }
            if (
                not query
                or query.lower() in " ".join(str(value) for value in entry.values()).lower()
            ):
                entries.append(entry)
        return entries

    def leaderboard_datasets(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT DISTINCT benchmark_name, benchmark_version FROM runs "
                "WHERE leaderboard_eligible = 1",
            )
        return [
            {
                "name": row["benchmark_name"],
                "version": row["benchmark_version"] or "latest",
                "ref": _join_ref(row["benchmark_name"], row["benchmark_version"]),
            }
            for row in rows
        ]

    async def _resume_harbor_job(self, job_path: Path) -> None:
        command = [harbor_cli_executable(), "job", "resume", "--job-path", str(job_path)]
        process = await asyncio.create_subprocess_exec(*command)
        code = await process.wait()
        if code != 0:
            raise RuntimeError(f"harbor job resume exited with {code}")

    def _mark_resume_running(self, run: dict) -> None:
        now = _now()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = NULL, failure_class = NULL, "
                "failure_code = NULL, failure_summary = NULL, updated_at = ? WHERE id = ?",
                ("running", now, run["id"]),
            )
        self.events.append("run", run["id"], "harbor.job.resume_requested", {})

    def _mark_resume_failed(self, run: dict, error: Exception) -> None:
        now = _now()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE runs SET status = ?, finished_at = ?, failure_class = ?, "
                "failure_code = ?, failure_summary = ?, updated_at = ? WHERE id = ?",
                (
                    "interrupted",
                    now,
                    "harbor_resume",
                    "resume_command_failed",
                    str(error),
                    now,
                    run["id"],
                ),
            )
        self.events.append(
            "run",
            run["id"],
            "harbor.job.resume_failed",
            {"error": str(error)},
            severity="warning",
        )

    def _job_config(self, job_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn, "SELECT config_json FROM webui_job_configs WHERE run_id = ?", (job_id,)
            )
        return json.loads(rows[0]["config_json"]) if rows else {}


def _job_dto(row: dict) -> dict:
    config = _config(row)
    result = _result_payload(row.get("result_path"))
    stats = result.get("stats", {})
    status = str(row["status"])
    attempts = max(1, int(row["n_attempts"]))
    expected_total = (int(row["n_tasks"]) if row.get("n_tasks") is not None else 0) * attempts
    total = int(result.get("n_total_trials", expected_total))
    complete = int(stats.get("n_completed_trials", 0))
    if not result and status in {"completed", "failed", "cancelled", "interrupted"}:
        complete = total
    return {
        "id": row["id"],
        "name": config.get("job_name", row.get("experiment_name", row["id"])),
        "status": status,
        "datasetRef": _join_ref(row["benchmark_name"], row["benchmark_version"]),
        "agentName": config.get("agent_name", row.get("agent_profile_name", row["agent_id"])),
        "harness": config.get("agent_harness", row["agent_id"]),
        "model": config.get("model", ""),
        "environmentName": config.get("environment_name", config.get("environment_preset_id", "")),
        "trial": {"completed": complete, "total": total},
        "score": _job_score(result),
        "costUsd": _number_or_none(stats.get("cost_usd")),
        "tokenUsageM": _token_usage_m(stats),
        "runtimeSeconds": _duration_seconds(row.get("started_at"), row.get("finished_at")),
        "createdAt": row["created_at"],
        "includeInLeaderboard": bool(row["leaderboard_eligible"]),
        "jobDir": row.get("job_dir"),
        "eventLogPath": _event_log_path(row, config),
        "artifactPaths": _artifacts(row),
        "failureCode": row.get("failure_code"),
    }


def _trial_dto(job_id: str, item: dict) -> dict:
    agent_result = item.get("agent_result") or {}
    token_usage = _trial_token_usage(agent_result, item.get("step_results"))
    return {
        "id": str(item.get("id", item.get("trial_name", "unknown"))),
        "jobId": job_id,
        "taskName": str(item.get("task_name", item.get("name", "unknown"))),
        "status": "failed" if item.get("exception_info") else "passed",
        "score": _verifier_score(item.get("verifier_result")),
        "retryCount": None,
        "runtimeSeconds": _duration_seconds(item.get("started_at"), item.get("finished_at")),
        "costUsd": _number_or_none(token_usage.get("cost_usd")),
        "tokenUsageM": _token_usage_m(token_usage),
        "logPath": trial_log_path(item),
    }


def _config(row: dict) -> dict:
    return json.loads(row["config_json"]) if row.get("config_json") else {}


def _dataset_ref(ref: str) -> tuple[str, str | None]:
    name, separator, version = ref.rpartition("@")
    return (name, version) if separator else (ref, None)


def _join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name


def _exception_list(value: str) -> list[str] | None:
    values = [item.strip() for item in value.replace(",", "\n").splitlines() if item.strip()]
    return values or None


def _event_level(severity: str) -> str:
    return {"error": "error", "warning": "warning"}.get(
        severity, "success" if severity == "info" else "info"
    )


def _job_score(result: dict) -> dict | None:
    """Expose only scores whose scale is explicit in Harbor's result payload."""
    value = result_pass_at_one(result)
    if value is not None:
        return {"kind": "percentage", "value": value * 100}
    return None


def _verifier_score(value: object) -> dict | None:
    if not isinstance(value, dict):
        return None
    rewards = value.get("rewards")
    if not isinstance(rewards, dict):
        return None
    value = rewards.get("pass")
    if isinstance(value, int | float) and value in {0, 1}:
        return {"kind": "percentage", "value": float(value) * 100}
    return None


def _version_filter(version: str | None) -> str:
    return "runs.benchmark_version IS NULL" if version is None else "runs.benchmark_version = ?"


def _duration_seconds(started: str | None, finished: str | None) -> int | None:
    if not started or not finished:
        return None
    return max(
        0, int((datetime.fromisoformat(finished) - datetime.fromisoformat(started)).total_seconds())
    )


def _artifacts(row: dict) -> list[str]:
    values = [row.get("result_path"), row.get("report_path")]
    if row.get("job_dir"):
        values.append(str(Path(row["job_dir"]) / "harbor.config.json"))
    return [value for value in values if value]


def _event_log_path(row: dict, config: dict) -> str | None:
    if not row.get("job_dir"):
        return None
    return str(resolve_harbor_log_path(Path(row["job_dir"]), config.get("job_name")))


def _now() -> str:
    return datetime.now().astimezone().isoformat()


def _result_payload(result_path: str | None) -> dict:
    if not result_path:
        return {}
    return load_result_payload(Path(result_path))


def _number_or_none(value: object) -> float | None:
    return float(value) if isinstance(value, int | float) else None


def _token_usage_m(stats: dict) -> float | None:
    values = [stats.get("n_input_tokens"), stats.get("n_output_tokens")]
    if not any(isinstance(value, int | float) for value in values):
        return None
    return sum(float(value) for value in values if isinstance(value, int | float)) / 1_000_000


def _trial_token_usage(agent_result: object, step_results: object) -> dict:
    contexts = [agent_result] if isinstance(agent_result, dict) else []
    if not contexts and isinstance(step_results, list):
        contexts = [item.get("agent_result") for item in step_results if isinstance(item, dict)]
    result: dict[str, float] = {}
    for context in contexts:
        if not isinstance(context, dict):
            continue
        for key in ("n_input_tokens", "n_output_tokens", "cost_usd"):
            value = context.get(key)
            if isinstance(value, int | float):
                result[key] = result.get(key, 0.0) + float(value)
    return result


def _first(values: list[str]) -> str | None:
    return values[0] if values else None
