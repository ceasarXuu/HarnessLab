from __future__ import annotations

from typing import Any

_REDACTED = "[REDACTED]"
_SENSITIVE_KEYS = {
    "authorization",
    "cookie",
    "credential",
    "credentials",
    "password",
    "set_cookie",
    "secret",
    "token",
    "api_key",
    "apikey",
}
_SENSITIVE_KEY_SUFFIXES = (
    "_api_key",
    "_apikey",
    "_auth_token",
    "_access_token",
    "_refresh_token",
    "_token",
    "_secret",
    "_password",
    "_credential",
    "_credentials",
)


def sanitize_event_payload(event_type: str, payload: dict) -> dict:
    sanitized = dict(payload)
    if event_type == "harbor.job.running":
        sanitized.pop("config", None)
    return _redact_value(sanitized)


def _redact_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {
            str(key): _REDACTED if _sensitive_key(str(key)) else _redact_value(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [_redact_value(item) for item in value]
    return value


def _sensitive_key(key: str) -> bool:
    normalized = key.casefold().replace("-", "_")
    return normalized in _SENSITIVE_KEYS or normalized.endswith(_SENSITIVE_KEY_SUFFIXES)
