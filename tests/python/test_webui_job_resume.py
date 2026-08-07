from __future__ import annotations

import asyncio
import json
import os
import sys
from pathlib import Path
from types import SimpleNamespace

from ornnlab.services.webui_job_resume import (
    agent_env,
    cleanup_resume_leftovers,
    clear_stale_job_lock,
    restore_sensitive_env,
)
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
