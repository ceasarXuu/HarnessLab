from __future__ import annotations

import logging
import re
from pathlib import Path

logger = logging.getLogger(__name__)
_FROM_PATTERN = re.compile(
    r"^\s*FROM\s+(?:(?:--\S+)\s+)*(?P<image>\S+)(?:\s+AS\s+(?P<alias>\S+))?",
    re.IGNORECASE,
)


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
    dockerfile_path = paths.environment_dir.joinpath("Dockerfile")
    if dockerfile_path.is_file():
        definitions.append("dockerfile")
    if paths.environment_dir.joinpath("docker-compose.yaml").is_file():
        definitions.append("docker-compose")

    tpu = None
    if config.tpu is not None:
        tpu = {"topology": config.tpu.topology, "type": config.tpu.type}

    description = task_config.task.description if task_config.task is not None else ""
    name = task_config.task.name if task_config.task is not None else task_dir.name
    container_images = []
    if config.docker_image:
        container_images.append(_container_image(config.docker_image, "environment-config"))
    if dockerfile_path.is_file():
        container_images.extend(
            _container_image(reference, "dockerfile-base")
            for reference in _dockerfile_base_images(dockerfile_path)
            if reference not in {image["reference"] for image in container_images}
        )
    logger.debug(
        "Parsed Dataset Task container images ref=%s task=%s images=%s",
        dataset_ref,
        name,
        [image["reference"] for image in container_images],
    )
    return {
        "datasetRef": dataset_ref,
        "description": description,
        "environment": {
            "allowedHosts": list(config.allowed_hosts or []),
            "buildTimeoutSeconds": config.build_timeout_sec,
            "containerImages": container_images,
            "definitions": definitions,
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


def _container_image(reference: str, source: str) -> dict:
    return {"platforms": None, "reference": reference, "source": source}


def _dockerfile_base_images(path: Path) -> list[str]:
    aliases: set[str] = set()
    images: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not (match := _FROM_PATTERN.match(line)):
            continue
        reference = match.group("image")
        if (
            reference.lower() not in aliases
            and reference.lower() != "scratch"
            and not reference.startswith("$")
            and reference not in images
        ):
            images.append(reference)
        if alias := match.group("alias"):
            aliases.add(alias.lower())
    return images


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
