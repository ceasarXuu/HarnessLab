from __future__ import annotations

from ornnlab.models.harbor import HarborCapabilitySnapshot, HarborJobConfigView


def harbor_running_event_payload(
    config: HarborJobConfigView,
    capability: HarborCapabilitySnapshot,
    artifacts: dict[str, str],
) -> dict:
    return {
        "job": {
            "name": config.job_name,
            "dataset": config.dataset.get("name"),
            "n_tasks": config.n_tasks,
            "n_attempts": config.n_attempts,
            "n_concurrent": config.n_concurrent,
            "jobs_dir": config.jobs_dir,
        },
        "agent": {
            "name": config.agent.get("name"),
            "model_name": config.agent.get("model_name"),
        },
        "environment": {"type": config.environment.get("type")},
        "capability": capability.model_dump(),
        "artifacts": artifacts,
    }
