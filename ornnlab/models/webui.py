from __future__ import annotations

from typing import Annotated, Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

from ornnlab.services.command_line import split_command


class WebUiModel(BaseModel):
    model_config = ConfigDict(extra="forbid", populate_by_name=True)


class DockerStartCommandInput(WebUiModel):
    command: str = Field(max_length=500)

    @field_validator("command")
    @classmethod
    def validate_command(cls, value: str) -> str:
        command = value.strip()
        if not command:
            return ""
        if "\n" in command or "\r" in command:
            raise ValueError("command must be a single executable and its arguments")
        parts = split_command(command)
        if any(part in {"&&", "||", "|", ";", ">", ">>", "<", "<<"} for part in parts):
            raise ValueError("shell operators are not supported")
        return command


class KeyValueInput(WebUiModel):
    key: str = Field(min_length=1)
    # A null value asks Harbor to inherit the variable from OrnnLab's process.
    value: str | None = None


class McpServerInput(WebUiModel):
    args: list[str] = Field(default_factory=list)
    command: str | None = None
    name: str = Field(min_length=1)
    transport: Literal["stdio", "sse", "streamable-http"] = "stdio"
    url: str | None = None

    @model_validator(mode="after")
    def validate_transport_configuration(self) -> McpServerInput:
        if self.transport == "stdio" and not self.command:
            raise ValueError("stdio MCP server requires command")
        if self.transport != "stdio" and not self.url:
            raise ValueError("remote MCP server requires url")
        return self


class AgentInput(WebUiModel):
    agent_name: str = Field(alias="agentName", min_length=1)
    authentication_mode: str | None = Field(alias="authenticationMode", default=None)
    env: list[KeyValueInput] = Field(default_factory=list)
    harness: str = Field(min_length=1)
    id: str = Field(min_length=1)
    import_path: str | None = Field(alias="importPath", default=None)
    kwargs: str = ""
    mcp_servers: list[McpServerInput] = Field(alias="mcpServers", default_factory=list)
    models: list[str] = Field(default_factory=list)
    setup_timeout_seconds: int | None = Field(alias="setupTimeoutSeconds", default=None, ge=1)
    timeout_seconds: int | None = Field(alias="timeoutSeconds", default=None, ge=1)
    skill_sources: list[str] = Field(alias="skillSources", default_factory=list)
    max_timeout_seconds: int | None = Field(alias="maxTimeoutSeconds", default=None, ge=1)


class EnvironmentInput(WebUiModel):
    allowed_hosts: list[str] = Field(alias="allowedHosts", default_factory=list)
    cpu_policy: str = Field(alias="cpuPolicy")
    delete_after_run: bool = Field(alias="deleteAfterRun")
    docker_compose_paths: list[str] = Field(alias="dockerComposePaths", default_factory=list)
    env: list[KeyValueInput] = Field(default_factory=list)
    environment_type: str = Field(alias="environmentType")
    force_build: bool = Field(alias="forceBuild")
    id: str = Field(min_length=1)
    import_path: str | None = Field(alias="importPath", default=None)
    kwargs: str
    memory_policy: str = Field(alias="memoryPolicy")
    mounts: str
    name: str = Field(min_length=1)
    override_cpus: str = Field(alias="overrideCpus")
    override_gpus: str = Field(alias="overrideGpus")
    override_memory_mb: str = Field(alias="overrideMemoryMb")
    override_storage_mb: str = Field(alias="overrideStorageMb")
    override_tpu: str = Field(alias="overrideTpu")
    profile_type: Literal["built-in", "custom"] = Field(alias="profileType")


class JobConfigInput(WebUiModel):
    agent_setup_timeout_multiplier: float = Field(
        alias="agentSetupTimeoutMultiplier", default=1.0, gt=0
    )
    agent_name: str = Field(alias="agentName", min_length=1)
    agent_timeout_multiplier: float = Field(alias="agentTimeoutMultiplier", default=1.0, gt=0)
    attempts: int = Field(ge=1)
    concurrency: int = Field(ge=1)
    dataset_ref: str = Field(alias="datasetRef", min_length=1)
    debug: bool = False
    environment_preset_id: str = Field(alias="environmentPresetId", min_length=1)
    environment_build_timeout_multiplier: float = Field(
        alias="environmentBuildTimeoutMultiplier", default=1.0, gt=0
    )
    extra_instruction_paths: list[str] = Field(alias="extraInstructionPaths", default_factory=list)
    include_in_leaderboard: bool = Field(alias="includeInLeaderboard")
    job_name: str = Field(alias="jobName", min_length=1)
    jobs_dir: str = Field(alias="jobsDir", min_length=1)
    max_retries: int = Field(alias="maxRetries", ge=0)
    metric: Literal["sum", "min", "max", "mean", "uv-script"] = "mean"
    model_name: str = Field(alias="modelName", min_length=1)
    notes: str = ""
    retry_exclude: str = Field(alias="retryExclude")
    retry_include: str = Field(alias="retryInclude")
    retry_max_wait_seconds: float = Field(alias="retryMaxWaitSeconds", ge=0)
    retry_min_wait_seconds: float = Field(alias="retryMinWaitSeconds", ge=0)
    retry_wait_multiplier: float = Field(alias="retryWaitMultiplier", ge=0)
    selected_task_names: list[str] | None = Field(alias="selectedTaskNames", default=None)
    timeout_multiplier: float = Field(alias="timeoutMultiplier", gt=0)
    verifier_timeout_multiplier: float = Field(
        alias="verifierTimeoutMultiplier", default=1.0, gt=0
    )
    verifier_mode: Literal["dataset-default", "skip"] = Field(alias="verifierMode")


class CreateJobInput(WebUiModel):
    config: JobConfigInput
    run_immediately: bool = Field(alias="runImmediately")


class DatasetImportInput(WebUiModel):
    name: str = Field(min_length=1)
    path: str = Field(min_length=1)
    task_count: int = Field(alias="taskCount", ge=0)
    version: str = Field(min_length=1)


class DatasetParentPathInput(WebUiModel):
    parent_path: str = Field(alias="parentPath", min_length=1)


class DatasetPathInput(WebUiModel):
    path: str = Field(min_length=1)


class LeaderboardUpdateInput(WebUiModel):
    include_in_leaderboard: bool = Field(alias="includeInLeaderboard")


PaginationLimit = Annotated[int, Field(ge=1, le=100)]
