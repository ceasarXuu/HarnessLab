from __future__ import annotations

from datetime import datetime

TERMINAL_STATUSES = {"completed", "failed", "cancelled", "interrupted"}


def trial_progress_total(row: dict) -> int:
    return (int(row["n_tasks"]) if row.get("n_tasks") is not None else 0) * max(
        1, int(row["n_attempts"])
    )


def execution_completed(result: dict, expected_total: int) -> bool:
    """True when Harbor's native result shows every trial executed.

    Outcome-agnostic: errored/not-passed trials still count as executed, matching
    Harbor's execution model (execution state and outcome quality are separate).
    """
    if not result:
        return False
    total = result.get("n_total_trials")
    if not isinstance(total, int) or total <= 0:
        return False
    stats = result.get("stats")
    completed = stats.get("n_completed_trials") if isinstance(stats, dict) else None
    if not isinstance(completed, int):
        return False
    return completed >= total


def job_trial_progress(
    result: dict,
    *,
    expected_total: int,
    terminal_without_result: bool = False,
) -> dict[str, int]:
    stats_value = result.get("stats")
    stats = stats_value if isinstance(stats_value, dict) else {}
    total = _non_negative_int(result.get("n_total_trials"), expected_total)
    terminal_count = _non_negative_int(stats.get("n_completed_trials"), 0)
    if not result and terminal_without_result:
        terminal_count = total
    errored = min(terminal_count, _non_negative_int(stats.get("n_errored_trials"), 0))
    completed = max(0, terminal_count - errored)
    passed = min(completed, _passed_trial_count(stats))
    return {
        "total": total,
        "completed": completed,
        "passed": passed,
        "notPassed": completed - passed,
        "errored": errored,
    }


def runtime_seconds(
    started: str | None,
    finished: str | None,
    *,
    now: datetime | None = None,
) -> int | None:
    if not started:
        return None
    start = datetime.fromisoformat(started)
    end = datetime.fromisoformat(finished) if finished else now or datetime.now().astimezone()
    if start.tzinfo is None and end.tzinfo is not None:
        end = end.replace(tzinfo=None)
    return max(0, int((end - start).total_seconds()))


def _passed_trial_count(stats: dict) -> int:
    passed: set[str] = set()
    evals = stats.get("evals")
    if not isinstance(evals, dict):
        return 0
    for evaluation in evals.values():
        if not isinstance(evaluation, dict):
            continue
        reward_stats = evaluation.get("reward_stats")
        if not isinstance(reward_stats, dict):
            continue
        for distribution in reward_stats.values():
            if not isinstance(distribution, dict):
                continue
            for reward, trial_names in distribution.items():
                try:
                    is_pass = float(reward) == 1
                except (TypeError, ValueError):
                    continue
                if is_pass and isinstance(trial_names, list):
                    passed.update(str(name) for name in trial_names)
    return len(passed)


def _non_negative_int(value: object, default: int) -> int:
    if isinstance(value, int) and not isinstance(value, bool) and value >= 0:
        return value
    return default
