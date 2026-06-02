from __future__ import annotations

import subprocess


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


def empty_snapshot() -> dict[str, set[int]]:
    return {"pids": set(), "pgids": set()}


def clone_snapshot(snapshot: dict[str, set[int]]) -> dict[str, set[int]]:
    return {"pids": set(snapshot["pids"]), "pgids": set(snapshot["pgids"])}


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


def live_all_pids() -> set[int]:
    return {row.pid for row in process_rows() if not row.is_zombie}


def live_harnesslab_agent_token_pids() -> set[int]:
    completed = subprocess.run(
        ["ps", "eww", "-axo", "pid=,command="],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    pids = set()
    for line in completed.stdout.splitlines():
        if "HARNESSLAB_AGENT_RUN_TOKEN=" not in line:
            continue
        parts = line.split(None, 1)
        if not parts:
            continue
        try:
            pids.add(int(parts[0]))
        except ValueError:
            continue
    return pids
