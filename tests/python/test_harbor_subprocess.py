import asyncio
import json
import sys

import pytest

from ornnlab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from ornnlab.services.harbor_subprocess import ManagedSubprocessHarborRunner
from ornnlab.settings import Settings


def test_managed_subprocess_runner_uses_harbor_config(tmp_path):
    script = tmp_path / "fake_harbor_cli.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import pathlib",
                "import sys",
                "config_path = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])",
                "config = json.loads(config_path.read_text())",
                "job_dir = pathlib.Path(config['jobs_dir'])",
                "job_dir.mkdir(parents=True, exist_ok=True)",
                "(job_dir / 'result.json').write_text(",
                "    json.dumps({'status': 'completed', 'score': 0.42}),",
                ")",
                "print('fake harbor completed', flush=True)",
            ]
        ),
        encoding="utf-8",
    )
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    config = builder.build(
        {"name": "oracle"},
        "terminal-bench",
        "2.0",
        1,
        1,
        1,
        str(tmp_path / "harbor-job"),
        "subprocess-success",
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    result = asyncio.run(
        ManagedSubprocessHarborRunner(command=[sys.executable, str(script)]).run(config)
    )

    assert result["status"] == "completed"
    assert result["score"] == 0.42
    assert (tmp_path / "harbor-job" / "job.log").read_text() == "fake harbor completed\n"


def test_managed_subprocess_runner_reads_harbor_native_result_layout(tmp_path):
    script = tmp_path / "fake_harbor_native_layout.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import pathlib",
                "import sys",
                "config_path = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])",
                "config = json.loads(config_path.read_text())",
                "job_dir = pathlib.Path(config['jobs_dir']) / config['job_name']",
                "job_dir.mkdir(parents=True, exist_ok=True)",
                "(job_dir / 'result.json').write_text(",
                "    json.dumps({'status': 'completed', 'score': 0.73}),",
                ")",
            ]
        ),
        encoding="utf-8",
    )
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    config = builder.build(
        {"name": "oracle"},
        "hello-world",
        "1.0",
        1,
        1,
        1,
        str(tmp_path / "harbor-job"),
        "native-layout",
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    result = asyncio.run(
        ManagedSubprocessHarborRunner(command=[sys.executable, str(script)]).run(config)
    )

    assert result["status"] == "completed"
    assert result["score"] == 0.73
    assert result["result_path"] == str(tmp_path / "harbor-job" / "native-layout" / "result.json")


def test_managed_subprocess_runner_uses_native_trial_stats_for_status(tmp_path):
    script = tmp_path / "fake_harbor_native_failure.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import pathlib",
                "import sys",
                "config_path = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])",
                "config = json.loads(config_path.read_text())",
                "job_dir = pathlib.Path(config['jobs_dir']) / config['job_name']",
                "job_dir.mkdir(parents=True, exist_ok=True)",
                "(job_dir / 'result.json').write_text(json.dumps({",
                "    'n_total_trials': 1,",
                "    'stats': {",
                "        'n_completed_trials': 1, 'n_errored_trials': 1, 'n_cancelled_trials': 0",
                "    },",
                "}))",
            ]
        ),
        encoding="utf-8",
    )
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    config = builder.build(
        {"name": "oracle"},
        "hello-world",
        "1.0",
        1,
        1,
        1,
        str(tmp_path / "harbor-job"),
        "native-failure",
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    result = asyncio.run(
        ManagedSubprocessHarborRunner(command=[sys.executable, str(script)]).run(config)
    )

    assert result["status"] == "failed"


def test_managed_subprocess_runner_cleans_process_group_on_cancel(tmp_path):
    script = tmp_path / "fake_long_harbor_cli.py"
    started = tmp_path / "started.txt"
    terminated = tmp_path / "terminated.txt"
    script.write_text(
        "\n".join(
            [
                "import pathlib",
                "import signal",
                "import sys",
                "import time",
                "started = pathlib.Path(sys.argv[1])",
                "terminated = pathlib.Path(sys.argv[2])",
                "def handle_term(signum, frame):",
                "    terminated.write_text(str(signum))",
                "    sys.exit(0)",
                "signal.signal(signal.SIGTERM, handle_term)",
                "started.write_text('ready')",
                "while True:",
                "    time.sleep(0.1)",
            ]
        ),
        encoding="utf-8",
    )
    settings = Settings(home=tmp_path)
    builder = HarborConfigBuilder(settings)
    config = builder.build(
        {"name": "oracle"},
        "terminal-bench",
        "2.0",
        1,
        1,
        1,
        str(tmp_path / "harbor-job"),
        "subprocess-cancel",
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    async def run_and_cancel() -> None:
        runner = ManagedSubprocessHarborRunner(
            command=[sys.executable, str(script), str(started), str(terminated)],
            terminate_grace_sec=0.5,
        )
        task = asyncio.create_task(runner.run(config))
        for _ in range(200):
            if started.exists():
                break
            await asyncio.sleep(0.01)
        assert started.exists()
        task.cancel()
        with pytest.raises(asyncio.CancelledError):
            await task

    asyncio.run(run_and_cancel())

    cleanup = json.loads((tmp_path / "harbor-job" / "harbor.cleanup.json").read_text())
    assert cleanup["reason"] == "task_cancelled"
    assert cleanup["terminated"] is True
    assert terminated.read_text() == "15"


def test_subprocess_command_env_uses_ornnlab_variable(monkeypatch):
    monkeypatch.setenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", "new-harbor run")

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == ["new-harbor", "run"]


def test_subprocess_command_env_ignores_legacy_harnesslab_variable(monkeypatch):
    # Regression guard: HARNESSLAB_HARBOR_SUBPROCESS_COMMAND was a legacy
    # compatibility fallback retired in v0.1.4. Production code in
    # ornnlab/services/harbor_subprocess.py must NOT read it. This test will
    # fail if anyone accidentally reintroduces a HARNESSLAB_* env fallback.
    # AC1 grep exempts this file via scripts/verify-ornnlab-rebrand.py guard
    # exemptions; see harnesslab-shim-retirement-prd.md SC-5 follow-up.
    monkeypatch.delenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", raising=False)
    monkeypatch.setenv("HARNESSLAB_HARBOR_SUBPROCESS_COMMAND", "old-harbor run")

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == ["harbor", "run"]
