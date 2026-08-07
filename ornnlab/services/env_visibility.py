from __future__ import annotations

import json

from ornnlab.settings import Settings
from ornnlab.storage import sqlite

HIDDEN_ENV_KEYS_PREFERENCE = "hidden_env_keys"


def hidden_env_keys(settings: Settings) -> list[str]:
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn,
            "SELECT value FROM webui_system_preferences WHERE key = ?",
            (HIDDEN_ENV_KEYS_PREFERENCE,),
        )
    if not rows:
        return []
    try:
        parsed = json.loads(rows[0]["value"])
    except (ValueError, TypeError):
        return []
    if not isinstance(parsed, list):
        return []
    return [key for key in parsed if isinstance(key, str) and key]


def save_hidden_env_keys(settings: Settings, keys: list[str]) -> None:
    normalized = sorted(dict.fromkeys(keys))
    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO webui_system_preferences(key, value, updated_at) "
            "VALUES (?, ?, CURRENT_TIMESTAMP) ON CONFLICT(key) DO UPDATE SET "
            "value = excluded.value, updated_at = excluded.updated_at",
            (HIDDEN_ENV_KEYS_PREFERENCE, json.dumps(normalized)),
        )
