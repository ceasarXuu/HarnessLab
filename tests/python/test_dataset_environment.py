from pathlib import Path

from ornnlab.services.dataset_environment import parse_task_summary


def test_task_environment_summary_uses_harbor_task_config(tmp_path: Path):
    task = tmp_path / "task-one"
    environment = task / "environment"
    tests = task / "tests"
    environment.mkdir(parents=True)
    tests.mkdir()
    (task / "instruction.md").write_text("Run the task.", encoding="utf-8")
    (tests / "test.sh").write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    (environment / "Dockerfile").write_text("FROM python:3.13-slim\n", encoding="utf-8")
    (environment / "docker-compose.yaml").write_text("services: {}\n", encoding="utf-8")
    (task / "task.toml").write_text(
        """
schema_version = "1.3"

[environment]
docker_image = "example/task:1.0"
os = "linux"
build_timeout_sec = 900
network_mode = "allowlist"
allowed_hosts = ["example.com"]
cpus = 4
memory_mb = 8192
storage_mb = 20480
gpus = 1
gpu_types = ["A100", "H100"]
workdir = "/workspace"
""".strip(),
        encoding="utf-8",
    )

    summary = parse_task_summary(task, "team/eval@1.0")

    assert summary == {
        "datasetRef": "team/eval@1.0",
        "description": "",
        "environment": {
            "allowedHosts": ["example.com"],
            "buildTimeoutSeconds": 900.0,
            "definitions": ["docker-image", "dockerfile", "docker-compose"],
            "dockerImage": "example/task:1.0",
            "networkMode": "allowlist",
            "os": "linux",
            "resources": {
                "cpus": 4,
                "gpuTypes": ["A100", "H100"],
                "gpus": 1,
                "memoryMb": 8192,
                "storageMb": 20480,
                "tpu": None,
            },
            "workdir": "/workspace",
        },
        "name": "task-one",
    }
