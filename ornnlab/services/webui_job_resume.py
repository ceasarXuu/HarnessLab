from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from ornnlab.services.container_proxy_runtime import RuntimeProxyPolicy

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
    path.write_text(
        json.dumps(config, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def _read_job_config_file(job_path: Path) -> dict:
    try:
        payload = json.loads((job_path / "config.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return payload if isinstance(payload, dict) else {}
