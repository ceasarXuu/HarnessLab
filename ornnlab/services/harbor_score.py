from __future__ import annotations


def pass_at_one(values: object) -> float | None:
    """Read Harbor's pass@1 from native or JSON-deserialized result data."""
    if not isinstance(values, dict):
        return None
    for key in (1, "1"):
        value = values.get(key)
        if isinstance(value, int | float):
            return float(value)
    return None


def result_pass_at_one(result: object) -> float | None:
    if not isinstance(result, dict):
        return None
    stats = result.get("stats")
    evals = stats.get("evals") if isinstance(stats, dict) else None
    if not isinstance(evals, dict):
        return None
    for dataset_stats in evals.values():
        if isinstance(dataset_stats, dict):
            score = pass_at_one(dataset_stats.get("pass_at_k"))
            if score is not None:
                return score
    return None
