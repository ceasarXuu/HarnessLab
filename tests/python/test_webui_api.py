from __future__ import annotations

import asyncio
import json
import time
from pathlib import Path

from ornnlab.services.agent_config_service import AgentConfigService
from ornnlab.services.experiment_service import _resolve_job_dir
from ornnlab.services.harbor_engine import _resolve_env
from ornnlab.services.harbor_score import pass_at_one, result_pass_at_one
from ornnlab.services.webui_dataset_service import _stored_dto
from ornnlab.services.webui_job_service import _job_score, _trial_dto
from ornnlab.services.webui_operation_service import WebUiOperationService
from ornnlab.services.webui_profile_service import WebUiProfileService
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


def test_webui_live_endpoint_is_lightweight(client):
    live = client.get(f"{API}/system/live")

    assert live.status_code == 200
    body = live.json()
    assert body["error"] is None
    assert body["data"] == {"status": "ok"}


def test_stored_dataset_without_a_local_path_is_not_marked_downloaded():
    dataset = _stored_dto(
        {
            "ref": "example@1.0",
            "name": "example",
            "version": "1.0",
            "visibility": "public",
            "task_count": 1,
            "source": "harbor registry",
            "registry_url": "https://hub.harborframework.com",
            "local_path": None,
        }
    )

    assert dataset["download"] == {"status": "not-downloaded"}


def test_webui_agent_and_environment_crud(client):
    empty_agents = client.get(f"{API}/agents").json()["data"]
    assert empty_agents["items"] == []
    assert empty_agents["total"] == 0

    harnesses = client.get(f"{API}/harnesses?limit=100").json()["data"]["items"]
    assert len(harnesses) >= 30
    claude_harness = next(item for item in harnesses if item["name"] == "claude-code")
    built_in_fields = set(claude_harness["capabilities"]["supportedFields"])
    expected_fields = {
        "customKwargs",
        "env",
        "harnessParameters",
        "mcpServers",
        "modelName",
        "skills",
        "timeouts",
    }
    assert expected_fields <= built_in_fields

    created_agent = client.post(f"{API}/agents", json=_agent_payload()).json()["data"]["operation"]
    assert created_agent["status"] == "completed"

    agent = client.get(f"{API}/agents/oracle-profile").json()["data"]
    assert agent["agentName"] == "Oracle profile"
    assert agent["harness"] == "oracle"
    assert agent["status"] == "configured"
    assert agent["timeoutSeconds"] == 1200
    assert set(agent["capabilities"]["supportedFields"]) == {"customKwargs", "env", "timeouts"}

    built_in_parameters = {item["key"] for item in claude_harness["capabilities"]["parameters"]}
    assert {
        "max_turns",
        "reasoning_effort",
        "allowed_tools",
        "max_thinking_tokens",
    } <= built_in_parameters
    max_thinking_tokens = next(
        item
        for item in claude_harness["capabilities"]["parameters"]
        if item["key"] == "max_thinking_tokens"
    )
    assert max_thinking_tokens["kind"] == "number"
    assert "MAX_THINKING_TOKENS" not in claude_harness["capabilities"]["environmentVariables"]
    claude_auth_modes = {
        item["value"]: item["environmentVariables"]
        for item in claude_harness["capabilities"]["authenticationModes"]
    }
    assert claude_auth_modes == {
        "anthropic-api": [
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_BASE_URL",
        ],
        "oauth": ["CLAUDE_CODE_OAUTH_TOKEN"],
        "bedrock": [
            "AWS_BEARER_TOKEN_BEDROCK",
            "AWS_ACCESS_KEY_ID",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
            "AWS_PROFILE",
            "AWS_REGION",
            "ANTHROPIC_SMALL_FAST_MODEL_AWS_REGION",
            "DISABLE_PROMPT_CACHING",
        ],
    }
    assert claude_harness["capabilities"]["environmentVariables"] == [
        "CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING",
        "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
    ]

    qwen_harness = next(item for item in harnesses if item["name"] == "qwen-coder")
    assert qwen_harness["capabilities"]["environmentVariables"] == [
        "OPENAI_API_KEY",
        "OPENAI_BASE_URL",
    ]
    qwen_parameters = {
        item["key"]: item for item in qwen_harness["capabilities"]["parameters"]
    }
    assert "api_key" not in qwen_parameters
    assert "base_url" not in qwen_parameters

    openhands_agent = next(item for item in harnesses if item["name"] == "openhands")
    openhands_parameters = {
        item["key"]: item for item in openhands_agent["capabilities"]["parameters"]
    }
    assert "reasoning_effort" not in openhands_parameters
    assert "temperature" not in openhands_parameters
    assert "LLM_REASONING_EFFORT" in openhands_agent["capabilities"]["environmentVariables"]
    assert "LLM_TEMPERATURE" in openhands_agent["capabilities"]["environmentVariables"]

    claude_profile = _agent_payload()
    claude_profile.update({
        "id": "claude-reusable",
        "agentName": "Claude reusable profile",
        "harness": "claude-code",
        "authenticationMode": "oauth",
        "env": [{"key": "CLAUDE_CODE_OAUTH_TOKEN", "value": None}],
        "models": ["claude-haiku-4-5", "claude-sonnet-4-5"],
    })
    updated = client.post(f"{API}/agents", json=claude_profile)
    assert updated.status_code == 200
    saved_built_in = client.get(f"{API}/agents/claude-reusable").json()["data"]
    assert saved_built_in["agentName"] == "Claude reusable profile"
    assert saved_built_in["models"] == ["claude-haiku-4-5", "claude-sonnet-4-5"]
    assert saved_built_in["authenticationMode"] == "oauth"
    compiled = WebUiProfileService(client.app.state.settings).agent_harbor_config(saved_built_in)
    assert compiled["env"]["CLAUDE_FORCE_OAUTH"] == "1"

    assert client.get(f"{API}/agents/built-in:oracle").status_code == 404

    created_environment = client.post(f"{API}/environments", json=_environment_payload()).json()
    assert created_environment["data"]["operation"]["status"] == "completed"
    copied = client.post(f"{API}/environments/local-docker/copy").json()["data"]["operation"]
    assert copied["status"] == "completed"

    environments = client.get(f"{API}/environments?type=custom").json()["data"]["items"]
    assert {item["id"] for item in environments} == {"local-docker", copied["resourceId"]}

    deleted = client.delete(f"{API}/environments/{copied['resourceId']}").json()
    assert deleted["data"]["operation"]["status"] == "completed"


def test_legacy_claude_thinking_environment_variable_moves_to_harness_parameters(client):
    payload = _agent_payload()
    payload.update(
        {"id": "claude-profile", "agentName": "Claude profile", "harness": "claude-code"}
    )
    payload["env"] = [{"key": "MAX_THINKING_TOKENS", "value": "4096"}]

    response = client.post(f"{API}/agents", json=payload)

    assert response.status_code == 200
    saved = client.get(f"{API}/agents/claude-profile").json()["data"]
    assert saved["env"] == []
    assert json.loads(saved["kwargs"])["max_thinking_tokens"] == "4096"


def test_agent_environment_variables_preserve_inherited_and_fixed_values(client, settings):
    payload = _agent_payload()
    payload["env"] = [
        {"key": "INHERITED_TOKEN", "value": None},
        {"key": "API_BASE_URL", "value": "https://example.test"},
    ]

    response = client.post(f"{API}/agents", json=payload)

    assert response.status_code == 200
    agent = client.get(f"{API}/agents/oracle-profile").json()["data"]
    assert agent["env"] == payload["env"]
    assert WebUiProfileService(settings).agent_harbor_config(agent)["env"] == {
        "INHERITED_TOKEN": "${INHERITED_TOKEN}",
        "API_BASE_URL": "https://example.test",
    }


def test_agent_environment_variable_inheritance_resolves_without_logging_values(
    monkeypatch, caplog
):
    monkeypatch.setenv("AVAILABLE_TOKEN", "private-value")

    resolved = _resolve_env({"AVAILABLE_TOKEN": None, "MISSING_TOKEN": None})

    assert resolved == {"AVAILABLE_TOKEN": "private-value"}
    assert "MISSING_TOKEN" in caplog.records[0].variable_name
    assert "private-value" not in caplog.text


def test_webui_job_create_events_and_leaderboard_update(client):
    _create_profile_prerequisites(client)

    response = client.post(f"{API}/jobs", json={"config": _job_payload(), "runImmediately": False})

    assert response.status_code == 200
    created = response.json()["data"]
    assert created["operation"]["status"] == "completed"
    assert created["job"]["name"] == "webui-smoke"
    assert created["job"]["harness"] == "oracle"
    assert created["job"]["model"] == "oracle-secondary"
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
    assert stored_config["model"] == "oracle-secondary"
    compiled_agent = AgentConfigService(client.app.state.settings).config(
        "oracle-profile", stored_config["model"]
    )
    assert compiled_agent["model_name"] == "oracle-secondary"

    listing = client.get(f"{API}/jobs?q=webui-smoke").json()["data"]
    assert listing["total"] == 1
    events = client.get(f"{API}/jobs/{job_id}/events").json()["data"]
    assert any(event["message"] == "webui.job.configured" for event in events)

    update = client.patch(
        f"{API}/jobs/{job_id}/leaderboard",
        json={"includeInLeaderboard": False},
    ).json()["data"]
    assert update["job"]["includeInLeaderboard"] is False
    assert update["operation"]["status"] == "completed"


def test_webui_job_rejects_model_outside_agent_template(client):
    _create_profile_prerequisites(client)
    payload = _job_payload()
    payload["modelName"] = "not-configured"

    response = client.post(f"{API}/jobs", json={"config": payload, "runImmediately": False})

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "INVALID_REQUEST"


def test_webui_job_uses_a_saved_agent_profile_backed_by_a_built_in_harness(client):
    _create_profile_prerequisites(client)
    payload = _job_payload()
    payload["agentName"] = "Oracle profile"
    payload["modelName"] = "oracle-primary"

    response = client.post(f"{API}/jobs", json={"config": payload, "runImmediately": False})

    assert response.status_code == 200
    assert response.json()["data"]["job"]["harness"] == "oracle"
    with sqlite.connect(client.app.state.settings) as conn:
        persisted = sqlite.rows(conn, "SELECT id FROM agents WHERE id = ?", ("oracle-profile",))
    assert persisted == [{"id": "oracle-profile"}]


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
    assert tasks["items"] == [
        {
            "datasetRef": "local/demo@v1",
            "description": "",
            "environment": {
                "allowedHosts": [],
                "buildTimeoutSeconds": 600.0,
                "definitions": [],
                "dockerImage": None,
                "networkMode": "public",
                "os": "linux",
                "resources": {
                    "cpus": None,
                    "gpuTypes": [],
                    "gpus": None,
                    "memoryMb": None,
                    "storageMb": None,
                    "tpu": None,
                },
                "workdir": None,
            },
            "name": "hello",
        }
    ]

    cancel = client.post(f"{API}/datasets/local%2Fdemo%40v1/download/cancel")
    assert cancel.status_code == 422
    dataset_response = client.get(f"{API}/datasets/local%2Fdemo%40v1").json()["data"]
    assert dataset_response["download"]["status"] == "downloaded"


def test_webui_external_dataset_storage_routes_preserve_files(client, tmp_path: Path):
    original = _dataset_directory(tmp_path / "original")
    relocated = _dataset_directory(tmp_path / "relocated")
    imported = client.post(
        f"{API}/datasets/import",
        json={"name": "local/storage", "version": "v1", "path": str(original), "taskCount": 1},
    ).json()["data"]["operation"]
    assert _wait_operation(client, imported["id"])["status"] == "completed"

    default_parent = client.get(f"{API}/datasets/storage/default-parent").json()["data"]
    assert default_parent["parentPath"] == str(client.app.state.settings.datasets_dir)

    delete = client.delete(f"{API}/datasets/local%2Fstorage%40v1/local")
    assert delete.status_code == 422
    assert delete.json()["error"]["code"] == "INVALID_REQUEST"

    moved = client.post(
        f"{API}/datasets/local%2Fstorage%40v1/relocate", json={"path": str(relocated)}
    ).json()["data"]["operation"]
    assert moved["status"] == "completed"
    detail = client.get(f"{API}/datasets/local%2Fstorage%40v1").json()["data"]
    assert detail["download"]["updatedAt"]
    assert detail["download"] == {
        "path": str(relocated),
        "sizeBytes": detail["download"]["sizeBytes"],
        "status": "downloaded",
        "storageKind": "external",
        "updatedAt": detail["download"]["updatedAt"],
    }

    removed = client.delete(f"{API}/datasets/local%2Fstorage%40v1/registration").json()["data"][
        "operation"
    ]
    assert removed["status"] == "completed"
    assert original.is_dir()
    assert relocated.is_dir()
    datasets = client.get(f"{API}/datasets?q=local/storage").json()["data"]
    assert datasets["items"] == []


def test_system_directory_picker_returns_the_native_selection(client, tmp_path: Path, monkeypatch):
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._choose_native_directory", lambda: str(tmp_path)
    )

    response = client.post(f"{API}/system/directory-picker")

    assert response.status_code == 200
    assert response.json()["data"] == {"path": str(tmp_path)}


def test_docker_start_command_is_persisted_executed_without_shell_and_exposed(
    client, monkeypatch
):
    executed: list[list[str]] = []
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._run_checked",
        lambda command: executed.append(command),
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_system_service._wait_for_docker",
        lambda: None,
        raising=False,
    )

    saved = client.put(f"{API}/system/docker/start-command", json={"command": "colima start"})
    docker = next(
        item
        for item in client.get(f"{API}/system/health").json()["data"]["items"]
        if item["kind"] == "docker"
    )
    started = client.post(
        f"{API}/system/docker/start", json={"command": "colima start"}
    ).json()["data"]["operation"]

    assert saved.json()["data"] == {"command": "colima start"}
    assert docker["startCommand"] == "colima start"
    assert _wait_operation(client, started["id"])["status"] == "completed"
    assert executed == [["colima", "start"]]


def test_docker_start_command_rejects_shell_operators(client):
    response = client.put(
        f"{API}/system/docker/start-command",
        json={"command": "colima start && whoami"},
    )

    assert response.status_code == 422
    assert response.json()["error"]["code"] == "VALIDATION_ERROR"


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


def test_job_dto_only_marks_a_failed_job_resumable_when_harbor_config_exists(
    client, tmp_path: Path
):
    _create_profile_prerequisites(client)
    created = client.post(
        f"{API}/jobs", json={"config": _job_payload(), "runImmediately": False}
    ).json()["data"]["job"]
    job_id = created["id"]
    job_dir = tmp_path / "resume-root"
    job_dir.mkdir()
    with sqlite.connect(client.app.state.settings) as conn:
        conn.execute(
            "UPDATE runs SET status = 'failed', job_dir = ?, harbor_job_name = ? WHERE id = ?",
            (str(job_dir), "native-job", job_id),
        )

    unavailable = client.get(f"{API}/jobs/{job_id}").json()["data"]
    rejected_resume = client.post(f"{API}/jobs/{job_id}/resume")
    native_dir = job_dir / "native-job"
    native_dir.mkdir()
    (native_dir / "config.json").write_text("{}", encoding="utf-8")
    available = client.get(f"{API}/jobs/{job_id}").json()["data"]

    assert unavailable["canResume"] is False
    assert rejected_resume.status_code == 422
    assert rejected_resume.json()["error"]["code"] == "INVALID_REQUEST"
    assert available["canResume"] is True


def test_copy_job_config_returns_an_editable_draft_without_creating_a_job(client):
    _create_profile_prerequisites(client)
    payload = _job_payload()
    payload.update({
        "jobName": "copy-source",
        "jobsDir": "/tmp/shared-jobs",
        "selectedTaskNames": ["hello"],
        "notes": "keep this note",
        "retryInclude": "TimeoutError, NetworkError",
    })
    created = client.post(
        f"{API}/jobs", json={"config": payload, "runImmediately": False}
    ).json()["data"]["job"]
    with sqlite.connect(client.app.state.settings) as conn:
        before = sqlite.rows(conn, "SELECT COUNT(*) AS total FROM runs")[0]["total"]

    response = client.get(f"{API}/jobs/{created['id']}/copy-config")

    with sqlite.connect(client.app.state.settings) as conn:
        after = sqlite.rows(conn, "SELECT COUNT(*) AS total FROM runs")[0]["total"]
    assert response.status_code == 200
    assert response.json()["data"] == {
        **payload,
        "jobName": "copy-source-copy",
    }
    assert before == after

    with sqlite.connect(client.app.state.settings) as conn:
        conn.execute("DELETE FROM webui_job_configs WHERE run_id = ?", (created["id"],))
    unavailable = client.get(f"{API}/jobs/{created['id']}/copy-config")
    assert unavailable.status_code == 422
    assert unavailable.json()["error"]["code"] == "INVALID_REQUEST"


def test_scores_require_an_explicit_harbor_scale():
    assert _job_score({"stats": {"evals": {"test": {"pass_at_k": {"1": 0.72}}}}}) == {
        "kind": "percentage",
        "value": 72.0,
    }
    raw_metric_result = {"score": 87, "stats": {"evals": {"test": {"metrics": [{"sum": 87}]}}}}
    assert _job_score(raw_metric_result) is None
    assert (
        _trial_dto(
            "job-1",
            {
                "id": "trial-raw",
                "task_name": "raw",
                "verifier_result": {"rewards": {"reward": 4}},
            },
        )["score"]
        is None
    )


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
    monkeypatch.setattr("ornnlab.services.system_health_probe._disk_usage", lambda _path: None)

    components = client.get(f"{API}/system/health").json()["data"]["items"]

    storage = next(component for component in components if component["kind"] == "resource-storage")
    assert storage["state"] == "unavailable"
    assert storage["availableBytes"] is None
    assert storage["totalBytes"] is None


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


def _dataset_directory(path: Path) -> Path:
    task = path / "hello"
    (task / "environment").mkdir(parents=True)
    (task / "instruction.md").write_text("Solve this task.", encoding="utf-8")
    (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")
    return path


def _agent_payload() -> dict:
    return {
        "id": "oracle-profile",
        "agentName": "Oracle profile",
        "harness": "oracle",
        "env": [],
        "kwargs": "",
        "mcpServers": [],
        "models": ["oracle-primary", "oracle-secondary"],
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
        "modelName": "oracle-secondary",
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
