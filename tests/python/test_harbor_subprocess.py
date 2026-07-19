import asyncio
import json
import logging
import os
import signal
import sys
from pathlib import Path

import pytest

from ornnlab.services.harbor_engine import HarborConfigBuilder, HarborEngine
from ornnlab.services.harbor_subprocess import ManagedSubprocessHarborRunner
from ornnlab.settings import Settings


def test_managed_subprocess_runner_uses_harbor_config(tmp_path, caplog):
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

    caplog.set_level(logging.INFO)
    result = asyncio.run(
        ManagedSubprocessHarborRunner(command=[sys.executable, str(script)]).run(config)
    )

    assert result["status"] == "completed"
    assert result["score"] == 0.42
    assert (tmp_path / "harbor-job" / "job.log").read_text() == "fake harbor completed\n"
    assert "harbor_subprocess.start" in caplog.text
    assert f"executable={sys.executable}" in caplog.text
    assert "job_name=subprocess-success" in caplog.text


def test_managed_subprocess_runner_resolves_proxy_only_in_child_environment(tmp_path):
    script = tmp_path / "fake_harbor_proxy_cli.py"
    script.write_text(
        "\n".join(
            [
                "import json",
                "import os",
                "import pathlib",
                "import sys",
                "config_path = pathlib.Path(sys.argv[sys.argv.index('--config') + 1])",
                "config = json.loads(config_path.read_text())",
                "assert config['environment']['env']['HTTPS_PROXY'] "
                "== 'http://172.17.0.1:32123'",
                "assert 'env' not in config['agents'][0]",
                "assert os.environ['ORNNLAB_CONTAINER_HTTPS_PROXY'] == 'http://172.17.0.1:32123'",
                "assert config_path.name != 'harbor.config.json'",
                "job_dir = pathlib.Path(config['jobs_dir'])",
                "job_dir.mkdir(parents=True, exist_ok=True)",
                "(job_dir / 'result.json').write_text(json.dumps({'status': 'completed'}))",
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
        runtime_container_env_defaults={"HTTPS_PROXY": "${ORNNLAB_CONTAINER_HTTPS_PROXY}"},
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    result = asyncio.run(
        ManagedSubprocessHarborRunner(command=[sys.executable, str(script)]).run(
            config,
            extra_env={"ORNNLAB_CONTAINER_HTTPS_PROXY": "http://172.17.0.1:32123"},
        )
    )

    assert result["status"] == "completed"
    artifact = (tmp_path / "harbor-job" / "harbor.config.json").read_text()
    assert "${ORNNLAB_CONTAINER_HTTPS_PROXY}" in artifact
    assert "172.17.0.1:32123" not in artifact
    assert list(tmp_path.glob("**/.harbor.runtime.*")) == []


def test_managed_subprocess_runner_rejects_missing_runtime_container_value(tmp_path):
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
        runtime_container_env_defaults={
            "HTTPS_PROXY": "${ORNNLAB_CONTAINER_HTTPS_PROXY}"
        },
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())

    with pytest.raises(
        RuntimeError,
        match="runtime container environment variable is unavailable: "
        "ORNNLAB_CONTAINER_HTTPS_PROXY",
    ):
        asyncio.run(
            ManagedSubprocessHarborRunner(
                command=[sys.executable, str(tmp_path / "must-not-run.py")]
            ).run(config)
        )


def test_managed_subprocess_runner_reports_missing_executable(tmp_path, caplog):
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
        "missing-cli",
    )
    builder.write_run_artifacts(config, HarborEngine(mode="subprocess").capability_snapshot())
    missing = tmp_path / "missing" / "harbor"

    caplog.set_level(logging.INFO)
    with pytest.raises(FileNotFoundError, match="Harbor CLI executable not found") as error:
        asyncio.run(ManagedSubprocessHarborRunner(command=[str(missing), "run"]).run(config))

    assert error.value.filename == str(missing)
    assert "harbor_subprocess.spawn_failed" in caplog.text
    assert f"executable={missing}" in caplog.text


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
    assert cleanup["returncode"] is not None
    if os.name != "nt":
        assert terminated.read_text() == str(signal.SIGTERM)
    else:
        assert not terminated.exists()


def test_subprocess_command_env_uses_ornnlab_variable(monkeypatch):
    monkeypatch.setenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", "new-harbor run")

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == ["new-harbor", "run"]


def test_subprocess_constructor_rejects_empty_command():
    with pytest.raises(ValueError, match="Harbor subprocess command cannot be empty"):
        ManagedSubprocessHarborRunner(command=[])


def test_subprocess_default_command_resolves_harbor_next_to_python(monkeypatch):
    monkeypatch.delenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", raising=False)
    monkeypatch.delenv("ORNNLAB_HARBOR_CLI", raising=False)
    monkeypatch.setattr("ornnlab.services.harbor_subprocess.shutil.which", lambda _: None)

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == [str(Path(sys.executable).parent / "harbor"), "run"]


def test_subprocess_default_command_prefers_ornnlab_harbor_cli(monkeypatch):
    monkeypatch.delenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", raising=False)
    monkeypatch.setenv("ORNNLAB_HARBOR_CLI", "/opt/ornnlab/bin/harbor")
    monkeypatch.setattr(
        "ornnlab.services.harbor_subprocess.shutil.which", lambda _: "/usr/bin/harbor"
    )

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == ["/opt/ornnlab/bin/harbor", "run"]


def test_subprocess_command_ignores_legacy_harnesslab_variable(monkeypatch):
    # Regression guard: the HARNESSLAB_* fallback was retired in v0.1.4.
    monkeypatch.delenv("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", raising=False)
    monkeypatch.setenv("HARNESSLAB_HARBOR_SUBPROCESS_COMMAND", "old-harbor run")
    monkeypatch.setattr(
        "ornnlab.services.harbor_subprocess.harbor_cli_executable",
        lambda: "/resolved/harbor",
    )

    runner = ManagedSubprocessHarborRunner()

    assert runner.command == ["/resolved/harbor", "run"]
