from __future__ import annotations

import pytest

from ornnlab.services.model_pricing import calculate_cost, pricing_snapshot


def test_custom_pricing_calculates_cache_aware_cost() -> None:
    snapshot = pricing_snapshot(
        {
            "modelPricing": [
                {
                    "modelName": "custom-model",
                    "source": "custom",
                    "inputCacheMissUsdPerMillion": 5,
                    "inputCacheHitUsdPerMillion": 0.5,
                    "outputUsdPerMillion": 25,
                }
            ]
        },
        "custom-model",
    )

    cost = calculate_cost(
        {
            "n_input_tokens": 2_546_646,
            "n_cache_tokens": 2_491_904,
            "n_output_tokens": 14_838,
            "cost_usd": 999,
        },
        snapshot,
    )

    assert cost == pytest.approx(1.890612)


def test_reported_pricing_preserves_harness_cost() -> None:
    assert calculate_cost({"cost_usd": 1.25}, {"source": "reported"}) == 1.25


def test_cache_aware_pricing_requires_cache_usage(caplog) -> None:
    cost = calculate_cost(
        {"n_input_tokens": 100, "n_output_tokens": 20},
        {
            "modelName": "custom-model",
            "source": "custom",
            "inputCacheMissUsdPerMillion": 1,
            "inputCacheHitUsdPerMillion": 0.1,
            "outputUsdPerMillion": 2,
        },
    )

    assert cost is None
    assert "without cache usage" in caplog.text


def test_litellm_pricing_is_resolved_to_a_job_snapshot(monkeypatch) -> None:
    import litellm

    monkeypatch.setitem(
        litellm.model_cost,
        "test-provider/test-model",
        {
            "input_cost_per_token": 2e-6,
            "cache_read_input_token_cost": 0.2e-6,
            "output_cost_per_token": 8e-6,
            "source": "https://provider.example/pricing",
        },
    )

    snapshot = pricing_snapshot(
        {
            "modelPricing": [
                {"modelName": "test-provider/test-model", "source": "litellm"}
            ]
        },
        "test-provider/test-model",
    )

    assert snapshot == {
        "modelName": "test-provider/test-model",
        "source": "litellm",
        "catalogModelName": "test-provider/test-model",
        "inputCacheMissUsdPerMillion": 2,
        "inputCacheHitUsdPerMillion": 0.2,
        "outputUsdPerMillion": 8,
        "sourceUrl": "https://provider.example/pricing",
    }
