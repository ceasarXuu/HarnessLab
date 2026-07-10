from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field


class HarborCapabilitySnapshot(BaseModel):
    model_config = ConfigDict(extra="forbid")

    harbor_version: str | None
    api_symbols: list[str]
    lifecycle_mode: str
    environment_backend: str
    config_format: str
    supports_cancel: bool


class HarborJobConfigView(BaseModel):
    model_config = ConfigDict(extra="forbid")

    job_name: str
    agent: dict[str, Any]
    dataset: dict[str, Any]
    n_tasks: int | None
    n_attempts: int
    n_concurrent: int
    jobs_dir: str
    timeout_multiplier: float = 1.0
    agent_timeout_multiplier: float = 1.0
    verifier_timeout_multiplier: float = 1.0
    agent_setup_timeout_multiplier: float = 1.0
    environment_build_timeout_multiplier: float = 1.0
    extra_instruction_paths: list[str] = Field(default_factory=list)
    debug: bool = False
    retry: dict[str, Any] = Field(default_factory=dict)
    verifier: dict[str, Any] = Field(default_factory=dict)
    metrics: list[dict[str, Any]] = Field(default_factory=list)
    environment: dict[str, Any] = Field(default_factory=lambda: {"type": "docker", "delete": True})
