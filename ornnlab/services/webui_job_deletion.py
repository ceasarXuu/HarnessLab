from __future__ import annotations

import json
import logging
import shutil
from pathlib import Path

from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)
TERMINAL_STATUSES = {"completed", "failed", "cancelled", "interrupted"}


class WebUiJobDeletionService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def delete(self, job_id: str) -> dict[str, object]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT runs.*, webui_job_configs.config_json FROM runs "
                "LEFT JOIN webui_job_configs ON webui_job_configs.run_id = runs.id "
                "WHERE runs.id = ?",
                (job_id,),
            )
            if not rows:
                raise KeyError(job_id)
            run = rows[0]
            if run["status"] not in TERMINAL_STATUSES:
                raise RuntimeError("running or queued jobs must be cancelled before deletion")

            experiment_id = str(run["experiment_id"])
            remaining_runs = conn.execute(
                "SELECT COUNT(*) FROM runs WHERE experiment_id = ? AND id != ?",
                (experiment_id, job_id),
            ).fetchone()[0]
            delete_experiment = remaining_runs == 0
            roots = self._artifact_roots(run, delete_experiment)
            self._validate_stored_artifacts(conn, run, roots, experiment_id)

            logger.info(
                "Deleting Job records and artifacts job_id=%s experiment_id=%s roots=%s",
                job_id,
                experiment_id,
                [str(path) for path in roots],
            )
            deleted = {
                "operations": conn.execute(
                    "DELETE FROM webui_operations WHERE resource_type = 'job' "
                    "AND resource_id = ?",
                    (job_id,),
                ).rowcount,
                "events": conn.execute(
                    "DELETE FROM experiment_events WHERE aggregate_id = ?", (job_id,)
                ).rowcount,
                "jobConfigs": conn.execute(
                    "DELETE FROM webui_job_configs WHERE run_id = ?", (job_id,)
                ).rowcount,
                "queueItems": conn.execute(
                    "DELETE FROM queue_items WHERE run_id = ?", (job_id,)
                ).rowcount,
            }
            deleted["runs"] = conn.execute(
                "DELETE FROM runs WHERE id = ?", (job_id,)
            ).rowcount
            if delete_experiment:
                deleted["events"] += conn.execute(
                    "DELETE FROM experiment_events WHERE aggregate_id = ?", (experiment_id,)
                ).rowcount
                deleted["experiments"] = conn.execute(
                    "DELETE FROM experiments WHERE id = ?", (experiment_id,)
                ).rowcount
            else:
                deleted["experiments"] = 0

            for root in roots:
                _remove_owned_tree(root)

        logger.info("Job deletion completed job_id=%s deleted=%s", job_id, deleted)
        return {"deletedJobId": job_id}

    def _artifact_roots(self, run: dict, delete_experiment: bool) -> list[Path]:
        run_root = self.settings.experiments_dir / str(run["id"])
        roots = [run_root] if run_root.exists() else []
        if delete_experiment:
            experiment_root = self.settings.experiments_dir / str(run["experiment_id"])
            if experiment_root.exists() and experiment_root != run_root:
                roots.append(experiment_root)

        jobs_dir_value = run.get("job_dir")
        if not jobs_dir_value:
            return roots
        jobs_dir = Path(str(jobs_dir_value)).expanduser()
        if _is_within(jobs_dir, run_root):
            return _deduplicated_roots(roots)

        config = json.loads(run["config_json"]) if run.get("config_json") else {}
        job_name = run.get("harbor_job_name") or config.get("job_name")
        if not isinstance(job_name, str) or not _safe_child_name(job_name):
            raise ValueError("Job artifact ownership cannot be proven")
        native_root = jobs_dir / job_name
        if native_root.exists():
            if native_root.is_symlink() or native_root.parent.resolve() != jobs_dir.resolve():
                raise ValueError("Job artifact ownership cannot be proven")
            roots.append(native_root)
        return _deduplicated_roots(roots)

    def _validate_stored_artifacts(
        self,
        conn,
        run: dict,
        roots: list[Path],
        experiment_id: str,
    ) -> None:
        pointers = [run.get("result_path"), run.get("report_path")]
        mirrors = sqlite.rows(
            conn,
            "SELECT mirror_file FROM experiment_events WHERE aggregate_id IN (?, ?) "
            "AND mirror_file IS NOT NULL",
            (run["id"], experiment_id),
        )
        pointers.extend(row["mirror_file"] for row in mirrors)
        for value in pointers:
            if not value:
                continue
            path = Path(str(value)).expanduser()
            if path.exists() and not any(_is_within(path, root) for root in roots):
                raise ValueError(f"Job artifact ownership cannot be proven: {path}")


def _safe_child_name(value: str) -> bool:
    return value not in {"", ".", ".."} and Path(value).name == value


def _is_within(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except ValueError:
        return False


def _deduplicated_roots(paths: list[Path]) -> list[Path]:
    result: list[Path] = []
    for path in sorted(paths, key=lambda item: len(item.parts)):
        if path.is_symlink():
            raise ValueError("Job artifact ownership cannot be proven")
        if not any(_is_within(path, parent) for parent in result):
            result.append(path)
    return result


def _remove_owned_tree(path: Path) -> None:
    if not path.exists():
        return
    if path.is_symlink() or not path.is_dir():
        raise ValueError(f"Job artifact root is not a directory: {path}")
    shutil.rmtree(path)
