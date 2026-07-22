from __future__ import annotations

import json
import os
import shutil
import subprocess
from uuid import uuid4

import pytest

from ornnlab.services.docker_orphan_service import DockerOrphanService
from ornnlab.services.owned_docker_environment import _ownership_compose_payload

pytestmark = pytest.mark.docker


def _real_docker_enabled() -> bool:
    return os.environ.get("ORNNLAB_REAL_DOCKER_OWNERSHIP") == "1" and bool(
        shutil.which("docker")
    )


@pytest.mark.skipif(
    not _real_docker_enabled(),
    reason="set ORNNLAB_REAL_DOCKER_OWNERSHIP=1 with Docker available",
)
def test_real_compose_labels_and_cleanup_cover_every_service(tmp_path):
    suffix = uuid4().hex[:10]
    project = f"ornnlab-ownership-{suffix}"
    instance_id = f"instance-{suffix}"
    run_id = f"run-{suffix}"
    base = tmp_path / "compose.json"
    ownership = tmp_path / "ownership.json"
    base.write_text(
        json.dumps(
            {
                "services": {
                    "main": {
                        "image": "ubuntu:24.04",
                        "command": ["sleep", "300"],
                    },
                    "sidecar": {
                        "image": "ubuntu:24.04",
                        "command": ["sleep", "300"],
                    },
                }
            }
        ),
        encoding="utf-8",
    )
    labels = {
        "ornnlab.managed": "true",
        "ornnlab.instance_id": instance_id,
        "ornnlab.run_id": run_id,
        "ornnlab.cleanup": "auto",
    }
    ownership.write_text(
        json.dumps(
            _ownership_compose_payload(
                ["main", "sidecar"], labels, networks=["default"]
            )
        ),
        encoding="utf-8",
    )
    compose = [
        "docker",
        "compose",
        "--project-name",
        project,
        "-f",
        str(base),
        "-f",
        str(ownership),
    ]
    try:
        subprocess.run([*compose, "up", "-d", "--wait"], check=True, timeout=60)
        before = DockerOrphanService(instance_id=instance_id).scan_ornnlab_containers()
        assert before["count"] == 2

        cleanup = DockerOrphanService(instance_id=instance_id).cleanup_run(run_id)

        assert cleanup["ok"] is True
        assert cleanup["removed_containers"] == 2
        assert cleanup["removed_networks"] == 1
        assert DockerOrphanService(instance_id=instance_id).scan_ornnlab_containers()["count"] == 0
    finally:
        subprocess.run(
            [*compose, "down", "--volumes", "--remove-orphans"],
            check=False,
            capture_output=True,
            timeout=30,
        )
