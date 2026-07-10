from __future__ import annotations

import asyncio
import json
import os
import shlex
import shutil
import signal
import sys
from pathlib import Path
from typing import Any

from ornnlab.models.harbor import HarborJobConfigView
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
        result_path = job_dir / "result.json"
        result = _read_or_write_result(result_path, return_code)
        return {
            "status": str(result.get("status", "completed")),
            "score": _score(result),
            "job_dir": str(job_dir),
            "result_path": str(result_path),
            "harbor_job_id": result.get("harbor_job_id"),
        }


def _command_from_env() -> list[str]:
    raw = os.environ.get("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", "harbor run")
    command = shlex.split(raw)
    if not command:
        raise ValueError("ORNNLAB_HARBOR_SUBPROCESS_COMMAND cannot be empty")
    return command


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
        with log_path.open("a", encoding="utf-8") as handle:
            handle.write(text)
    return "".join(chunks)


async def _terminate_process_group(
    process: asyncio.subprocess.Process,
    grace_sec: float,
) -> dict[str, Any]:
    pid = process.pid
    cleanup: dict[str, Any] = {"pid": pid, "terminated": False, "killed": False}
    try:
        os.killpg(pid, signal.SIGTERM)
        cleanup["terminated"] = True
    except ProcessLookupError:
        cleanup["missing"] = True
        cleanup["returncode"] = process.returncode
        return cleanup
    except AttributeError:
        process.terminate()
        cleanup["terminated"] = True
    try:
        await asyncio.wait_for(process.wait(), timeout=grace_sec)
    except TimeoutError:
        try:
            os.killpg(pid, signal.SIGKILL)
            cleanup["killed"] = True
        except ProcessLookupError:
            cleanup["missing_after_term"] = True
        except AttributeError:
            process.kill()
            cleanup["killed"] = True
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
