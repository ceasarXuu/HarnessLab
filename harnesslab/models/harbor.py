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
    environment: dict[str, Any] = Field(
        default_factory=lambda: {"type": "docker", "delete": True}
    )
