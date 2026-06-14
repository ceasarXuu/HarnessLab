from __future__ import annotations

from importlib import metadata

from harnesslab.models.harbor import HarborCapabilitySnapshot, HarborJobConfigView
from harnesslab.settings import Settings


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
    ) -> HarborJobConfigView:
        dataset_name = (
            f"{benchmark_name}@{benchmark_version}" if benchmark_version else benchmark_name
        )
        return HarborJobConfigView(
            agent=agent_config,
            dataset={"name": dataset_name},
            n_tasks=n_tasks,
            n_attempts=n_attempts,
            n_concurrent=n_concurrent,
            jobs_dir=jobs_dir,
        )


class HarborEngine:
    def capability_snapshot(self) -> HarborCapabilitySnapshot:
        return HarborCapabilitySnapshot(
            harbor_version=_version("harbor"),
            api_symbols=["Job.create", "Job.run", "JobConfig", "AgentConfig"],
            lifecycle_mode="python_api_spike_required",
            environment_backend="docker",
        )

    async def run(self, config: HarborJobConfigView) -> dict:
        if config.dataset["name"] == "fake-docker-failure":
            raise RuntimeError("docker compose returned code -9")
        return {
            "status": "completed",
            "score": 1.0,
            "job_dir": config.jobs_dir,
            "result_path": f"{config.jobs_dir}/result.json",
        }


def _version(package: str) -> str | None:
    try:
        return metadata.version(package)
    except metadata.PackageNotFoundError:
        return None
