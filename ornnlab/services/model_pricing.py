from __future__ import annotations

import logging
from typing import Any

logger = logging.getLogger(__name__)

RATE_FIELDS = (
    "inputCacheMissUsdPerMillion",
    "inputCacheHitUsdPerMillion",
    "outputUsdPerMillion",
)


def pricing_snapshot(agent: dict[str, Any], model_name: str) -> dict[str, Any]:
    configured = next(
        (
            item
            for item in agent.get("modelPricing", [])
            if isinstance(item, dict) and item.get("modelName") == model_name
        ),
        {"modelName": model_name, "source": "reported"},
    )
    source = configured.get("source", "reported")
    if source == "reported":
        return {"modelName": model_name, "source": source}
    if source == "custom":
        return {
            "modelName": model_name,
            "source": source,
            **{field: float(configured[field]) for field in RATE_FIELDS},
        }
    if source != "litellm":
        raise ValueError(f"unsupported model pricing source: {source}")
    return _litellm_snapshot(model_name)


def calculate_cost(usage: dict[str, Any], snapshot: dict[str, Any] | None) -> float | None:
    if not snapshot or snapshot.get("source") == "reported":
        return _number_or_none(usage.get("cost_usd"))

    total_input = _number_or_none(usage.get("n_input_tokens"))
    output = _number_or_none(usage.get("n_output_tokens"))
    if total_input is None or output is None:
        return None

    miss_rate = _number_or_none(snapshot.get("inputCacheMissUsdPerMillion"))
    hit_rate = _number_or_none(snapshot.get("inputCacheHitUsdPerMillion"))
    output_rate = _number_or_none(snapshot.get("outputUsdPerMillion"))
    if miss_rate is None or hit_rate is None or output_rate is None:
        return None

    cached = _number_or_none(usage.get("n_cache_tokens"))
    if cached is None:
        if hit_rate != miss_rate:
            logger.warning(
                "Cannot calculate cache-aware model cost without cache usage",
                extra={
                    "model_name": snapshot.get("modelName"),
                    "pricing_source": snapshot.get("source"),
                },
            )
            return None
        cached = 0.0
    cached = min(max(cached, 0.0), total_input)
    uncached = total_input - cached
    return (
        uncached * miss_rate + cached * hit_rate + output * output_rate
    ) / 1_000_000


def _litellm_snapshot(model_name: str) -> dict[str, Any]:
    try:
        import litellm
    except ImportError as exc:
        raise ValueError("LiteLLM pricing is unavailable") from exc

    candidates = (model_name, model_name.split("/", 1)[-1])
    matched_name = next((name for name in candidates if litellm.model_cost.get(name)), None)
    if matched_name is None:
        raise ValueError(f"LiteLLM has no pricing entry for model '{model_name}'")
    pricing = litellm.model_cost[matched_name]
    input_rate = _number_or_none(pricing.get("input_cost_per_token"))
    cache_value = pricing.get("cache_read_input_token_cost")
    if cache_value is None:
        cache_value = pricing.get("input_cost_per_token_cache_hit")
    cache_rate = _number_or_none(cache_value)
    output_rate = _number_or_none(pricing.get("output_cost_per_token"))
    if input_rate is None or output_rate is None:
        raise ValueError(f"LiteLLM pricing for model '{model_name}' is incomplete")
    if cache_rate is None:
        cache_rate = input_rate
    snapshot = {
        "modelName": model_name,
        "source": "litellm",
        "catalogModelName": matched_name,
        "inputCacheMissUsdPerMillion": round(input_rate * 1_000_000, 12),
        "inputCacheHitUsdPerMillion": round(cache_rate * 1_000_000, 12),
        "outputUsdPerMillion": round(output_rate * 1_000_000, 12),
        "sourceUrl": pricing.get("source"),
    }
    logger.info(
        "LiteLLM pricing snapshot resolved",
        extra={
            "catalog_model_name": matched_name,
            "model_name": model_name,
            "pricing_source": "litellm",
        },
    )
    return snapshot


def _number_or_none(value: object) -> float | None:
    return float(value) if isinstance(value, int | float) else None
