from __future__ import annotations

import asyncio
import json
import logging
import os
import re
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

import psutil

from ornnlab.services.command_line import split_command
from ornnlab.services.container_proxy_runtime import RuntimeProxyPolicy
from ornnlab.settings import Settings
from ornnlab.storage import sqlite
from ornnlab.storage.paths import atomic_write_text

logger = logging.getLogger(__name__)

_PROXY_ENV_RUNTIME_NAMES = {
    "HTTP_PROXY": "ORNNLAB_CONTAINER_HTTP_PROXY",
    "http_proxy": "ORNNLAB_CONTAINER_HTTP_PROXY",
    "HTTPS_PROXY": "ORNNLAB_CONTAINER_HTTPS_PROXY",
    "https_proxy": "ORNNLAB_CONTAINER_HTTPS_PROXY",
    "ALL_PROXY": "ORNNLAB_CONTAINER_ALL_PROXY",
    "all_proxy": "ORNNLAB_CONTAINER_ALL_PROXY",
    "NO_PROXY": "ORNNLAB_CONTAINER_NO_PROXY",
    "no_proxy": "ORNNLAB_CONTAINER_NO_PROXY",
}


async def prepare_resume_proxy(
    proxy_runtime: Any, job_path: Path
) -> RuntimeProxyPolicy | None:
    """Start fresh host relays and rewrite the Job config to use them.

    Resumed Docker Jobs inherit the relay URL baked into ``config.json`` by the
    original run; the relay servers died with the previous backend process, so
    the Job must be re-prepared before ``harbor job resume`` starts containers.
    """
    config = _read_job_config_file(job_path)
    env = (config.get("environment") or {}).get("env")
    if not isinstance(env, dict) or not any(
        name in env for name in ("HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy")
    ):
        return None
    policy = await proxy_runtime.prepare_policy()
    if not policy.subprocess_env:
        await policy.close()
        return None
    rewrite_config_proxy_env(job_path, policy.subprocess_env)
    return policy


async def cleanup_resume_leftovers(job_path: Path) -> bool:
    """Chown root-owned Job leftovers via a short-lived root container.

    Interrupted Docker Jobs can leave files owned by root (container subprocesses);
    Harbor's resume cleanup cannot remove them. A root container on the bind-mounted
    Job directory restores ownership before ``harbor job resume`` runs.
    """
    if not hasattr(os, "getuid") or not _has_root_owned_files(job_path):
        return False
    command = _docker_command()
    process = await asyncio.create_subprocess_exec(
        *command,
        "run",
        "--rm",
        "-v",
        f"{job_path}:/work",
        "alpine",
        "chown",
        "-R",
        f"{os.getuid()}:{os.getgid()}",
        "/work",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.STDOUT,
    )
    output, _ = await process.communicate()
    if process.returncode != 0:
        logger.warning(
            "docker.resume_leftover_chown_failed path=%s exit=%s output=%s",
            job_path,
            process.returncode,
            (output or b"").decode("utf-8", errors="replace")[-300:],
        )
        return False
    logger.info("docker.resume_leftover_chown_completed path=%s", job_path)
    return True


def clear_stale_job_lock(
    settings: Settings,
    run_id: str,
    job_path: Path,
    job_name: str | None = None,
) -> bool:
    """Back up and remove a stale lock.json when the Job is provably dead.

    Harbor refuses to resume when the existing lock does not match the re-resolved
    Job lock (e.g. after a proxy relay rewrite). The lock is a concurrency guard, so
    it is only cleared when: no active OrnnLab operation references the run, no live
    Harbor process references the Job path, and no running OrnnLab-managed Docker
    containers remain for the run.
    """
    lock_path = job_path / "lock.json"
    if not lock_path.exists():
        return False
    if _active_operation_for_run(settings, run_id):
        logger.info(
            "docker.resume_lock_kept reason=active_operation run_id=%s", run_id
        )
        return False
    if _live_harbor_process_for(job_path, job_name):
        logger.info(
            "docker.resume_lock_kept reason=live_harbor_process path=%s", job_path
        )
        return False
    if _run_has_live_containers(settings, run_id):
        logger.info(
            "docker.resume_lock_kept reason=live_containers run_id=%s", run_id
        )
        return False
    backup = job_path / f"lock.json.bak-{int(time.time())}"
    shutil.copy2(lock_path, backup)
    lock_path.unlink()
    logger.info(
        "docker.resume_stale_lock_cleared path=%s backup=%s",
        job_path,
        backup.name,
    )
    return True


def active_resume_operation(settings: Settings, run_id: str) -> bool:
    return _active_operation_for_run(settings, run_id, operation_type="resume-job")


def _active_operation_for_run(
    settings: Settings, run_id: str, operation_type: str | None = None
) -> bool:
    if operation_type:
        query = (
            "SELECT id FROM webui_operations WHERE resource_id = ? "
            "AND operation_type = ? AND status IN ('queued', 'running')"
        )
        params: tuple[Any, ...] = (run_id, operation_type)
    else:
        query = (
            "SELECT id FROM webui_operations WHERE resource_id = ? "
            "AND status IN ('queued', 'running')"
        )
        params = (run_id,)
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(conn, query, params)
    return bool(rows)


def _live_harbor_process_for(job_path: Path, job_name: str | None = None) -> bool:
    for sidecar_path in _job_sidecar_candidates(job_path, job_name):
        if not sidecar_path.is_file():
            continue
        alive = _sidecar_process_alive(sidecar_path)
        if alive is not None:
            return alive
    try:
        for proc in psutil.process_iter(["cmdline"]):
            args = proc.info.get("cmdline") or []
            if not any("harbor" in arg.lower() for arg in args):
                continue
            if _cmdline_targets_job(args, job_path):
                return True
    except (psutil.Error, OSError):
        return True
    return False


def _job_sidecar_candidates(job_path: Path, job_name: str | None) -> list[Path]:
    name = _job_pid_sidecar_name(job_name or job_path.name)
    return [job_path.parent / name, job_path / name]


def _job_pid_sidecar_name(job_name: str) -> str:
    safe = re.sub(r"[^A-Za-z0-9._-]", "_", job_name)
    return f".ornnlab-{safe}.pid"


def _sidecar_process_alive(path: Path) -> bool | None:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    pid = payload.get("pid")
    if not isinstance(pid, int):
        return None
    try:
        proc = psutil.Process(pid)
    except psutil.NoSuchProcess:
        return False
    except (psutil.Error, OSError):
        return True
    start_time = payload.get("start_time")
    if isinstance(start_time, int | float):
        try:
            if abs(proc.create_time() - float(start_time)) > 1.0:
                return False
        except (psutil.Error, OSError):
            return True
    return True


def _cmdline_targets_job(args: list[str], job_path: Path) -> bool:
    if "--job-path" in args:
        index = args.index("--job-path")
        return index + 1 < len(args) and Path(args[index + 1]) == job_path
    if "--config" in args:
        index = args.index("--config")
        if index + 1 >= len(args):
            return True
        return _config_belongs_to_job(Path(args[index + 1]), job_path)
    return False


def _config_belongs_to_job(config_path: Path, job_path: Path) -> bool:
    payload = _read_config_file(config_path)
    if _config_matches_job(payload, job_path):
        return True
    config_text = config_path.as_posix()
    same_jobs_dir = config_text.startswith(f"{job_path.parent.as_posix()}/")
    if same_jobs_dir or "ornnlab-harbor-runtime" in config_text:
        logger.info(
            "docker.resume_lock_probe_inconclusive config=%s same_jobs_dir=%s "
            "unreadable=%s",
            config_text,
            same_jobs_dir,
            not payload,
        )
        return True
    return False


def _config_matches_job(payload: dict, job_path: Path) -> bool:
    if not payload:
        return False
    jobs_dir = payload.get("jobs_dir")
    return (
        payload.get("job_name") == job_path.name
        and jobs_dir in {str(job_path), str(job_path.parent)}
    )


def _run_has_live_containers(settings: Settings, run_id: str) -> bool:
    command = _docker_command()
    try:
        result = subprocess.run(
            [
                *command,
                "ps",
                "-aq",
                "--filter",
                "status=running",
                "--filter",
                "label=ornnlab.managed=true",
                "--filter",
                f"label=ornnlab.instance_id={settings.instance_id}",
                "--filter",
                f"label=ornnlab.run_id={run_id}",
            ],
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return True
    return bool(result.stdout.strip())


def _docker_command() -> list[str]:
    return split_command(os.environ.get("ORNNLAB_DOCKER_COMMAND", "docker"))


def _has_root_owned_files(job_path: Path) -> bool:
    uid = os.getuid()
    for root, dirs, files in os.walk(job_path):
        for name in dirs + files:
            try:
                if os.stat(os.path.join(root, name)).st_uid != uid:
                    return True
            except OSError:
                continue
    return False


def agent_env(profiles: Any, agent_id: str | None) -> dict[str, str]:
    if not agent_id:
        return {}
    try:
        agent = profiles.get_agent(agent_id)
    except KeyError:
        return {}
    return _profile_env(agent)


def environment_env(profiles: Any, preset_id: str | None) -> dict[str, str]:
    if not preset_id:
        return {}
    try:
        environment = profiles.get_environment(preset_id)
    except KeyError:
        return {}
    return _profile_env(environment)


def _profile_env(profile: Any) -> dict[str, str]:
    entries = profile.get("env") or []
    return {
        str(entry["key"]): str(entry["value"])
        for entry in entries
        if isinstance(entry, dict) and entry.get("key") and entry.get("value") is not None
    }


def restore_sensitive_env(
    job_path: Path,
    real_env: dict[str, str],
    environment_env: dict[str, str] | None = None,
) -> None:
    """Replace redacted sensitive values with the Agent's real values.

    Harbor's config serializer templates sensitive env vars when they match the
    host environment and redacts them otherwise; a resume whose process env lacks
    the secret would bake a redacted value into the Job config. Restore those
    values from the Agent and Environment profiles so the resumed trials receive
    real credentials.
    """
    for config_path in (job_path / "config.json", *job_path.glob("*/config.json")):
        config = _read_config_file(config_path)
        changed = False
        agents = config.get("agents")
        if isinstance(agents, list):
            for agent in agents:
                if isinstance(agent, dict) and _restore_env(agent.get("env"), real_env):
                    changed = True
        if isinstance(config.get("agent"), dict) and _restore_env(
            config["agent"].get("env"), real_env
        ):
            changed = True
        environment_env_value = (config.get("environment") or {}).get("env")
        if isinstance(environment_env_value, dict):
            if environment_env and _restore_env(environment_env_value, environment_env):
                changed = True
            elif _has_redacted(environment_env_value):
                logger.warning(
                    "docker.resume_environment_env_redacted_unrestorable path=%s",
                    config_path,
                )
        verifier_env = (config.get("verifier") or {}).get("env")
        if _has_redacted(verifier_env):
            logger.warning(
                "docker.resume_verifier_env_redacted_unrestorable path=%s", config_path
            )
        if changed:
            _write_config(config_path, config)


def _has_redacted(env: Any) -> bool:
    return isinstance(env, dict) and any(
        isinstance(value, str) and "****" in value for value in env.values()
    )


def _read_config_file(path: Path) -> dict:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _restore_env(env: Any, real_env: dict[str, str]) -> bool:
    if not isinstance(env, dict):
        return False
    changed = False
    for key, value in env.items():
        if isinstance(value, str) and "****" in value and key in real_env:
            env[key] = real_env[key]
            changed = True
    return changed


def rewrite_config_proxy_env(job_path: Path, subprocess_env: dict) -> None:
    config_path = job_path / "config.json"
    config = _read_job_config_file(job_path)
    env = (config.get("environment") or {}).get("env")
    if isinstance(env, dict) and _apply_proxy_env(env, subprocess_env):
        config["environment"]["env"] = env
        _write_config(config_path, config)
    for trial_dir in job_path.glob("*/"):
        trial_config_path = trial_dir / "config.json"
        trial_config = _read_job_config_file(trial_dir)
        trial_env = (trial_config.get("environment") or {}).get("env")
        if isinstance(trial_env, dict) and _apply_proxy_env(trial_env, subprocess_env):
            trial_config["environment"]["env"] = trial_env
            _write_config(trial_config_path, trial_config)


def _apply_proxy_env(env: dict, subprocess_env: dict) -> bool:
    changed = False
    for name, runtime_name in _PROXY_ENV_RUNTIME_NAMES.items():
        if (
            name in env
            and runtime_name in subprocess_env
            and env[name] != subprocess_env[runtime_name]
        ):
            env[name] = subprocess_env[runtime_name]
            changed = True
    return changed


def _write_config(path: Path, config: dict) -> None:
    atomic_write_text(
        path, json.dumps(config, indent=2, ensure_ascii=False) + "\n"
    )


def _read_job_config_file(job_path: Path) -> dict:
    try:
        payload = json.loads((job_path / "config.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}
