from __future__ import annotations

import asyncio
import json
import os
from importlib import import_module, metadata
from pathlib import Path
from typing import Any, cast

from ornnlab.models.harbor import HarborCapabilitySnapshot, HarborJobConfigView
from ornnlab.services.harbor_subprocess import ManagedSubprocessHarborRunner
from ornnlab.settings import Settings
from ornnlab.storage.paths import atomic_write_text

CONFIG_FILE_NAME = "harbor.config.json"
CAPABILITY_FILE_NAME = "harbor.capability.json"


class HarborConfigBuilder:
    def __init__(self, settings: Settings):
        self.settings = settings

    def build(
        self,
        agent_config: dict,
        benchmark_name: str,
        benchmark_version: str | None,
        n_tasks: int | None,
        n_attempts: int,
        n_concurrent: int,
        jobs_dir: str,
        job_name: str | None = None,
    ) -> HarborJobConfigView:
        dataset_name = (
            f"{benchmark_name}@{benchmark_version}" if benchmark_version else benchmark_name
        )
        return HarborJobConfigView(
            job_name=job_name or f"ornnlab-{_slug(dataset_name)}",
            agent=_normalize_agent_view(agent_config),
            dataset={
                "name": dataset_name,
                "benchmark_name": benchmark_name,
                "benchmark_version": benchmark_version,
            },
            n_tasks=n_tasks,
            n_attempts=n_attempts,
            n_concurrent=n_concurrent,
            jobs_dir=jobs_dir,
        )

    def write_run_artifacts(
        self,
        config: HarborJobConfigView,
        snapshot: HarborCapabilitySnapshot,
    ) -> dict[str, str]:
        config_path = self.config_path(config)
        capability_path = self.capability_path(config)
        atomic_write_text(
            config_path,
            json.dumps(self.to_job_config_payload(config), indent=2, sort_keys=True),
        )
        atomic_write_text(
            capability_path,
            snapshot.model_dump_json(indent=2),
        )
        return {
            "config_path": str(config_path),
            "capability_path": str(capability_path),
        }

    @staticmethod
    def config_path(config: HarborJobConfigView) -> Path:
        return Path(config.jobs_dir) / CONFIG_FILE_NAME

    @staticmethod
    def capability_path(config: HarborJobConfigView) -> Path:
        return Path(config.jobs_dir) / CAPABILITY_FILE_NAME

    @staticmethod
    def to_job_config_payload(config: HarborJobConfigView) -> dict[str, Any]:
        return {
            "job_name": config.job_name,
            "jobs_dir": config.jobs_dir,
            "n_attempts": config.n_attempts,
            "n_concurrent_trials": config.n_concurrent,
            "quiet": True,
            "environment": config.environment,
            "agents": [_agent_config_payload(config.agent)],
            "datasets": [_dataset_config_payload(config.dataset, config.n_tasks)],
        }


class HarborEngine:
    def __init__(self, mode: str | None = None):
        self.mode = _normalize_mode(mode)

    def capability_snapshot(self) -> HarborCapabilitySnapshot:
        return HarborCapabilitySnapshot(
            harbor_version=_version("harbor"),
            api_symbols=["Job.create", "Job.run", "JobConfig", "AgentConfig"],
            lifecycle_mode=self.mode,
            environment_backend="docker",
            config_format="harbor.models.job.config.JobConfig",
            supports_cancel=self.mode == "subprocess",
        )

    async def run(self, config: HarborJobConfigView) -> dict:
        if self.mode == "subprocess":
            return await ManagedSubprocessHarborRunner().run(config)
        if self.mode == "python-api":
            return await PythonApiHarborRunner().run(config)
        return await FakeHarborRunner().run(config)


class FakeHarborRunner:
    async def run(self, config: HarborJobConfigView) -> dict:
        if config.dataset["name"] == "fake-docker-failure":
            raise RuntimeError("docker compose returned code -9")
        if config.dataset["name"] == "fake-slow-cancel":
            for _ in range(20):
                await asyncio.sleep(0.01)
        result = {
            "status": "completed",
            "score": 1.0,
            "job_dir": config.jobs_dir,
            "result_path": f"{config.jobs_dir}/result.json",
        }
        atomic_write_text(
            Path(result["result_path"]),
            json.dumps(result, indent=2, sort_keys=True),
        )
        return result


class PythonApiHarborRunner:
    async def run(self, config: HarborJobConfigView) -> dict:
        job_config_model = cast(Any, import_module("harbor.models.job.config")).JobConfig
        job_model = cast(Any, import_module("harbor.job")).Job
        payload = HarborConfigBuilder.to_job_config_payload(config)
        job_config = job_config_model.model_validate(payload)
        job = await job_model.create(job_config)
        result = await job.run()
        result_path = Path(config.jobs_dir) / "result.json"
        atomic_write_text(result_path, result.model_dump_json(indent=2))
        return {
            "status": _status_from_result(result),
            "score": _score_from_result(result),
            "job_dir": str(getattr(job, "job_dir", config.jobs_dir)),
            "result_path": str(result_path),
            "harbor_job_id": str(getattr(result, "id", getattr(job, "id", ""))),
        }


def _version(package: str) -> str | None:
    try:
        return metadata.version(package)
    except metadata.PackageNotFoundError:
        return None


def _normalize_mode(mode: str | None) -> str:
    raw = mode or os.environ.get("ORNNLAB_HARBOR_ENGINE", "fake")
    normalized = raw.strip().lower().replace("_", "-")
    aliases = {
        "fake": "fake",
        "python": "python-api",
        "python-api": "python-api",
        "real": "python-api",
        "cli": "subprocess",
        "subprocess": "subprocess",
    }
    if normalized not in aliases:
        raise ValueError(
            "ORNNLAB_HARBOR_ENGINE must be one of fake, python-api, subprocess, or real"
        )
    return aliases[normalized]


def _normalize_agent_view(agent_config: dict[str, Any]) -> dict[str, Any]:
    result = dict(agent_config)
    if "agent_timeout_sec" in result and "override_timeout_sec" not in result:
        result["override_timeout_sec"] = result.pop("agent_timeout_sec")
    if "setup_timeout_sec" in result and "override_setup_timeout_sec" not in result:
        result["override_setup_timeout_sec"] = result.pop("setup_timeout_sec")
    return result


def _agent_config_payload(agent: dict[str, Any]) -> dict[str, Any]:
    allowed = {
        "name",
        "import_path",
        "model_name",
        "skills",
        "override_timeout_sec",
        "override_setup_timeout_sec",
        "max_timeout_sec",
        "extra_allowed_hosts",
        "kwargs",
        "env",
    }
    payload = {key: value for key, value in agent.items() if key in allowed}
    if "env" in payload:
        payload["env"] = _resolve_env(payload["env"])
    return _without_empty_values(payload)


def _dataset_config_payload(dataset: dict[str, Any], n_tasks: int | None) -> dict[str, Any]:
    name = str(dataset["name"])
    benchmark_name = dataset.get("benchmark_name")
    version = dataset.get("benchmark_version")
    if benchmark_name is None:
        benchmark_name, version = _split_dataset_ref(name)
    payload: dict[str, Any] = {"name": benchmark_name, "version": version}
    if n_tasks is not None:
        payload["n_tasks"] = n_tasks
    return _without_empty_values(payload)


def _resolve_env(env: Any) -> dict[str, str]:
    if not isinstance(env, dict):
        return {}
    resolved: dict[str, str] = {}
    for key, value in env.items():
        if value is None:
            inherited = os.environ.get(str(key))
            if inherited is not None:
                resolved[str(key)] = inherited
            continue
        resolved[str(key)] = str(value)
    return resolved


def _without_empty_values(payload: dict[str, Any]) -> dict[str, Any]:
    empty_values = (None, "", [], {})
    return {key: value for key, value in payload.items() if value not in empty_values}


def _split_dataset_ref(value: str) -> tuple[str, str | None]:
    if "@" not in value:
        return value, None
    name, version = value.rsplit("@", 1)
    return name, version or None


def _status_from_result(result: Any) -> str:
    stats = getattr(result, "stats", None)
    if stats is None:
        return "completed"
    if getattr(stats, "n_cancelled_trials", 0) > 0:
        return "cancelled"
    if getattr(stats, "n_errored_trials", 0) > 0:
        return "failed"
    total = getattr(result, "n_total_trials", 0)
    completed = getattr(stats, "n_completed_trials", 0)
    return "completed" if total == 0 or completed >= total else "interrupted"


def _score_from_result(result: Any) -> float | None:
    stats = getattr(result, "stats", None)
    evals = getattr(stats, "evals", {}) if stats is not None else {}
    for dataset_stats in evals.values():
        pass_at_k = getattr(dataset_stats, "pass_at_k", {}) or {}
        score = pass_at_k.get(1, pass_at_k.get("1"))
        if score is not None:
            return float(score)
        for metric in getattr(dataset_stats, "metrics", []) or []:
            score = _metric_score(metric)
            if score is not None:
                return score
    return None


def _metric_score(metric: Any) -> float | None:
    if not isinstance(metric, dict):
        return None
    for key in ["score", "mean", "reward", "accuracy"]:
        value = metric.get(key)
        if isinstance(value, int | float):
            return float(value)
    return None


def _slug(value: str) -> str:
    return "".join(ch if ch.isalnum() else "-" for ch in value).strip("-") or "job"
