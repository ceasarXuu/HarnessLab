from __future__ import annotations

import asyncio
import json
import time
from pathlib import Path

from ornnlab.services.experiment_service import _resolve_job_dir
from ornnlab.services.harbor_score import pass_at_one, result_pass_at_one
from ornnlab.services.webui_job_service import _job_score, _trial_dto
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.storage import sqlite

API = "/api/webui/v1"


def test_webui_envelope_and_legacy_routes_are_not_registered(client):
    health = client.get(f"{API}/system/health")

    assert health.status_code == 200
    body = health.json()
    assert body["error"] is None
    assert body["meta"]["requestId"]
    assert body["data"]["total"] == 7
    assert client.get("/api/experiments").status_code == 404
    assert client.get("/api/agents").status_code == 404
    assert client.get("/api/system/status").status_code == 404
    assert client.post(f"{API}/jobs/example/retry").status_code == 404


def test_webui_agent_and_environment_crud(client):
    created_agent = client.post(f"{API}/agents", json=_agent_payload()).json()["data"]["operation"]
    assert created_agent["status"] == "completed"

    agent = client.get(f"{API}/agents/oracle-profile").json()["data"]
    assert agent["agentName"] == "Oracle profile"
    assert agent["harness"] == "oracle"
    assert agent["status"] == "configured"
    assert agent["timeoutSeconds"] == 1200

    built_in_delete = client.delete(f"{API}/agents/built-in:oracle")
    assert built_in_delete.status_code == 403
    assert built_in_delete.json()["error"]["code"] == "RESOURCE_IMMUTABLE"

    created_environment = client.post(f"{API}/environments", json=_environment_payload()).json()
    assert created_environment["data"]["operation"]["status"] == "completed"
    copied = client.post(f"{API}/environments/local-docker/copy").json()["data"]["operation"]
    assert copied["status"] == "completed"

    environments = client.get(f"{API}/environments?type=custom").json()["data"]["items"]
    assert {item["id"] for item in environments} == {"local-docker", copied["resourceId"]}

    deleted = client.delete(f"{API}/environments/{copied['resourceId']}").json()
    assert deleted["data"]["operation"]["status"] == "completed"


def test_webui_job_create_events_and_leaderboard_update(client):
    _create_profile_prerequisites(client)

    response = client.post(f"{API}/jobs", json={"config": _job_payload(), "runImmediately": False})

    assert response.status_code == 200
    created = response.json()["data"]
    assert created["operation"]["status"] == "completed"
    assert created["job"]["name"] == "webui-smoke"
    assert created["job"]["harness"] == "oracle"
    job_id = created["job"]["id"]
    with sqlite.connect(client.app.state.settings) as conn:
        stored = sqlite.rows(
            conn, "SELECT config_json FROM webui_job_configs WHERE run_id = ?", (job_id,)
        )[0]
    config = json.loads(stored["config_json"])["harbor_overrides"]
    assert config["agent_timeout_multiplier"] == 1
    assert config["verifier_timeout_multiplier"] == 1
    assert config["agent_setup_timeout_multiplier"] == 1
    assert config["environment_build_timeout_multiplier"] == 1
    assert config["extra_instruction_paths"] == ["instructions/review.md"]
    stored_config = json.loads(stored["config_json"])
    assert stored_config["jobs_dir"] == "jobs/webui-smoke"
    assert stored_config["environment_name"] == "Local Docker"

    listing = client.get(f"{API}/jobs?q=webui-smoke").json()["data"]
    assert listing["total"] == 1
    events = client.get(f"{API}/jobs/{job_id}/events").json()["data"]
    assert events

    update = client.patch(
        f"{API}/jobs/{job_id}/leaderboard",
        json={"includeInLeaderboard": False},
    ).json()["data"]
    assert update["job"]["includeInLeaderboard"] is False
    assert update["operation"]["status"] == "completed"


def test_webui_job_requires_a_configured_custom_agent_profile(client):
    _create_profile_prerequisites(client)
    payload = _job_payload()
    payload["agentName"] = "oracle"

    response = client.post(f"{API}/jobs", json={"config": payload, "runImmediately": False})

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "INVALID_REQUEST"


def test_webui_import_dataset_operation_is_persisted_and_pollable(client, tmp_path: Path):
    dataset = tmp_path / "dataset"
    task = dataset / "hello"
    (task / "environment").mkdir(parents=True)
    (task / "instruction.md").write_text("Solve this task.", encoding="utf-8")
    (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")

    response = client.post(
        f"{API}/datasets/import",
        json={"name": "local/demo", "version": "v1", "path": str(dataset), "taskCount": 1},
    )
    operation_id = response.json()["data"]["operation"]["id"]
    operation = _wait_operation(client, operation_id)

    assert operation["status"] == "completed"
    datasets = client.get(f"{API}/datasets?q=local/demo").json()["data"]
    assert datasets["items"][0]["download"]["path"] == str(dataset)
    tasks = client.get(f"{API}/datasets/local%2Fdemo%40v1/tasks").json()["data"]
    assert tasks["items"] == [{"datasetRef": "local/demo@v1", "description": "", "name": "hello"}]

    cancel = client.post(f"{API}/datasets/local%2Fdemo%40v1/download/cancel")
    assert cancel.status_code == 422
    dataset_response = client.get(f"{API}/datasets/local%2Fdemo%40v1").json()["data"]
    assert dataset_response["download"]["status"] == "downloaded"


def test_system_restart_reports_real_supervisor_requirement(client):
    response = client.post(f"{API}/system/service/restart")

    assert response.status_code == 200
    operation = response.json()["data"]["operation"]
    assert operation["status"] == "failed"
    assert operation["error"]["code"] == "SERVICE_RESTART_UNAVAILABLE"
    persisted = client.get(f"{API}/operations/{operation['id']}").json()["data"]
    assert persisted["status"] == "failed"


def test_trial_dto_uses_real_harbor_result_fields_only():
    trial = _trial_dto(
        "job-1",
        {
            "id": "trial-1",
            "task_name": "hello",
            "agent_result": {
                "cost_usd": 0.2,
                "n_input_tokens": 1_000,
                "n_output_tokens": 2_000,
            },
            "verifier_result": {"rewards": {"pass": 1}},
            "trial_uri": "file:///not-a-log-path",
        },
    )

    assert trial["score"] == {"kind": "percentage", "value": 100.0}
    assert trial["tokenUsageM"] == 0.003
    assert trial["retryCount"] is None
    assert trial["logPath"] is None


def test_webui_reads_trials_from_harbor_native_result_layout(client, tmp_path: Path):
    job_dir = tmp_path / "native-job"
    trial_dir = job_dir / "native-job" / "trial-a"
    trial_dir.mkdir(parents=True)
    (job_dir / "native-job" / "config.json").write_text("{}", encoding="utf-8")
    (trial_dir / "trial.log").write_text("trial log\n", encoding="utf-8")
    (trial_dir / "result.json").write_text(
        json.dumps(
            {
                "id": "trial-a",
                "task_name": "hello",
                "trial_uri": trial_dir.as_uri(),
                "started_at": "2026-07-11T00:00:00+00:00",
                "finished_at": "2026-07-11T00:00:02+00:00",
            }
        ),
        encoding="utf-8",
    )
    _create_profile_prerequisites(client)
    created = client.post(
        f"{API}/jobs", json={"config": _job_payload(), "runImmediately": False}
    ).json()["data"]["job"]
    job_id = created["id"]
    with sqlite.connect(client.app.state.settings) as conn:
        conn.execute(
            "UPDATE runs SET job_dir = ?, harbor_job_name = ? WHERE id = ?",
            (str(job_dir), "native-job", job_id),
        )

    trials = client.get(f"{API}/jobs/{job_id}/trials").json()["data"]

    assert trials == [
        {
            "id": "trial-a",
            "jobId": job_id,
            "taskName": "hello",
            "status": "passed",
            "score": None,
            "retryCount": None,
            "runtimeSeconds": 2,
            "costUsd": None,
            "tokenUsageM": None,
            "logPath": str(trial_dir / "trial.log"),
        }
    ]


def test_scores_require_an_explicit_harbor_scale():
    assert _job_score({"stats": {"evals": {"test": {"pass_at_k": {"1": 0.72}}}}}) == {
        "kind": "percentage",
        "value": 72.0,
    }
    raw_metric_result = {"score": 87, "stats": {"evals": {"test": {"metrics": [{"sum": 87}]}}}}
    assert _job_score(raw_metric_result) is None
    assert _trial_dto(
        "job-1",
        {
            "id": "trial-raw",
            "task_name": "raw",
            "verifier_result": {"rewards": {"reward": 4}},
        },
    )["score"] is None


def test_pass_at_one_supports_native_and_json_keys():
    assert pass_at_one({1: 0.8, "1": 0.4}) == 0.8
    assert result_pass_at_one({"stats": {"evals": {"test": {"pass_at_k": {"1": 0.72}}}}}) == 0.72


def test_configured_jobs_dir_is_the_harbor_execution_directory(tmp_path: Path):
    configured = tmp_path / "chosen-job-folder"

    resolved = _resolve_job_dir(str(configured), tmp_path / "default-job-folder")

    assert resolved == str(configured.resolve())


def test_webui_rejects_removed_or_unsupported_contract_fields(client):
    agent = _agent_payload()
    agent["mcpServers"] = [{"name": "broken", "transport": "stdio"}]
    invalid_mcp = client.post(f"{API}/agents", json=agent)
    assert invalid_mcp.status_code == 422
    assert invalid_mcp.json()["error"]["code"] == "VALIDATION_ERROR"

    agent = _agent_payload()
    agent["status"] = "needs-token"
    assert client.post(f"{API}/agents", json=agent).status_code == 422

    environment = _environment_payload()
    environment["dockerImage"] = "not-a-harbor-environment-field"
    invalid_environment = client.post(f"{API}/environments", json=environment)
    assert invalid_environment.status_code == 422
    assert invalid_environment.json()["error"]["code"] == "VALIDATION_ERROR"

    _create_profile_prerequisites(client)
    job = _job_payload()
    job["split"] = "test"
    invalid_job = client.post(f"{API}/jobs", json={"config": job, "runImmediately": False})
    assert invalid_job.status_code == 422
    assert invalid_job.json()["error"]["code"] == "VALIDATION_ERROR"

    invalid_task_query = client.get(f"{API}/datasets/terminal-bench%402.0/tasks?split=test")
    assert invalid_task_query.status_code == 422
    assert invalid_task_query.json()["error"]["code"] == "INVALID_REQUEST"

    invalid_detail_query = client.get(f"{API}/agents/built-in:oracle?split=test")
    assert invalid_detail_query.status_code == 422
    assert invalid_detail_query.json()["error"]["code"] == "INVALID_REQUEST"


def test_operation_cancel_persists_terminal_state(settings):
    async def verify() -> None:
        tasks: dict = {}
        operations = WebUiOperationService(settings, tasks)

        async def work(progress) -> None:
            progress(25, "Working")
            await asyncio.sleep(1)

        operation = operations.submit("long-work", "system", "test", work)
        await asyncio.sleep(0)
        cancelled = operations.cancel(operation["id"])
        await asyncio.sleep(0)

        assert cancelled["status"] == "cancelled"
        assert operations.get(operation["id"])["status"] == "cancelled"

    asyncio.run(verify())


def test_webui_rejects_cancelling_terminal_job(client):
    _create_profile_prerequisites(client)
    created = client.post(
        f"{API}/jobs", json={"config": _job_payload(), "runImmediately": False}
    ).json()["data"]
    with sqlite.connect(client.app.state.settings) as conn:
        conn.execute("UPDATE runs SET status = 'completed' WHERE id = ?", (created["job"]["id"],))

    response = client.post(f"{API}/jobs/{created['job']['id']}/cancel")

    assert response.status_code == 409
    assert response.json()["error"]["code"] == "OPERATION_CONFLICT"


def test_system_health_degrades_storage_when_disk_probe_fails(client, monkeypatch):
    monkeypatch.setattr("ornnlab.services.webui_system_service._disk_usage", lambda _path: None)

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    storage = next(component for component in components if component["kind"] == "resource-storage")
    assert storage["status"] == "unavailable"
    assert storage["value"] == "unavailable"


def _create_profile_prerequisites(client) -> None:
    assert client.post(f"{API}/agents", json=_agent_payload()).status_code == 200
    assert client.post(f"{API}/environments", json=_environment_payload()).status_code == 200


def _wait_operation(client, operation_id: str) -> dict:
    for _ in range(40):
        operation = client.get(f"{API}/operations/{operation_id}").json()["data"]
        if operation["status"] in {"completed", "failed", "cancelled"}:
            return operation
        time.sleep(0.05)
    raise AssertionError(f"operation {operation_id} did not reach a terminal state")


def _agent_payload() -> dict:
    return {
        "id": "oracle-profile",
        "agentName": "Oracle profile",
        "harness": "oracle",
        "type": "custom",
        "env": [],
        "kwargs": "",
        "mcpServers": [],
        "models": [],
        "skillSources": [],
        "timeoutSeconds": 1200,
    }


def _environment_payload() -> dict:
    return {
        "id": "local-docker",
        "name": "Local Docker",
        "profileType": "custom",
        "environmentType": "docker",
        "allowedHosts": [],
        "mounts": "",
        "env": [],
        "kwargs": "",
        "forceBuild": False,
        "deleteAfterRun": True,
        "cpuPolicy": "auto",
        "memoryPolicy": "auto",
        "overrideCpus": "",
        "overrideMemoryMb": "",
        "overrideStorageMb": "",
        "overrideGpus": "",
        "overrideTpu": "",
        "dockerComposePaths": [],
    }


def _job_payload() -> dict:
    return {
        "agentSetupTimeoutMultiplier": 1,
        "agentName": "Oracle profile",
        "agentTimeoutMultiplier": 1,
        "attempts": 1,
        "concurrency": 1,
        "datasetRef": "harbor/hello-world@latest",
        "debug": False,
        "environmentPresetId": "local-docker",
        "environmentBuildTimeoutMultiplier": 1,
        "extraInstructionPaths": ["instructions/review.md"],
        "includeInLeaderboard": True,
        "jobName": "webui-smoke",
        "jobsDir": "jobs/webui-smoke",
        "maxRetries": 0,
        "metric": "mean",
        "notes": "",
        "retryExclude": "",
        "retryInclude": "",
        "retryMaxWaitSeconds": 30,
        "retryMinWaitSeconds": 1,
        "retryWaitMultiplier": 1,
        "selectedTaskNames": None,
        "timeoutMultiplier": 1,
        "verifierTimeoutMultiplier": 1,
        "verifierMode": "dataset-default",
    }
