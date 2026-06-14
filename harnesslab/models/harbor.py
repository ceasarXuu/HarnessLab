from __future__ import annotations

from pydantic import BaseModel, ConfigDict


class HarborCapabilitySnapshot(BaseModel):
    model_config = ConfigDict(extra="forbid")

    harbor_version: str | None
    api_symbols: list[str]
    lifecycle_mode: str
    environment_backend: str


class HarborJobConfigView(BaseModel):
    model_config = ConfigDict(extra="forbid")

    agent: dict
    dataset: dict
    n_tasks: int | None
    n_attempts: int
    n_concurrent: int
    jobs_dir: str
