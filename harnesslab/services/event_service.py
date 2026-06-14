from __future__ import annotations

import json
from pathlib import Path

from harnesslab.models.events import EventRecord
from harnesslab.services.clock import now_iso
from harnesslab.settings import Settings
from harnesslab.storage import sqlite
from harnesslab.storage.paths import atomic_write_text, ensure_parent


class EventService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def append(
        self,
        aggregate_type: str,
        aggregate_id: str,
        event_type: str,
        payload: dict,
        severity: str = "info",
    ) -> EventRecord:
        ts = now_iso()
        body = json.dumps(payload, sort_keys=True, separators=(",", ":"))
        with sqlite.connect(self.settings) as conn:
            cursor = conn.execute(
                "INSERT INTO experiment_events("
                "aggregate_type, aggregate_id, ts, event_type, severity, payload_json"
                ") VALUES (?, ?, ?, ?, ?, ?)",
                (aggregate_type, aggregate_id, ts, event_type, severity, body),
            )
            event_id = cursor.lastrowid
            if event_id is None:
                raise RuntimeError("SQLite did not return an event id")
        self._mirror(event_id, aggregate_type, aggregate_id, event_type, severity, ts, payload)
        return EventRecord(
            id=event_id,
            aggregate_type=aggregate_type,
            aggregate_id=aggregate_id,
            ts=ts,
            event_type=event_type,
            severity=severity,
            payload=payload,
        )

    def list_after(self, aggregate_id: str, after: int = 0) -> list[EventRecord]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT * FROM experiment_events WHERE aggregate_id = ? AND id > ? ORDER BY id",
                (aggregate_id, after),
            )
        return [
            EventRecord(
                id=row["id"],
                aggregate_type=row["aggregate_type"],
                aggregate_id=row["aggregate_id"],
                ts=row["ts"],
                event_type=row["event_type"],
                severity=row["severity"],
                payload=json.loads(row["payload_json"]),
            )
            for row in rows
        ]

    def _mirror(
        self,
        event_id: int,
        aggregate_type: str,
        aggregate_id: str,
        event_type: str,
        severity: str,
        ts: str,
        payload: dict,
    ) -> None:
        path = self.settings.experiments_dir / aggregate_id / "harnesslab-events.jsonl"
        ensure_parent(path)
        line = json.dumps(
            {
                "id": event_id,
                "aggregate_type": aggregate_type,
                "aggregate_id": aggregate_id,
                "ts": ts,
                "event_type": event_type,
                "severity": severity,
                "payload": payload,
            },
            sort_keys=True,
        )
        previous = path.read_text(encoding="utf-8") if path.exists() else ""
        atomic_write_text(Path(path), f"{previous}{line}\n")
