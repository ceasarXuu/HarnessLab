from __future__ import annotations

from pydantic import BaseModel, Field


class ReportSummary(BaseModel):
    run_id: str
    status: str
    score: float | None = None
    failure_class: str | None = None
    failure_code: str | None = None
    artifact_links: list[str] = Field(default_factory=list)
