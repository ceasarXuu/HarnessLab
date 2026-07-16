from __future__ import annotations

import json
import logging
from typing import Any

logger = logging.getLogger(__name__)


def normalize_agent_profile(agent: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(agent)
    if normalized.get("harness") != "claude-code":
        return normalized

    env = list(normalized.get("env", []))
    legacy = next((item for item in env if item.get("key") == "MAX_THINKING_TOKENS"), None)
    if legacy is None:
        return normalized

    normalized["env"] = [
        item for item in env if item.get("key") != "MAX_THINKING_TOKENS"
    ]
    value = legacy.get("value")
    if value is not None and str(value).strip():
        kwargs = _parse_kwargs(normalized.get("kwargs", ""))
        kwargs.setdefault("max_thinking_tokens", str(value).strip())
        normalized["kwargs"] = json.dumps(kwargs, sort_keys=True)
    logger.debug(
        "Normalized legacy Claude Code thinking configuration",
        extra={"agent_id": normalized.get("id")},
    )
    return normalized


def _parse_kwargs(value: str) -> dict[str, Any]:
    if not value.strip() or value.strip().lower() == "none":
        return {}
    try:
        parsed = json.loads(value)
    except json.JSONDecodeError:
        return dict(entry.split("=", 1) for entry in value.splitlines() if "=" in entry)
    return parsed if isinstance(parsed, dict) else {}
