from ornnlab.models.webui import AgentInput
from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.settings import Settings


def create_test_agent(
    settings: Settings,
    *,
    agent_id: str = "oracle",
    harness: str = "oracle",
    name: str = "Oracle",
) -> dict:
    return WebUiProfileService(settings).create_agent(
        AgentInput.model_validate(
            {
                "id": agent_id,
                "agentName": name,
                "harness": harness,
                "type": "custom",
                "env": [],
                "kwargs": "",
                "mcpServers": [],
                "models": [harness],
                "skillSources": [],
            }
        )
    )
