from __future__ import annotations

import json
from uuid import uuid4

from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class TemplateService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def create(self, name: str, config: dict) -> dict:
        template_id = f"tpl-{uuid4().hex[:12]}"
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "INSERT INTO templates(id, name, config_json, created_at, updated_at) "
                "VALUES (?, ?, ?, ?, ?)",
                (template_id, name, json.dumps(config, sort_keys=True), now, now),
            )
        return self.get(template_id)

    def list(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT * FROM templates WHERE deleted_at IS NULL ORDER BY created_at DESC",
            )
        return [self._decode(row) for row in rows]

    def get(self, template_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT * FROM templates WHERE id = ? AND deleted_at IS NULL",
                (template_id,),
            )
        if not rows:
            raise KeyError(template_id)
        return self._decode(rows[0])

    def soft_delete(self, template_id: str) -> dict:
        template = self.get(template_id)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE templates SET deleted_at = ?, updated_at = ? WHERE id = ?",
                (now_iso(), now_iso(), template_id),
            )
        return template

    @staticmethod
    def _decode(row: dict) -> dict:
        decoded = dict(row)
        decoded["config"] = json.loads(decoded.pop("config_json"))
        return decoded
