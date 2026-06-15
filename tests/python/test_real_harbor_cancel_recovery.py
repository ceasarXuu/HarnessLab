import asyncio
import json
import os
import shutil

import pytest

from ornnlab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from ornnlab.settings import Settings

pytestmark = pytest.mark.docker


def _real_harbor_enabled() -> bool:
    return (
        os.environ.get("ORNNLAB_REAL_HARBOR", os.environ.get("HARNESSLAB_REAL_HARBOR")) == "1"
        and shutil.which("docker") is not None
    )


def _real_config(tmp_path, job_name: str):
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    benchmark_version = (
        os.environ.get("ORNNLAB_REAL_HARBOR_BENCHMARK_VERSION")
        or os.environ.get("HARNESSLAB_REAL_HARBOR_BENCHMARK_VERSION")
        or None
    )
    config = builder.build(
        agent_config={
            "name": os.environ.get(
                "ORNNLAB_REAL_HARBOR_AGENT",
                os.environ.get("HARNESSLAB_REAL_HARBOR_AGENT", "oracle"),
            )
        },
        benchmark_name=os.environ.get(
            "ORNNLAB_REAL_HARBOR_BENCHMARK",
            os.environ.get("HARNESSLAB_REAL_HARBOR_BENCHMARK", "terminal-bench"),
        ),
        benchmark_version=benchmark_version,
        n_tasks=int(
            os.environ.get(
                "ORNNLAB_REAL_HARBOR_N_TASKS",
                os.environ.get("HARNESSLAB_REAL_HARBOR_N_TASKS", "1"),
            )
        ),
        n_attempts=1,
        n_concurrent=1,
        jobs_dir=str(tmp_path / job_name / "harbor-job"),
        job_name=job_name,
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())
    return config


@pytest.mark.skipif(
    not _real_harbor_enabled(),
    reason="set ORNNLAB_REAL_HARBOR=1 with Docker available to run real Harbor subprocess",
)
def test_real_harbor_subprocess_smoke(tmp_path):
    config = _real_config(tmp_path, "real-harbor-subprocess-smoke")

    result = asyncio.run(HarborEngine(mode="subprocess").run(config))

    job_dir = tmp_path / "real-harbor-subprocess-smoke" / "harbor-job"
    assert result["status"] in {"completed", "failed", "cancelled", "interrupted"}
    assert (job_dir / "harbor.config.json").exists()
    assert (job_dir / "job.log").exists()
    assert (job_dir / "result.json").exists()


@pytest.mark.skipif(
    not _real_harbor_enabled(),
    reason="set ORNNLAB_REAL_HARBOR=1 with Docker available to run real Harbor cancel recovery",
)
def test_real_harbor_subprocess_cancel_writes_cleanup_evidence(tmp_path):
    config = _real_config(tmp_path, "real-harbor-subprocess-cancel")

    async def run_and_cancel() -> None:
        task = asyncio.create_task(HarborEngine(mode="subprocess").run(config))
        await asyncio.sleep(
            float(
                os.environ.get(
                    "ORNNLAB_REAL_HARBOR_CANCEL_DELAY",
                    os.environ.get("HARNESSLAB_REAL_HARBOR_CANCEL_DELAY", "1.0"),
                )
            )
        )
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task

    asyncio.run(run_and_cancel())

    job_dir = tmp_path / "real-harbor-subprocess-cancel" / "harbor-job"
    cleanup = json.loads((job_dir / "harbor.cleanup.json").read_text())
    assert cleanup["reason"] == "task_cancelled"
    assert cleanup["terminated"] is True or cleanup.get("missing") is True
    assert "returncode" in cleanup
