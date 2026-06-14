from __future__ import annotations

import hashlib
from collections.abc import Iterable


def experiment_kind(agent_count: int, benchmark_count: int) -> str:
    if agent_count > 1:
        return "comparison"
    if benchmark_count > 1:
        return "batch"
    return "single"


def stable_hash(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]


def unique_preserving_order(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def derive_experiment_status(statuses: Iterable[str]) -> str:
    unique = set(statuses)
    if unique == {"completed"}:
        return "completed"
    if "completed" in unique and "failed" in unique:
        return "partially_failed"
    if "failed" in unique:
        return "failed"
    if "cancelled" in unique:
        return "cancelled"
    if "interrupted" in unique:
        return "interrupted"
    if "running" in unique:
        return "running"
    if "draft" in unique:
        return "draft"
    return "queued"
