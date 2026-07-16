from __future__ import annotations

from typing import Any, Literal

CapabilityField = Literal[
    "modelName",
    "env",
    "skills",
    "mcpServers",
    "timeouts",
    "harnessParameters",
    "customKwargs",
]


def agent_capabilities(harness: str, *, custom_profile: bool = False) -> dict[str, Any]:
    fields: set[CapabilityField] = {"env", "timeouts"}
    if custom_profile:
        fields.add("customKwargs")
    if harness != "nop" and harness != "oracle":
        fields.add("modelName")
    if harness in _SKILLS_HARNESSES or harness == "custom-harness":
        fields.add("skills")
    if harness in _MCP_HARNESSES or harness == "custom-harness":
        fields.add("mcpServers")
    parameters = _parameters_for_harness(harness)
    if harness == "custom-harness":
        parameters = []
        fields.update({"modelName", "skills", "mcpServers", "harnessParameters"})
    elif parameters:
        fields.add("harnessParameters")
    return {
        "authenticationModes": _authentication_modes(harness),
        "environmentVariables": _harbor_environment_variables(harness),
        "parameters": parameters,
        "supportedFields": sorted(fields),
    }


def custom_agent_capabilities(harness: str) -> dict[str, Any]:
    return agent_capabilities(harness, custom_profile=True)


def default_authentication_mode(harness: str) -> str | None:
    modes = _authentication_modes(harness)
    return modes[0]["value"] if modes else None


def _param(
    key: str,
    label: str,
    *,
    kind: str = "text",
    choices: list[str] | None = None,
    default: str | int | bool | None = None,
    source: str = "kwarg",
) -> dict[str, Any]:
    payload: dict[str, Any] = {"key": key, "kind": kind, "label": label, "source": source}
    if choices:
        payload["choices"] = choices
    if default is not None:
        payload["defaultValue"] = default
    return payload


def _parameters_for_harness(harness: str) -> list[dict[str, Any]]:
    environment_kwargs = _harbor_environment_kwargs(harness)
    configured = {
        item["key"]: item
        for item in _PARAMETERS_BY_HARNESS.get(harness, [])
        if item["key"] not in environment_kwargs
    }
    descriptors = _harbor_descriptor_parameters(harness)
    merged: dict[str, dict[str, Any]] = {}
    for key in configured.keys() | descriptors.keys():
        manual = configured.get(key, {})
        descriptor = descriptors.get(key, {})
        merged[key] = {
            **manual,
            **descriptor,
            "label": manual.get("label", descriptor.get("label", key.replace("_", " ").title())),
        }
    return [merged[key] for key in sorted(merged)]


def _harbor_descriptor_parameters(harness: str) -> dict[str, dict[str, Any]]:
    agent_class = _harbor_agent_class(harness)
    if agent_class is None:
        return {}
    parameters: dict[str, dict[str, Any]] = {}
    for descriptor in getattr(agent_class, "CLI_FLAGS", []):
        kind = {"bool": "boolean", "int": "number", "float": "number"}.get(
            descriptor.type, "text"
        )
        parameters[descriptor.kwarg] = _param(
            descriptor.kwarg,
            descriptor.kwarg.replace("_", " ").title(),
            kind=kind,
            choices=list(descriptor.choices or []),
            default=descriptor.default,
            # BaseInstalledAgent resolves both CLI_FLAGS and ENV_VARS from constructor kwargs.
            source="kwarg",
        )
    return parameters


def _harbor_environment_variables(harness: str) -> list[str]:
    agent_class = _harbor_agent_class(harness)
    if agent_class is None:
        return []
    cli_kwargs = {
        descriptor.kwarg for descriptor in getattr(agent_class, "CLI_FLAGS", [])
    }
    described = {
        descriptor.env
        for descriptor in getattr(agent_class, "ENV_VARS", [])
        if descriptor.kwarg not in cli_kwargs
    }
    return sorted(described | set(_ENVIRONMENT_VARIABLES_BY_HARNESS.get(harness, [])))


def _harbor_environment_kwargs(harness: str) -> set[str]:
    agent_class = _harbor_agent_class(harness)
    if agent_class is None:
        return set()
    return {descriptor.kwarg for descriptor in getattr(agent_class, "ENV_VARS", [])}


def _harbor_agent_class(harness: str) -> Any | None:
    from harbor.agents.factory import AgentFactory

    return next((item for item in AgentFactory._AGENTS if item.name() == harness), None)


def _authentication_modes(harness: str) -> list[dict[str, Any]]:
    return [dict(item) for item in _AUTHENTICATION_MODES_BY_HARNESS.get(harness, [])]


_ENVIRONMENT_VARIABLES_BY_HARNESS: dict[str, list[str]] = {
    "claude-code": [
        "CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING",
        "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
    ],
}

_AUTHENTICATION_MODES_BY_HARNESS: dict[str, list[dict[str, Any]]] = {
    "claude-code": [
        {
            "environmentVariables": [
                "ANTHROPIC_API_KEY",
                "ANTHROPIC_AUTH_TOKEN",
                "ANTHROPIC_BASE_URL",
            ],
            "label": "Anthropic API",
            "value": "anthropic-api",
        },
        {
            "environmentVariables": ["CLAUDE_CODE_OAUTH_TOKEN"],
            "label": "Claude OAuth",
            "value": "oauth",
        },
        {
            "environmentVariables": [
                "AWS_BEARER_TOKEN_BEDROCK",
                "AWS_ACCESS_KEY_ID",
                "AWS_SECRET_ACCESS_KEY",
                "AWS_SESSION_TOKEN",
                "AWS_PROFILE",
                "AWS_REGION",
                "ANTHROPIC_SMALL_FAST_MODEL_AWS_REGION",
                "DISABLE_PROMPT_CACHING",
            ],
            "label": "Amazon Bedrock",
            "value": "bedrock",
        },
    ],
}


_MCP_HARNESSES = {
    "acp",
    "antigravity-cli",
    "claude-code",
    "cline-cli",
    "codex",
    "copilot-cli",
    "cursor-cli",
    "gemini-cli",
    "goose",
    "hermes",
    "kimi-cli",
    "mini-swe-agent",
    "openclaw",
    "opencode",
    "openhands",
    "openhands-sdk",
    "qwen-coder",
    "terminus-2",
}

_SKILLS_HARNESSES = {
    "antigravity-cli",
    "claude-code",
    "cline-cli",
    "codex",
    "copilot-cli",
    "cursor-cli",
    "gemini-cli",
    "goose",
    "hermes",
    "kimi-cli",
    "openclaw",
    "opencode",
    "openhands-sdk",
    "pi",
    "qwen-coder",
    "terminus-2",
}

_REASONING = ["low", "medium", "high", "xhigh"]

_PARAMETERS_BY_HARNESS: dict[str, list[dict[str, Any]]] = {
    "acp": [
        _param("permission_mode", "Permission mode", choices=["allow", "deny"]),
        _param("auth_policy", "Auth policy", choices=["auto", "explicit", "disabled"]),
        _param("authenticate_method_id", "Authenticate method ID"),
        _param("registry_spec", "Registry spec"),
        _param("registry_ref", "Registry ref", default="main"),
        _param("distribution_preference", "Distribution preference"),
        _param("target_platform", "Target platform"),
    ],
    "aider": [
        _param("reasoning_effort", "Reasoning effort"),
        _param("thinking_tokens", "Thinking tokens", kind="number"),
        _param("cache_prompts", "Cache prompts", kind="boolean"),
        _param("auto_lint", "Auto lint", kind="boolean"),
        _param("auto_test", "Auto test", kind="boolean"),
        _param("test_cmd", "Test command"),
        _param("stream", "Stream output", kind="boolean"),
        _param("map_tokens", "Map tokens", kind="number"),
    ],
    "antigravity-cli": [
        _param("reasoning_effort", "Reasoning effort"),
        _param("sandbox", "Sandbox", kind="boolean"),
    ],
    "claude-code": [
        _param("memory_dir", "Memory directory"),
        _param("max_turns", "Max turns", kind="number"),
        _param("reasoning_effort", "Reasoning effort", choices=[*_REASONING, "max"]),
        _param("thinking", "Thinking", choices=["enabled", "adaptive", "disabled"]),
        _param("thinking_display", "Thinking display", choices=["summarized", "omitted"]),
        _param("max_thinking_tokens", "Max thinking tokens", kind="number"),
        _param("max_budget_usd", "Max budget USD"),
        _param("fallback_model", "Fallback model"),
        _param("append_system_prompt", "Append system prompt"),
        _param("allowed_tools", "Allowed tools"),
        _param("disallowed_tools", "Disallowed tools"),
    ],
    "cline-cli": [
        _param("thinking", "Thinking tokens", kind="number"),
        _param("reasoning_effort", "Reasoning effort", choices=["none", *_REASONING]),
        _param("max_consecutive_mistakes", "Max consecutive mistakes", kind="number"),
        _param("double_check_completion", "Double-check completion", kind="boolean"),
        _param("setup_retries", "Setup retries", kind="number"),
        _param("setup_retry_delay_sec", "Setup retry delay seconds", kind="number"),
        _param("setup_command_timeout_sec", "Setup command timeout seconds", kind="number"),
    ],
    "codex": [
        _param("reasoning_effort", "Reasoning effort", default="high"),
        _param(
            "reasoning_summary",
            "Reasoning summary",
            choices=["auto", "concise", "detailed", "none"],
        ),
    ],
    "copilot-cli": [_param("reasoning_effort", "Reasoning effort", choices=_REASONING)],
    "cursor-cli": [_param("mode", "Mode", choices=["plan", "ask"])],
    "gemini-cli": [
        _param("reasoning_effort", "Reasoning effort"),
        _param("sandbox", "Sandbox", kind="boolean"),
    ],
    "goose": [_param("max_turns", "Max turns", kind="number")],
    "hermes": [_param("toolsets", "Toolsets")],
    "langgraph": [
        _param("project_path", "Project path"),
        _param("graph", "Graph"),
        _param("config", "Config file", default="langgraph.json"),
        _param("model_kwargs", "Model kwargs"),
        _param("configurable", "Configurable"),
        _param("dependency_overrides", "Dependency overrides"),
    ],
    "mini-swe-agent": [
        _param("reasoning_effort", "Reasoning effort"),
        _param("config_file", "Config file"),
        _param("cost_limit", "Cost limit", default="0"),
    ],
    "nemo-agent": [
        _param(
            "llm_type",
            "LLM type",
            choices=[
                "nim",
                "openai",
                "azure_openai",
                "aws_bedrock",
                "litellm",
                "huggingface_inference",
                "dynamo",
            ],
            default="nim",
        ),
        _param("config_file", "Config file"),
        _param("workflow_package", "Workflow package"),
        _param("nat_repo", "NAT repo"),
        _param("verbose", "Log level", choices=["debug", "info", "warning", "error"]),
    ],
    "openclaw": [
        _param("openclaw_agent_id", "OpenClaw agent ID", default="main"),
        _param("thinking", "Thinking", default="high"),
        _param("timeout", "Timeout seconds", kind="number"),
    ],
    "opencode": [_param("variant", "Variant")],
    "openhands": [
        _param("disable_tool_calls", "Disable tool calls", kind="boolean"),
        _param("reasoning_effort", "Reasoning effort", default="high"),
        _param("temperature", "Temperature"),
        _param("max_iterations", "Max iterations", kind="number"),
        _param("caching_prompt", "Caching prompt", kind="boolean"),
        _param("top_p", "Top P"),
        _param("num_retries", "Retry count", kind="number"),
        _param("max_budget_per_task", "Max budget per task"),
        _param("drop_params", "Drop params", kind="boolean"),
        _param("disable_vision", "Disable vision", kind="boolean"),
    ],
    "openhands-sdk": [
        _param("reasoning_effort", "Reasoning effort", default="high"),
        _param("load_skills", "Load skills", kind="boolean", default=True),
        _param("skill_paths", "Skill paths"),
        _param("collect_token_ids", "Collect token IDs", kind="boolean"),
        _param("max_iterations", "Max iterations", kind="number"),
        _param("temperature", "Temperature"),
    ],
    "pi": [
        _param(
            "thinking",
            "Thinking",
            choices=["off", "minimal", "low", "medium", "high", "xhigh"],
        )
    ],
    "qwen-coder": [
        _param("api_key", "API key environment value"),
        _param("base_url", "Base URL"),
    ],
    "rovodev-cli": [_param("max_thinking_tokens", "Max thinking tokens", kind="number")],
    "swe-agent": [
        _param("per_instance_cost_limit", "Per-instance cost limit"),
        _param("total_cost_limit", "Total cost limit"),
        _param("max_input_tokens", "Max input tokens"),
        _param("temperature", "Temperature"),
        _param("top_p", "Top P"),
    ],
    "terminus-2": [
        _param("max_turns", "Max turns", kind="number"),
        _param("parser_name", "Parser", choices=["json", "xml"], default="json"),
        _param("api_base", "API base"),
        _param("temperature", "Temperature"),
        _param(
            "reasoning_effort",
            "Reasoning effort",
            choices=["none", "minimal", *_REASONING, "max", "default"],
        ),
        _param("max_thinking_tokens", "Max thinking tokens", kind="number"),
        _param("tmux_pane_width", "Tmux pane width", kind="number", default=160),
        _param("tmux_pane_height", "Tmux pane height", kind="number", default=40),
        _param("use_responses_api", "Use Responses API", kind="boolean"),
    ],
    "trae-agent": [
        _param("max_steps", "Max steps", kind="number", default=200),
        _param("temperature", "Temperature", default="0.7"),
        _param("max_tokens", "Max tokens", kind="number", default=16384),
        _param("top_p", "Top P", default="0.95"),
        _param("top_k", "Top K", kind="number", default=20),
    ],
}
