from __future__ import annotations

import os
import signal
import subprocess
import time
import uuid


def run_agent_process(
    command: str,
    stdin: str | None,
    timeout: int,
) -> subprocess.CompletedProcess:
    token = f"harnesslab-agent-{uuid.uuid4().hex}"
    process = subprocess.Popen(
        command,
        stdin=subprocess.PIPE if stdin is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        shell=True,
        start_new_session=True,
        env=agent_env(token),
    )
    try:
        stdout, stderr = process.communicate(input=stdin, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        cleanup = terminate_process_tree(process, token)
        stdout, stderr = communicate_after_timeout(process)
        raise AgentCommandTimedOut(
            configured_timeout=timeout,
            cleanup=cleanup,
            stdout=stdout or decode_timeout_output(error.output),
            stderr=stderr or decode_timeout_output(error.stderr),
        ) from error
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


def agent_env(token: str) -> dict[str, str]:
    env = os.environ.copy()
    env["HARNESSLAB_AGENT_RUN_TOKEN"] = token
    return env


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


class ProcessRow:
    def __init__(self, pid: int, ppid: int, pgid: int, stat: str, command: str) -> None:
        self.pid = pid
        self.ppid = ppid
        self.pgid = pgid
        self.stat = stat
        self.command = command

    @property
    def is_zombie(self) -> bool:
        return "Z" in self.stat


def terminate_process_tree(process: subprocess.Popen, token: str) -> CleanupResult:
    snapshot = process_tree_snapshot(process.pid, token)
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
    alive_pids = live_snapshot_pids(final)
    return CleanupResult(
        root_pid=process.pid,
        pids=final["pids"],
        pgids=final["pgids"],
        token=token,
        token_survivors=token_survivors,
        alive_pids=alive_pids,
    )


def process_tree_snapshot(root_pid: int, token: str | None = None) -> dict[str, set[int]]:
    rows = process_rows()
    by_parent: dict[int, list[ProcessRow]] = {}
    by_pid = {}
    for row in rows:
        by_parent.setdefault(row.ppid, []).append(row)
        by_pid[row.pid] = row
    pids: set[int] = set()
    pgids: set[int] = set()
    stack = [root_pid]
    if token:
        stack.extend(token_process_pids(token))
    while stack:
        pid = stack.pop()
        if pid in pids:
            continue
        pids.add(pid)
        row = by_pid.get(pid)
        if row and row.pgid > 0:
            pgids.add(row.pgid)
        for child in by_parent.get(pid, []):
            if child.pgid > 0:
                pgids.add(child.pgid)
            stack.append(child.pid)
    return {"pids": pids, "pgids": pgids}


def process_rows() -> list[ProcessRow]:
    completed = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,pgid=,stat=,command="],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    rows = []
    for line in completed.stdout.splitlines():
        parts = line.split(None, 4)
        if len(parts) != 5:
            continue
        try:
            rows.append(
                ProcessRow(
                    pid=int(parts[0]),
                    ppid=int(parts[1]),
                    pgid=int(parts[2]),
                    stat=parts[3],
                    command=parts[4],
                )
            )
        except ValueError:
            continue
    return rows


def token_process_pids(token: str) -> set[int]:
    completed = subprocess.run(
        ["ps", "eww", "-axo", "pid=,command="],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    pids = set()
    for line in completed.stdout.splitlines():
        if token not in line:
            continue
        parts = line.split(None, 1)
        if not parts:
            continue
        try:
            pids.add(int(parts[0]))
        except ValueError:
            continue
    return pids


def merge_process_snapshots(
    first: dict[str, set[int]],
    second: dict[str, set[int]],
) -> dict[str, set[int]]:
    return {
        "pids": set(first["pids"]) | set(second["pids"]),
        "pgids": set(first["pgids"]) | set(second["pgids"]),
    }


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


def snapshot_alive(snapshot: dict[str, set[int]]) -> bool:
    return bool(live_snapshot_pids(snapshot))


def live_snapshot_pids(snapshot: dict[str, set[int]]) -> set[int]:
    rows = {row.pid: row for row in process_rows()}
    live = set()
    for pid in snapshot["pids"]:
        row = rows.get(pid)
        if row and not row.is_zombie:
            live.add(pid)
    return live


def live_token_pids(token: str) -> set[int]:
    rows = {row.pid: row for row in process_rows()}
    live = set()
    for pid in token_process_pids(token):
        row = rows.get(pid)
        if row and not row.is_zombie:
            live.add(pid)
    return live


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
