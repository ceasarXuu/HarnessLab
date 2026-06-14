from __future__ import annotations

import json
from pathlib import Path

from pydantic import ValidationError

from harnesslab.models.agent import AgentProfile
from harnesslab.services.clock import now_iso
from harnesslab.services.event_service import EventService
from harnesslab.services.profile_compiler import ProfileCompiler
from harnesslab.settings import Settings
from harnesslab.storage import sqlite
from harnesslab.storage.paths import atomic_write_text


class AgentService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.compiler = ProfileCompiler(settings)
        self.events = EventService(settings)

    def create(self, payload: dict) -> dict:
        profile = AgentProfile.model_validate(payload)
        path = self._profile_path(profile.id)
        atomic_write_text(path, json.dumps(profile.model_dump(), indent=2, sort_keys=True))
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "INSERT INTO agents("
                "id, name, kind, harbor_agent_name, model_name, status, profile_path, "
                "created_at, updated_at"
                ") VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    profile.id,
                    profile.name,
                    profile.kind,
                    profile.harbor.agent,
                    profile.harbor.model,
                    "draft",
                    str(path),
                    now,
                    now,
                ),
            )
        self.events.append("agent", profile.id, "agent.created", {"name": profile.name})
        return self.get(profile.id)

    def list(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(conn, "SELECT * FROM agents ORDER BY created_at DESC")

    def get(self, agent_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT * FROM agents WHERE id = ?", (agent_id,))
        if not rows:
            raise KeyError(agent_id)
        return rows[0]

    def compile(self, agent_id: str) -> dict:
        row = self.get(agent_id)
        profile = AgentProfile.model_validate_json(Path(row["profile_path"]).read_text())
        result = self.compiler.compile(profile)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE agents SET status = ?, harbor_import_path = ?, updated_at = ? WHERE id = ?",
                (
                    "compiled",
                    result["agent_config"].get("import_path"),
                    now_iso(),
                    agent_id,
                ),
            )
        self.events.append("agent", agent_id, "agent.compiled", {"mode": result["mode"]})
        return result

    def validate(self, payload: dict) -> dict:
        try:
            AgentProfile.model_validate(payload)
        except ValidationError as exc:
            return {"valid": False, "errors": exc.errors()}
        return {"valid": True, "errors": []}

    def _profile_path(self, agent_id: str) -> Path:
        return self.settings.agents_dir / f"{agent_id}.json"
