from __future__ import annotations

import os
import select
import signal
import subprocess
import sys
import threading
import time
import uuid
from pathlib import Path

from ornnlab_tb_ps import (
    clone_snapshot,
    empty_snapshot,
    live_all_pids,
    live_ornnlab_agent_token_pids,
    live_snapshot_pids,
    live_token_pids,
    merge_process_snapshots,
    process_tree_snapshot,
    snapshot_alive,
)


def run_agent_process(
    command: str,
    stdin: str | None,
    timeout: int,
    cleanup_log_path: Path | None = None,
) -> subprocess.CompletedProcess:
    token = f"ornnlab-agent-{uuid.uuid4().hex}"
    baseline_pids = live_all_pids()
    process = subprocess.Popen(
        supervised_shell_command(command),
        stdin=subprocess.PIPE if stdin is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        shell=True,
        start_new_session=True,
        env=agent_env(token),
    )
    tracker = ProcessTracker(process.pid, token, baseline_pids)
    tracker.start()
    try:
        try:
            stdout, stderr = process.communicate(input=stdin, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            tracker.stop()
            cleanup = terminate_process_tree(
                process,
                token,
                tracker.snapshot(),
                tracker.baseline_pids,
            )
            record_cleanup("after timeout", cleanup, cleanup_log_path)
            stdout, stderr = communicate_after_timeout(process)
            raise AgentCommandTimedOut(
                configured_timeout=timeout,
                cleanup=cleanup,
                stdout=stdout or decode_timeout_output(error.output),
                stderr=stderr or decode_timeout_output(error.stderr),
            ) from error
        tracker.stop()
        cleanup = terminate_lingering_processes(
            process,
            token,
            tracker.snapshot(),
            tracker.baseline_pids,
        )
    finally:
        tracker.stop()
    if cleanup:
        record_cleanup("after exit", cleanup, cleanup_log_path)
        if not cleanup.succeeded:
            raise RuntimeError(
                "agent command exited but left live child processes: "
                + cleanup.message()
            )
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def agent_env(token: str) -> dict[str, str]:
    env = os.environ.copy()
    env["ORNNLAB_AGENT_RUN_TOKEN"] = token
    return env


def supervised_shell_command(command: str) -> str:
    return "sleep 0.05; " + command


def record_cleanup(
    phase: str,
    cleanup: "CleanupResult",
    cleanup_log_path: Path | None,
) -> None:
    message = f"ornnlab agent cleanup {phase}: {cleanup.message()}"
    print(message, file=sys.stderr)
    if cleanup_log_path is None:
        return
    cleanup_log_path.parent.mkdir(parents=True, exist_ok=True)
    with cleanup_log_path.open("a", encoding="utf-8") as handle:
        handle.write(message + "\n")


class AgentCommandTimedOut(TimeoutError):
    def __init__(
        self,
        configured_timeout: int,
        cleanup: CleanupResult,
        stdout: str | None,
        stderr: str | None,
    ) -> None:
        self.configured_timeout = configured_timeout
        self.cleanup = cleanup
        self.stdout = stdout or ""
        self.stderr = stderr or ""
        super().__init__(
            "agent command timed out; "
            f"configured_timeout_sec={configured_timeout}; cleanup={cleanup.message()}"
        )

    @property
    def cleanup_succeeded(self) -> bool:
        return self.cleanup.succeeded


class CleanupResult:
    def __init__(
        self,
        root_pid: int,
        pids: set[int],
        pgids: set[int],
        token: str,
        token_survivors: set[int],
        alive_pids: set[int],
    ) -> None:
        self.root_pid = root_pid
        self.pids = pids
        self.pgids = pgids
        self.token = token
        self.token_survivors = token_survivors
        self.alive_pids = alive_pids

    @property
    def succeeded(self) -> bool:
        return not self.token_survivors and not self.alive_pids

    def message(self) -> str:
        return (
            f"root_pid={self.root_pid} pids={sorted(self.pids)} "
            f"pgids={sorted(self.pgids)} token={self.token} "
            f"token_survivors={sorted(self.token_survivors)} "
            f"alive_pids={sorted(self.alive_pids)} "
            f"succeeded={self.succeeded}"
        )


class ProcessTracker:
    def __init__(
        self,
        root_pid: int,
        token: str,
        baseline_pids: set[int],
        startup_interval: float = 0.001,
        steady_interval: float = 0.05,
        startup_window: float = 1.0,
    ) -> None:
        self.root_pid = root_pid
        self.token = token
        self.baseline_pids = baseline_pids
        self.startup_interval = startup_interval
        self.steady_interval = steady_interval
        self.startup_deadline = time.monotonic() + startup_window
        self._snapshot = empty_snapshot()
        self._lock = threading.Lock()
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._kqueue = create_process_kqueue()
        self._registered_pids: set[int] = set()

    def start(self) -> None:
        self._capture_once()
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        if self._thread.is_alive():
            self._thread.join(timeout=1.0)
        self._capture_once()
        if self._kqueue is not None:
            self._kqueue.close()
            self._kqueue = None

    def snapshot(self) -> dict[str, set[int]]:
        with self._lock:
            return clone_snapshot(self._snapshot)

    def _run(self) -> None:
        while not self._stop.is_set():
            self._capture_once()
            interval = (
                self.startup_interval
                if time.monotonic() < self.startup_deadline
                else self.steady_interval
            )
            self._wait_for_activity(interval)

    def _capture_once(self) -> None:
        snapshot = process_tree_snapshot(self.root_pid, self.token)
        with self._lock:
            self._snapshot = merge_process_snapshots(self._snapshot, snapshot)
            observed = clone_snapshot(self._snapshot)
        self._register_process_events(observed["pids"])

    def _wait_for_activity(self, timeout: float) -> None:
        if self._kqueue is None:
            self._stop.wait(timeout)
            return
        try:
            self._kqueue.control(None, 100, timeout)
        except OSError:
            self._stop.wait(timeout)

    def _register_process_events(self, pids: set[int]) -> None:
        if self._kqueue is None:
            return
        for pid in sorted(pids - self._registered_pids):
            if pid <= 0:
                continue
            event = select.kevent(
                pid,
                filter=select.KQ_FILTER_PROC,
                flags=select.KQ_EV_ADD | select.KQ_EV_ENABLE,
                fflags=select.KQ_NOTE_FORK | select.KQ_NOTE_EXIT,
            )
            try:
                self._kqueue.control([event], 0, 0)
                self._registered_pids.add(pid)
            except OSError:
                pass


def create_process_kqueue():
    if not hasattr(select, "kqueue"):
        return None
    try:
        return select.kqueue()
    except OSError:
        return None


def terminate_process_tree(
    process: subprocess.Popen,
    token: str,
    observed_snapshot: dict[str, set[int]] | None = None,
    baseline_pids: set[int] | None = None,
) -> CleanupResult:
    snapshot = merge_process_snapshots(
        observed_snapshot or empty_snapshot(),
        process_tree_snapshot(process.pid, token),
    )
    signal_process_snapshot(snapshot, signal.SIGTERM)
    wait_for_process_exit(process, 2.0)
    wait_for_snapshot_exit(snapshot, 2.0)
    final = merge_process_snapshots(snapshot, process_tree_snapshot(process.pid, token))
    if snapshot_alive(final):
        signal_process_snapshot(final, signal.SIGKILL)
        wait_for_process_exit(process, 2.0)
        wait_for_snapshot_exit(final, 2.0)
        final = merge_process_snapshots(final, process_tree_snapshot(process.pid, token))
    token_survivors = live_token_pids(token)
    alive_pids = live_snapshot_pids(final) | escaped_process_pids(
        baseline_pids or set(),
        final,
    )
    return CleanupResult(
        root_pid=process.pid,
        pids=final["pids"],
        pgids=final["pgids"],
        token=token,
        token_survivors=token_survivors,
        alive_pids=alive_pids,
    )


def terminate_lingering_processes(
    process: subprocess.Popen,
    token: str,
    observed_snapshot: dict[str, set[int]] | None = None,
    baseline_pids: set[int] | None = None,
) -> CleanupResult | None:
    snapshot = merge_process_snapshots(
        observed_snapshot or empty_snapshot(),
        process_tree_snapshot(process.pid, token),
    )
    escaped_pids = escaped_process_pids(baseline_pids or set(), snapshot)
    if not snapshot_alive(snapshot) and not escaped_pids:
        return None
    if not snapshot_alive(snapshot):
        return CleanupResult(
            root_pid=process.pid,
            pids=snapshot["pids"],
            pgids=snapshot["pgids"],
            token=token,
            token_survivors=set(),
            alive_pids=escaped_pids,
        )
    return terminate_process_tree(process, token, snapshot, baseline_pids)


def signal_process_snapshot(snapshot: dict[str, set[int]], sig: int) -> None:
    current_pgid = os.getpgrp()
    for pgid in sorted(snapshot["pgids"]):
        if pgid == current_pgid:
            continue
        try:
            os.killpg(pgid, sig)
        except (ProcessLookupError, PermissionError):
            pass
    for pid in sorted(snapshot["pids"]):
        try:
            os.kill(pid, sig)
        except (ProcessLookupError, PermissionError):
            pass


def wait_for_process_exit(process: subprocess.Popen, timeout: float) -> None:
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        pass


def wait_for_snapshot_exit(snapshot: dict[str, set[int]], timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not snapshot_alive(snapshot):
            return
        time.sleep(0.05)


def escaped_process_pids(
    baseline_pids: set[int],
    known_snapshot: dict[str, set[int]],
) -> set[int]:
    if os.environ.get("ORNNLAB_AGENT_STRICT_GLOBAL_PROCESS_SCAN") != "1":
        return set()
    if not baseline_pids:
        return set()
    return (
        live_all_pids()
        - baseline_pids
        - known_snapshot["pids"]
        - live_ornnlab_agent_token_pids()
        - {os.getpid()}
    )


def communicate_after_timeout(process: subprocess.Popen) -> tuple[str, str]:
    try:
        stdout, stderr = process.communicate(timeout=1.0)
    except subprocess.TimeoutExpired:
        return "", ""
    return stdout or "", stderr or ""


def decode_timeout_output(output) -> str:
    if output is None:
        return ""
    if isinstance(output, bytes):
        return output.decode(errors="replace")
    return str(output)
