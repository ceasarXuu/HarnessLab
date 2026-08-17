from __future__ import annotations

from ornnlab.services.webui_job_dto import (
    dataset_ref,
    job_config,
    job_dto,
    join_ref,
    version_filter,
)
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def leaderboard(
    settings: Settings,
    dataset_ref_value: str,
    query: str | None = None,
    metric: str | None = None,
) -> list[dict]:
    benchmark, version = dataset_ref(dataset_ref_value)
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn,
            "SELECT runs.*, webui_job_configs.config_json FROM runs "
            "LEFT JOIN webui_job_configs ON webui_job_configs.run_id = runs.id "
            "WHERE runs.status = 'completed' AND runs.leaderboard_eligible = 1 "
            f"AND runs.benchmark_name = ? AND {version_filter(version)} "
            "ORDER BY runs.finished_at DESC",
            (benchmark,) if version is None else (benchmark, version),
        )
    entries = []
    for row in rows:
        job = job_dto(row)
        config = job_config(row)
        if (
            metric
            and config.get("harbor_overrides", {}).get("metrics", [{}])[0].get("type") != metric
        ):
            continue
        entry = {
            "agentName": job["agentName"],
            "comparabilityKey": row.get("comparability_key") or "",
            "costUsd": job["costUsd"],
            "datasetRef": job["datasetRef"],
            "harness": job["harness"],
            "jobId": job["id"],
            "metric": config.get("harbor_overrides", {})
            .get("metrics", [{}])[0]
            .get("type", "mean"),
            "model": job["model"],
            "rank": 0,
            "reportPath": row.get("report_path"),
            "runtimeSeconds": job["runtimeSeconds"],
            "score": job["score"],
            "submittedAt": row.get("finished_at") or row["created_at"],
            "tokenUsageM": job["tokenUsageM"],
            "trial": job["trial"],
        }
        if not query or query.lower() in " ".join(str(value) for value in entry.values()).lower():
            entries.append(entry)
    entries.sort(
        key=lambda entry: (
            entry["score"] is None,
            -entry["score"]["value"] if entry["score"] else 0,
            entry["submittedAt"],
        )
    )
    for rank, entry in enumerate(entries, start=1):
        entry["rank"] = rank
    return entries


def leaderboard_datasets(settings: Settings) -> list[dict]:
    with sqlite.connect(settings) as conn:
        rows = sqlite.rows(
            conn,
            "SELECT DISTINCT benchmark_name, benchmark_version FROM runs WHERE status != 'deleted'",
        )
    return [
        {
            "name": row["benchmark_name"],
            "version": row["benchmark_version"] or "latest",
            "ref": join_ref(row["benchmark_name"], row["benchmark_version"]),
        }
        for row in rows
    ]
