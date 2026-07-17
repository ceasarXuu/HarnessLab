from __future__ import annotations

import logging
from pathlib import Path

logger = logging.getLogger(__name__)


def parse_task_summary(task_dir: Path, dataset_ref: str) -> dict:
    """Parse the user-facing Task summary from Harbor's canonical task model."""
    from harbor.models.task.config import TaskConfig
    from harbor.models.task.paths import TaskPaths

    paths = TaskPaths(task_dir)
    task_config = TaskConfig.model_validate_toml(paths.config_path.read_text())
    config = task_config.environment
    definitions = []
    if config.docker_image:
        definitions.append("docker-image")
    if paths.environment_dir.joinpath("Dockerfile").is_file():
        definitions.append("dockerfile")
    if paths.environment_dir.joinpath("docker-compose.yaml").is_file():
        definitions.append("docker-compose")

    tpu = None
    if config.tpu is not None:
        tpu = {"topology": config.tpu.topology, "type": config.tpu.type}

    description = task_config.task.description if task_config.task is not None else ""
    name = task_config.task.name if task_config.task is not None else task_dir.name
    return {
        "datasetRef": dataset_ref,
        "description": description,
        "environment": {
            "allowedHosts": list(config.allowed_hosts or []),
            "buildTimeoutSeconds": config.build_timeout_sec,
            "definitions": definitions,
            "dockerImage": config.docker_image,
            "imagePlatforms": None,
            "networkMode": config.network_mode.value,
            "os": config.os.value,
            "resources": {
                "cpus": config.cpus,
                "gpuTypes": list(config.gpu_types or []),
                "gpus": config.gpus,
                "memoryMb": config.memory_mb,
                "storageMb": config.storage_mb,
                "tpu": tpu,
            },
            "workdir": config.workdir,
        },
        "name": name,
    }


def parse_local_tasks(path: Path, dataset_ref: str) -> list[dict]:
    if not path.is_dir():
        return []

    from harbor.models.task.task import Task

    tasks = []
    for child in sorted(path.iterdir()):
        if not child.is_dir() or not Task.is_valid_dir(child, disable_verification=True):
            continue
        try:
            tasks.append(parse_task_summary(child, dataset_ref))
        except Exception:
            logger.warning(
                "Failed to parse Dataset Task environment ref=%s task=%s",
                dataset_ref,
                child.name,
                exc_info=True,
            )
            tasks.append(
                {
                    "datasetRef": dataset_ref,
                    "description": "",
                    "environment": None,
                    "name": child.name,
                }
            )
    return tasks
