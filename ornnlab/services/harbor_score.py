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


def result_score(result: object) -> float | None:
    """Best-effort 0..1 score for a run result payload.

    Prefers pass@1, then falls back to the first proportional metric
    (``mean``/``reward``/``accuracy``) — e.g. a binary pass/fail run with a
    single attempt carries its success rate as ``mean``, and no pass@k.
    """
    score = result_pass_at_one(result)
    if score is not None:
        return score
    if not isinstance(result, dict):
        return None
    stats = result.get("stats")
    evals = stats.get("evals") if isinstance(stats, dict) else None
    if not isinstance(evals, dict):
        return None
    for dataset_stats in evals.values():
        if not isinstance(dataset_stats, dict):
            continue
        for metric in dataset_stats.get("metrics") or []:
            if not isinstance(metric, dict):
                continue
            for key in ("mean", "reward", "accuracy"):
                value = metric.get(key)
                if isinstance(value, int | float) and 0.0 <= float(value) <= 1.0:
                    return float(value)
    return None
