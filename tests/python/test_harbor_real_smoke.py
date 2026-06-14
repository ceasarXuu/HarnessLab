import asyncio
import os
import shutil

import pytest

from harnesslab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from harnesslab.settings import Settings


@pytest.mark.docker
@pytest.mark.skipif(
    os.environ.get("HARNESSLAB_REAL_HARBOR") != "1" or shutil.which("docker") is None,
    reason="set HARNESSLAB_REAL_HARBOR=1 with Docker available to run real Harbor smoke",
)
def test_real_harbor_python_api_smoke(tmp_path):
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    benchmark_version = os.environ.get("HARNESSLAB_REAL_HARBOR_BENCHMARK_VERSION") or None
    config = builder.build(
        agent_config={"name": os.environ.get("HARNESSLAB_REAL_HARBOR_AGENT", "oracle")},
        benchmark_name=os.environ.get("HARNESSLAB_REAL_HARBOR_BENCHMARK", "terminal-bench"),
        benchmark_version=benchmark_version,
        n_tasks=int(os.environ.get("HARNESSLAB_REAL_HARBOR_N_TASKS", "1")),
        n_attempts=1,
        n_concurrent=1,
        jobs_dir=str(tmp_path / "harbor-job"),
        job_name="real-harbor-smoke",
    )

    builder.write_run_artifacts(config, HarborEngine(mode="python-api").capability_snapshot())
    result = asyncio.run(HarborEngine(mode="python-api").run(config))

    assert result["status"] in {"completed", "failed", "cancelled", "interrupted"}
    assert (tmp_path / "harbor-job" / "result.json").exists()
