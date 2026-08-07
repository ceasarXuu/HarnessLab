from __future__ import annotations

import asyncio
import json
import os
import sys
from pathlib import Path
from types import SimpleNamespace

import psutil

from ornnlab.services.webui_job_resume import (
    _live_harbor_process_for,
    agent_env,
    cleanup_resume_leftovers,
    clear_stale_job_lock,
    environment_env,
    restore_sensitive_env,
)
from ornnlab.services.webui_job_service import WebUiJobService
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.storage import sqlite


def _fake_docker(tmp_path, ps_output: str = "") -> Path:
    script = tmp_path / "fake_docker.py"
    script.write_text(
        "import sys\n"
        f"ps_output = {ps_output!r}\n"
        "args = sys.argv[1:]\n"
        "if args[:2] == ['ps', '-aq']:\n"
        "    print(ps_output)\n",
        encoding="utf-8",
    )
    return script


def _write_lock(job_path: Path) -> Path:
    job_path.mkdir(parents=True)
    lock = job_path / "lock.json"
    lock.write_text('{"schema_version": 1}', encoding="utf-8")
    return lock


def test_clear_stale_job_lock_clears_when_job_is_provably_dead(
    settings, tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    lock = _write_lock(job_path)
    monkeypatch.setenv(
        "ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {_fake_docker(tmp_path)}"
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter", lambda *_: iter([])
    )

    assert clear_stale_job_lock(settings, "run-1", job_path) is True
    assert not lock.exists()
    backups = list(job_path.glob("lock.json.bak-*"))
    assert len(backups) == 1
    assert backups[0].read_text(encoding="utf-8") == '{"schema_version": 1}'


def test_clear_stale_job_lock_skips_without_lock_file(settings, tmp_path: Path):
    job_path = tmp_path / "job"
    job_path.mkdir()
    assert clear_stale_job_lock(settings, "run-1", job_path) is False


def test_clear_stale_job_lock_respects_active_operation(
    settings, tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    lock = _write_lock(job_path)
    monkeypatch.setenv(
        "ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {_fake_docker(tmp_path)}"
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter", lambda *_: iter([])
    )
    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO webui_operations(id, operation_type, resource_type, "
            "resource_id, status, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            ("op-active", "resume-job", "job", "run-1", "running", "2026-08-07T00:00:00Z"),
        )

    assert clear_stale_job_lock(settings, "run-1", job_path) is False
    assert lock.exists()


def test_clear_stale_job_lock_respects_live_harbor_process(
    settings, tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    lock = _write_lock(job_path)
    monkeypatch.setenv(
        "ORNNLAB_DOCKER_COMMAND", f"{sys.executable} {_fake_docker(tmp_path)}"
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: iter(
            [
                SimpleNamespace(
                    info={
                        "cmdline": [
                            "harbor",
                            "job",
                            "resume",
                            "--job-path",
                            str(job_path),
                        ]
                    }
                )
            ]
        ),
    )

    assert clear_stale_job_lock(settings, "run-1", job_path) is False
    assert lock.exists()


def test_clear_stale_job_lock_respects_live_containers(
    settings, tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    lock = _write_lock(job_path)
    monkeypatch.setenv(
        "ORNNLAB_DOCKER_COMMAND",
        f"{sys.executable} {_fake_docker(tmp_path, ps_output='abc123')}",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter", lambda *_: iter([])
    )

    assert clear_stale_job_lock(settings, "run-1", job_path) is False
    assert lock.exists()


def test_cleanup_resume_leftovers_chowns_root_owned_files(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    (job_path / "trial").mkdir(parents=True)
    (job_path / "trial" / "root.txt").write_text("x", encoding="utf-8")
    monkeypatch.setattr("ornnlab.services.webui_job_resume.os.getuid", lambda: 99999)

    calls: list[tuple] = []

    class FakeProcess:
        returncode = 0

        async def communicate(self):
            return b"", None

    async def fake_spawn(*args, **_kwargs):
        calls.append(args)
        return FakeProcess()

    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.asyncio.create_subprocess_exec", fake_spawn
    )
    monkeypatch.setenv("ORNNLAB_DOCKER_COMMAND", "docker")

    assert asyncio.run(cleanup_resume_leftovers(job_path)) is True
    assert calls == [
        (
            "docker",
            "run",
            "--rm",
            "-v",
            f"{job_path}:/work",
            "alpine",
            "chown",
            "-R",
            f"99999:{os.getgid()}",
            "/work",
        )
    ]


def test_cleanup_resume_leftovers_skips_without_root_owned_files(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "job"
    job_path.mkdir()
    (job_path / "mine.txt").write_text("x", encoding="utf-8")

    calls: list[tuple] = []

    async def fake_spawn(*args, **_kwargs):
        calls.append(args)
        raise AssertionError("docker must not be invoked")

    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.asyncio.create_subprocess_exec", fake_spawn
    )

    assert asyncio.run(cleanup_resume_leftovers(job_path)) is False
    assert calls == []


def test_restore_sensitive_env_replaces_redacted_values(tmp_path: Path):
    job_path = tmp_path / "job"
    job_path.mkdir()
    trial_dir = job_path / "trial-a"
    trial_dir.mkdir()
    (job_path / "config.json").write_text(
        json.dumps(
            {
                "agents": [
                    {
                        "env": {
                            "ANTHROPIC_AUTH_TOKEN": "sk-d****b47",
                            "ANTHROPIC_MODEL": "deepseek-v4-pro",
                        }
                    }
                ]
            }
        ),
        encoding="utf-8",
    )
    (trial_dir / "config.json").write_text(
        json.dumps({"agent": {"env": {"ANTHROPIC_AUTH_TOKEN": "sk-d****b47"}}}),
        encoding="utf-8",
    )

    restore_sensitive_env(job_path, {"ANTHROPIC_AUTH_TOKEN": "sk-d08b6-real-token"})

    updated = json.loads((job_path / "config.json").read_text(encoding="utf-8"))
    assert updated["agents"][0]["env"]["ANTHROPIC_AUTH_TOKEN"] == "sk-d08b6-real-token"
    assert updated["agents"][0]["env"]["ANTHROPIC_MODEL"] == "deepseek-v4-pro"
    trial = json.loads((trial_dir / "config.json").read_text(encoding="utf-8"))
    assert trial["agent"]["env"]["ANTHROPIC_AUTH_TOKEN"] == "sk-d08b6-real-token"


def test_restore_sensitive_env_keeps_unmapped_redacted_values(tmp_path: Path):
    job_path = tmp_path / "job"
    job_path.mkdir()
    (job_path / "config.json").write_text(
        json.dumps({"agents": [{"env": {"UNKNOWN_SECRET": "ab****xyz"}}]}),
        encoding="utf-8",
    )

    restore_sensitive_env(job_path, {"ANTHROPIC_AUTH_TOKEN": "real"})

    updated = json.loads((job_path / "config.json").read_text(encoding="utf-8"))
    assert updated["agents"][0]["env"]["UNKNOWN_SECRET"] == "ab****xyz"


def test_agent_env_reads_profile_env_and_ignores_inherited(monkeypatch):
    profiles = SimpleNamespace(
        get_agent=lambda _agent_id: {
            "env": [
                {"key": "KEY_ONE", "value": "value-one"},
                {"key": "INHERITED_VAR", "value": None},
                {"key": "", "value": "x"},
            ]
        }
    )

    assert agent_env(profiles, "agent-1") == {"KEY_ONE": "value-one"}
    assert agent_env(profiles, None) == {}

    def _missing(_agent_id):
        raise KeyError("agent deleted")

    assert agent_env(SimpleNamespace(get_agent=_missing), "agent-1") == {}


def test_environment_env_reads_preset_and_ignores_inherited():
    profiles = SimpleNamespace(
        get_environment=lambda _preset_id: {
            "env": [
                {"key": "HF_TOKEN", "value": "hf-real"},
                {"key": "INHERITED_VAR", "value": None},
            ]
        }
    )

    assert environment_env(profiles, "preset-1") == {"HF_TOKEN": "hf-real"}
    assert environment_env(profiles, None) == {}

    def _missing(_preset_id):
        raise KeyError("preset deleted")

    assert environment_env(SimpleNamespace(get_environment=_missing), "preset-1") == {}


def test_restore_sensitive_env_restores_environment_env(tmp_path: Path, caplog):
    from ornnlab.services.webui_job_resume import _has_redacted

    job_path = tmp_path / "job"
    job_path.mkdir()
    (job_path / "config.json").write_text(
        json.dumps(
            {
                "agents": [{"env": {"ANTHROPIC_AUTH_TOKEN": "sk-d****b47"}}],
                "environment": {"env": {"HF_TOKEN": "hf****abcd"}},
                "verifier": {"env": {"VERIFIER_SECRET": "vs****wxyz"}},
            }
        ),
        encoding="utf-8",
    )

    restore_sensitive_env(
        job_path,
        {"ANTHROPIC_AUTH_TOKEN": "sk-real-token"},
        environment_env={"HF_TOKEN": "hf-real-token"},
    )

    updated = json.loads((job_path / "config.json").read_text(encoding="utf-8"))
    assert updated["agents"][0]["env"]["ANTHROPIC_AUTH_TOKEN"] == "sk-real-token"
    assert updated["environment"]["env"]["HF_TOKEN"] == "hf-real-token"
    assert updated["verifier"]["env"]["VERIFIER_SECRET"] == "vs****wxyz"
    assert any("verifier_env_redacted_unrestorable" in record.message for record in caplog.records)
    assert _has_redacted({"A": "x****y"}) is True
    assert _has_redacted({"A": "plain"}) is False


def _iter_cmdlines(*cmdlines: list[str]):
    return iter(SimpleNamespace(info={"cmdline": cmdline}) for cmdline in cmdlines)


def test_live_harbor_process_matches_run_config_by_content(tmp_path: Path, monkeypatch):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    config = job_path.parent / "harbor.config.json"
    config.write_text(
        json.dumps({"job_name": "run-some-job", "jobs_dir": str(job_path.parent)}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(config)]),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_matches_temp_runtime_config(tmp_path: Path, monkeypatch):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    runtime_config = tmp_path / ".harbor.runtime.config.json"
    runtime_config.write_text(
        json.dumps({"job_name": "run-some-job", "jobs_dir": str(job_path.parent)}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(runtime_config)]),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_fails_closed_on_shared_config_of_sibling_job(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    config = job_path.parent / "harbor.config.json"
    config.write_text(
        json.dumps({"job_name": "run-other-job", "jobs_dir": str(job_path.parent)}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(config)]),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_ignores_config_in_unrelated_jobs_dir(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs-a" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    other = tmp_path / "jobs-b" / "harbor.config.json"
    other.parent.mkdir(parents=True)
    other.write_text(
        json.dumps({"job_name": "other", "jobs_dir": str(other.parent)}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(other)]),
    )

    assert _live_harbor_process_for(job_path) is False


def test_live_harbor_process_fails_closed_on_unreadable_same_dir_config(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    missing = job_path.parent / "harbor.config.json"
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(missing)]),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_uses_live_sidecar_process(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    sidecar = job_path.parent / ".ornnlab-run-some-job.pid"
    sidecar.write_text(json.dumps({"pid": 4242, "start_time": 100.0}), encoding="utf-8")
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.Process",
        lambda pid: SimpleNamespace(create_time=lambda: 100.0),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_treats_dead_sidecar_as_authoritative(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    sidecar = job_path.parent / ".ornnlab-run-some-job.pid"
    sidecar.write_text(json.dumps({"pid": 4242, "start_time": 100.0}), encoding="utf-8")
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.Process",
        lambda pid: (_ for _ in ()).throw(psutil.NoSuchProcess(pid)),
    )

    assert _live_harbor_process_for(job_path) is False


def test_live_harbor_process_detects_pid_reuse(tmp_path: Path, monkeypatch):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    sidecar = job_path.parent / ".ornnlab-run-some-job.pid"
    sidecar.write_text(json.dumps({"pid": 4242, "start_time": 100.0}), encoding="utf-8")
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.Process",
        lambda pid: SimpleNamespace(create_time=lambda: 999.0),
    )

    assert _live_harbor_process_for(job_path) is False


def test_live_harbor_process_uses_legacy_sidecar_location(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs"  # legacy: job_path IS the jobs dir
    job_path.mkdir(parents=True)
    sidecar = job_path / ".ornnlab-run-some-job.pid"
    sidecar.write_text(json.dumps({"pid": 4242, "start_time": 100.0}), encoding="utf-8")
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.Process",
        lambda pid: SimpleNamespace(create_time=lambda: 100.0),
    )

    assert _live_harbor_process_for(job_path, job_name="run-some-job") is True


def test_live_harbor_process_matches_legacy_layout_config(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs"  # legacy: job_path IS the jobs dir
    job_path.mkdir(parents=True)
    config = job_path / "harbor.config.json"
    config.write_text(
        json.dumps({"job_name": "jobs", "jobs_dir": str(job_path)}),
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(["harbor", "run", "--config", str(config)]),
    )

    assert _live_harbor_process_for(job_path) is True


def test_live_harbor_process_matches_resume_by_exact_job_path(
    tmp_path: Path, monkeypatch
):
    job_path = tmp_path / "jobs" / "run-some-job"
    job_path.parent.mkdir(parents=True)
    sibling = tmp_path / "jobs" / "run-sibling-job"
    monkeypatch.setattr(
        "ornnlab.services.webui_job_resume.psutil.process_iter",
        lambda *_: _iter_cmdlines(
            ["harbor", "job", "resume", "--job-path", str(job_path)],
            ["harbor", "job", "resume", "--job-path", str(sibling)],
        ),
    )

    assert _live_harbor_process_for(job_path) is True


def test_resume_harbor_job_forwards_merged_env(settings, tmp_path: Path, monkeypatch):
    captured: dict = {}

    class FakeProcess:
        returncode = 0

        async def communicate(self):
            return b"", None

    async def fake_spawn(*_args, **_kwargs):
        captured["env"] = _kwargs.get("env")
        return FakeProcess()

    monkeypatch.setattr(
        "ornnlab.services.webui_job_service.asyncio.create_subprocess_exec", fake_spawn
    )
    service = WebUiJobService(
        settings, WebUiOperationService(settings, {}), object()
    )

    asyncio.run(
        service._resume_harbor_job(
            tmp_path, env={**os.environ, "AGENT_KEY": "agent-value"}
        )
    )

    assert captured["env"]["AGENT_KEY"] == "agent-value"
    assert "PATH" in captured["env"]
