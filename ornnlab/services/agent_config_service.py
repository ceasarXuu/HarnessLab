import json
import logging

from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


class AgentConfigService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def config(self, agent_id: str, model_name: str | None = None) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM agents WHERE id = ? AND status = 'active'",
                (agent_id,),
            )
        if not rows:
            raise KeyError(agent_id)
        config = json.loads(rows[0]["config_json"])
        logger.debug(
            "Compiling canonical Agent template",
            extra={"agent_id": agent_id, "harness": config.get("harness")},
        )
        return WebUiProfileService(self.settings).agent_harbor_config(config, model_name)
