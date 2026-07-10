from __future__ import annotations

import json
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
            rows = sqlite.rows(
                conn,
                "SELECT agents.profile_path, webui_agent_configs.config_json FROM agents "
                "LEFT JOIN webui_agent_configs ON webui_agent_configs.agent_id = agents.id "
                "WHERE agents.id = ?",
                (agent_id,),
            )
        if not rows:
            raise KeyError(agent_id)
        if rows[0]["config_json"]:
            from ornnlab.services.webui_profile_service import WebUiProfileService

            config = json.loads(rows[0]["config_json"])
            return WebUiProfileService(self.settings).agent_harbor_config(config)
        profile = AgentProfile.model_validate_json(Path(rows[0]["profile_path"]).read_text())
        return self.compiler.compile(profile)["agent_config"]
