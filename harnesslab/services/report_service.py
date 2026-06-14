from __future__ import annotations

import json
from pathlib import Path

from harnesslab.models.report import ReportSummary
from harnesslab.settings import Settings
from harnesslab.storage.paths import atomic_write_text


class ReportService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def write_summary(
        self,
        run_id: str,
        status: str,
        job_dir: str,
        score: float | None,
        failure_class: str | None = None,
        failure_code: str | None = None,
    ) -> str:
        report_dir = self.settings.experiments_dir / run_id / "report"
        summary = ReportSummary(
            run_id=run_id,
            status=status,
            score=score,
            failure_class=failure_class,
            failure_code=failure_code,
            artifact_links=[f"{job_dir}/result.json", f"{job_dir}/job.log"],
        )
        summary_path = report_dir / "summary.json"
        atomic_write_text(summary_path, json.dumps(summary.model_dump(), indent=2, sort_keys=True))
        html = (
            "<!doctype html><html><head><meta charset='utf-8'><title>HarnessLab Report</title>"
            "</head><body><h1>HarnessLab Report</h1>"
            f"<p>Run: {run_id}</p><p>Status: {status}</p><p>Score: {score}</p></body></html>"
        )
        index_path = report_dir / "index.html"
        atomic_write_text(index_path, html)
        return str(index_path)

    def read_summary(self, report_path: str | None) -> dict:
        if report_path is None:
            raise KeyError("missing report path")
        summary_path = Path(report_path).with_name("summary.json")
        if not summary_path.exists():
            raise KeyError(str(summary_path))
        return json.loads(summary_path.read_text(encoding="utf-8"))
