from __future__ import annotations

from pydantic import BaseModel


class EventRecord(BaseModel):
    id: int
    aggregate_type: str
    aggregate_id: str
    ts: str
    event_type: str
    severity: str
    payload: dict
