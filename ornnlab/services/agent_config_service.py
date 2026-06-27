from __future__ import annotations

from pathlib import Path

from ornnlab.models.agent import AgentProfile
from ornnlab.services.profile_compiler import ProfileCompiler
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class AgentConfigService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.compiler = ProfileCompiler(settings)

    def config(self, agent_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT profile_path FROM agents WHERE id = ?", (agent_id,))
        if not rows:
            raise KeyError(agent_id)
        profile = AgentProfile.model_validate_json(Path(rows[0]["profile_path"]).read_text())
        return self.compiler.compile(profile)["agent_config"]
