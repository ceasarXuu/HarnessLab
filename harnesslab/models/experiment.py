from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

ExperimentKind = Literal["single", "comparison", "batch"]
RunStatus = Literal["draft", "queued", "running", "completed", "failed", "cancelled", "interrupted"]


class ExperimentCreate(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str
    agent_ids: list[str]
    benchmark_names: list[str]
    benchmark_version: str | None = None
    split: str | None = None
    n_tasks: int | None = None
    n_attempts: int = 1
    n_concurrent: int = 1
    mode: str = "manual"


class ExperimentView(BaseModel):
    id: str
    name: str
    kind: ExperimentKind
    status: str
    requested_run_count: int
    mode: str
    created_at: str
    updated_at: str


class RunView(BaseModel):
    id: str
    experiment_id: str
    status: RunStatus
    run_order: int
    agent_id: str
    benchmark_name: str
    benchmark_version: str | None = None
    split: str | None = None
    n_tasks: int | None = None
    n_attempts: int
    n_concurrent: int
    job_dir: str | None = None
    report_path: str | None = None
    failure_class: str | None = None
    failure_code: str | None = None


class ExperimentCreated(BaseModel):
    experiment: ExperimentView
    runs: list[RunView] = Field(default_factory=list)
