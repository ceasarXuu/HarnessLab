from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator


class HarborProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    agent: str | None = None
    model: str | None = None
    kwargs: dict[str, str | int | float | bool] = Field(default_factory=dict)


class AuthProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    inherit_env: list[str] = Field(default_factory=list)
    include_paths: list[str] = Field(default_factory=list)


class SkillsProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    paths: list[str] = Field(default_factory=list)


class McpProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    config_paths: list[str] = Field(default_factory=list)


class RuntimeProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    agent_timeout_sec: int = 3600
    setup_timeout_sec: int = 600
    backend: Literal["docker", "local"] = "docker"


class CommandAgentProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    install: list[str] = Field(default_factory=list)
    run: str
    working_dir: str = "workspace"
    shell: Literal["bash", "sh"] = "bash"

    @field_validator("run")
    @classmethod
    def require_instruction_placeholder(cls, value: str) -> str:
        if "{{instruction}}" not in value:
            raise ValueError("custom-command run template must contain {{instruction}}")
        if value.count("{{instruction}}") != 1:
            raise ValueError("custom-command run template must contain {{instruction}} once")
        return value


class AgentProfile(BaseModel):
    model_config = ConfigDict(extra="forbid")

    schema_version: Literal[2]
    id: str
    name: str
    kind: str
    description: str | None = None
    harbor: HarborProfile = Field(default_factory=HarborProfile)
    command_agent: CommandAgentProfile | None = None
    auth: AuthProfile = Field(default_factory=AuthProfile)
    skills: SkillsProfile = Field(default_factory=SkillsProfile)
    mcp: McpProfile = Field(default_factory=McpProfile)
    runtime: RuntimeProfile = Field(default_factory=RuntimeProfile)

    @field_validator("id")
    @classmethod
    def validate_id(cls, value: str) -> str:
        if not value or any(ch.isspace() for ch in value):
            raise ValueError("agent id must be non-empty and contain no whitespace")
        return value
