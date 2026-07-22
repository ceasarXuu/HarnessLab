from __future__ import annotations

import json
import logging
from pathlib import Path

from ornnlab.services.event_payload_security import sanitize_event_payload
from ornnlab.settings import Settings
from ornnlab.storage import sqlite
from ornnlab.storage.paths import atomic_write_text

logger = logging.getLogger(__name__)


def redact_historical_event_payloads(settings: Settings) -> dict[str, int]:
    database_events = _redact_database(settings)
    mirror_files = 0
    mirror_events = 0
    for path in settings.experiments_dir.glob("*/ornnlab-events.jsonl"):
        changed = _redact_mirror(path)
        if changed:
            mirror_files += 1
            mirror_events += changed
    result = {
        "databaseEvents": database_events,
        "mirrorFiles": mirror_files,
        "mirrorEvents": mirror_events,
    }
    if any(result.values()):
        logger.warning("event_history.redacted counts=%s", result)
    else:
        logger.debug("event_history.redaction_not_needed")
    return result


def _redact_database(settings: Settings) -> int:
    changed = 0
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn,
            "SELECT id, event_type, payload_json FROM experiment_events",
        )
        for row in rows:
            payload = _json_object(row["payload_json"])
            sanitized = sanitize_event_payload(str(row["event_type"]), payload)
            if sanitized == payload:
                continue
            conn.execute(
                "UPDATE experiment_events SET payload_json = ? WHERE id = ?",
                (_compact_json(sanitized), row["id"]),
            )
            changed += 1
    return changed


def _redact_mirror(path: Path) -> int:
    if path.is_symlink() or not path.is_file():
        return 0
    original = path.read_text(encoding="utf-8")
    output: list[str] = []
    changed = 0
    for line in original.splitlines():
        record = _json_object(line)
        payload = record.get("payload")
        event_type = record.get("event_type")
        if isinstance(payload, dict) and isinstance(event_type, str):
            sanitized = sanitize_event_payload(event_type, payload)
            if sanitized != payload:
                record["payload"] = sanitized
                changed += 1
        output.append(json.dumps(record, sort_keys=True) if record else line)
    if changed:
        suffix = "\n" if original.endswith("\n") else ""
        atomic_write_text(path, "\n".join(output) + suffix)
    return changed


def _json_object(value: str) -> dict:
    try:
        payload = json.loads(value)
    except (json.JSONDecodeError, TypeError):
        return {}
    return payload if isinstance(payload, dict) else {}


def _compact_json(payload: dict) -> str:
    return json.dumps(payload, sort_keys=True, separators=(",", ":"))
