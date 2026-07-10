from __future__ import annotations

import asyncio
import json
import os
import shutil
import signal
import sys
from pathlib import Path
from typing import Any

from ornnlab.models.harbor import HarborJobConfigView
from ornnlab.services.command_line import split_command
from ornnlab.services.harbor_paths import resolve_harbor_result_path
from ornnlab.storage.paths import atomic_write_text, ensure_parent

JOB_LOG_NAME = "job.log"
CLEANUP_FILE_NAME = "harbor.cleanup.json"
CONFIG_FILE_NAME = "harbor.config.json"


class ManagedSubprocessHarborRunner:
    def __init__(
        self,
        command: list[str] | None = None,
        terminate_grace_sec: float = 2.0,
    ):
        self.command = command or _command_from_env()
        self.terminate_grace_sec = terminate_grace_sec

    async def run(self, config: HarborJobConfigView) -> dict:
        job_dir = Path(config.jobs_dir)
        job_dir.mkdir(parents=True, exist_ok=True)
        log_path = job_dir / JOB_LOG_NAME
        config_path = job_dir / CONFIG_FILE_NAME
        process = await asyncio.create_subprocess_exec(
            *self.command,
            "--config",
            str(config_path),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            start_new_session=True,
        )
        output_task = asyncio.create_task(_mirror_stdout(process, log_path))
        try:
            return_code = await process.wait()
            output = await output_task
        except asyncio.CancelledError:
            cleanup = await _terminate_process_group(process, self.terminate_grace_sec)
            cleanup["reason"] = "task_cancelled"
            cleanup["command"] = self.command
            atomic_write_text(
                job_dir / CLEANUP_FILE_NAME,
                json.dumps(cleanup, indent=2, sort_keys=True),
            )
            output_task.cancel()
            await _ignore_cancelled(output_task)
            raise
        if return_code != 0:
            raise RuntimeError(f"harbor subprocess exited with {return_code}: {output[-400:]}")
        result_path = resolve_harbor_result_path(job_dir, config.job_name)
        result = _read_or_write_result(result_path, return_code)
        return {
            "status": _status_from_result_payload(result),
            "score": _score(result),
            "job_dir": str(job_dir),
            "result_path": str(result_path),
            "harbor_job_id": result.get("harbor_job_id"),
        }


def _command_from_env() -> list[str]:
    raw = os.environ.get("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", "harbor run")
    try:
        return split_command(raw)
    except ValueError as error:
        if str(error) != "command cannot be empty":
            raise
        raise ValueError("ORNNLAB_HARBOR_SUBPROCESS_COMMAND cannot be empty") from None


def harbor_cli_executable() -> str:
    return (
        os.environ.get("ORNNLAB_HARBOR_CLI")
        or shutil.which("harbor")
        or str(Path(sys.executable).parent / "harbor")
    )


async def _mirror_stdout(
    process: asyncio.subprocess.Process,
    log_path: Path,
) -> str:
    ensure_parent(log_path)
    chunks: list[str] = []
    stream = process.stdout
    if stream is None:
        return ""
    while True:
        chunk = await stream.read(4096)
        if not chunk:
            break
        text = chunk.decode("utf-8", errors="replace")
        chunks.append(text)
        with log_path.open("a", encoding="utf-8", newline="") as handle:
            handle.write(text)
    return "".join(chunks)


async def _terminate_process_group(
    process: asyncio.subprocess.Process,
    grace_sec: float,
) -> dict[str, Any]:
    pid = process.pid
    cleanup: dict[str, Any] = {"pid": pid, "terminated": False, "killed": False}
    kill_process_group = getattr(os, "killpg", None)
    try:
        if kill_process_group is None:
            process.terminate()
        else:
            kill_process_group(pid, signal.SIGTERM)
        cleanup["terminated"] = True
    except ProcessLookupError:
        cleanup["missing"] = True
        cleanup["returncode"] = process.returncode
        return cleanup
    try:
        await asyncio.wait_for(process.wait(), timeout=grace_sec)
    except TimeoutError:
        try:
            force_kill = getattr(signal, "SIGKILL", None)
            if kill_process_group is None or force_kill is None:
                process.kill()
            else:
                kill_process_group(pid, force_kill)
            cleanup["killed"] = True
        except ProcessLookupError:
            cleanup["missing_after_term"] = True
        await process.wait()
    cleanup["returncode"] = process.returncode
    return cleanup


async def _ignore_cancelled(task: asyncio.Task[str]) -> None:
    try:
        await task
    except asyncio.CancelledError:
        return


def _read_or_write_result(path: Path, return_code: int) -> dict[str, Any]:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    result = {
        "status": "interrupted",
        "score": None,
        "subprocess_returncode": return_code,
        "failure_class": "harbor_protocol",
        "failure_code": "missing_result_json_after_success_exit",
        "warning": "harbor exited 0 but did not produce result.json",
    }
    atomic_write_text(path, json.dumps(result, indent=2, sort_keys=True))
    return result


def _score(result: dict[str, Any]) -> float | None:
    value = result.get("score")
    if isinstance(value, int | float):
        return float(value)
    return None


def _status_from_result_payload(result: dict[str, Any]) -> str:
    """Map Harbor's CLI result JSON to the same terminal states as its Python API."""
    explicit_status = result.get("status")
    if isinstance(explicit_status, str) and explicit_status in {
        "completed",
        "failed",
        "cancelled",
        "interrupted",
    }:
        return explicit_status

    stats = result.get("stats")
    if not isinstance(stats, dict):
        return "completed"
    if _positive_int(stats.get("n_cancelled_trials")):
        return "cancelled"
    if _positive_int(stats.get("n_errored_trials")):
        return "failed"

    total = _positive_int(result.get("n_total_trials"))
    completed = _positive_int(stats.get("n_completed_trials"))
    return "completed" if total == 0 or completed >= total else "interrupted"


def _positive_int(value: Any) -> int:
    return value if isinstance(value, int) and value > 0 else 0
