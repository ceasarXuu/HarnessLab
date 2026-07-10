from __future__ import annotations

from pathlib import Path


def resolve_harbor_result_path(jobs_dir: Path, job_name: str | None) -> Path:
    """Locate the result written by either Harbor's native or legacy layout."""
    direct_result = jobs_dir / "result.json"
    native_result = native_harbor_job_dir(jobs_dir, job_name) / "result.json"
    if direct_result.exists() or not native_result.exists():
        return direct_result
    return native_result


def resolve_harbor_log_path(jobs_dir: Path, job_name: str | None) -> Path:
    """Prefer Harbor's native Job log while retaining the subprocess mirror fallback."""
    direct_log = jobs_dir / "job.log"
    native_log = native_harbor_job_dir(jobs_dir, job_name) / "job.log"
    if native_log.exists():
        return native_log
    return direct_log


def resolve_harbor_job_path(jobs_dir: Path, job_name: str | None) -> Path:
    """Return the directory Harbor CLI expects for `harbor job resume`."""
    native_dir = native_harbor_job_dir(jobs_dir, job_name)
    return native_dir if (native_dir / "config.json").is_file() else jobs_dir


def native_harbor_job_dir(jobs_dir: Path, job_name: str | None) -> Path:
    return jobs_dir / job_name if job_name else jobs_dir
