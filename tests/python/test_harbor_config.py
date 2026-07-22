from ornnlab.services.harbor_engine import HarborConfigBuilder, HarborEngine


def test_harbor_config_builder_creates_dataset_ref(settings):
    config = HarborConfigBuilder(settings).build(
        agent_config={"name": "oracle"},
        benchmark_name="terminal-bench",
        benchmark_version="2.0",
        n_tasks=1,
        n_attempts=1,
        n_concurrent=1,
        jobs_dir="/tmp/jobs",
    )

    assert config.job_name == "ornnlab-terminal-bench-2-0"
    assert config.dataset["name"] == "terminal-bench@2.0"
    assert config.agent["name"] == "oracle"
    assert config.environment["import_path"].endswith(":OwnedDockerEnvironment")
    assert config.environment["kwargs"]["ornnlab_run_id"] == config.job_name
    assert config.environment["kwargs"]["ornnlab_instance_id"] == settings.instance_id


def test_builder_emits_valid_harbor_job_config_payload(settings, monkeypatch):
    from harbor.models.job.config import JobConfig

    monkeypatch.setenv("ORNNLAB_TEST_ENV", "present")
    config = HarborConfigBuilder(settings).build(
        agent_config={
            "name": "oracle",
            "model_name": "oracle-secondary",
            "agent_timeout_sec": 120,
            "env": {"ORNNLAB_TEST_ENV": None},
        },
        benchmark_name="terminal-bench",
        benchmark_version="2.0",
        n_tasks=1,
        n_attempts=1,
        n_concurrent=1,
        jobs_dir="/tmp/jobs",
        job_name="run-test",
        overrides={
            "agent_timeout_multiplier": 1.2,
            "verifier_timeout_multiplier": 1.3,
            "agent_setup_timeout_multiplier": 1.4,
            "environment_build_timeout_multiplier": 1.5,
            "extra_instruction_paths": ["instructions/review.md"],
        },
    )

    payload = HarborConfigBuilder(settings).to_job_config_payload(config)
    job_config = JobConfig.model_validate(payload)

    assert payload["agents"][0]["override_timeout_sec"] == 120
    assert payload["agents"][0]["model_name"] == "oracle-secondary"
    assert payload["agents"][0]["env"] == {"ORNNLAB_TEST_ENV": "present"}
    assert payload["datasets"][0] == {
        "name": "terminal-bench",
        "version": "2.0",
        "n_tasks": 1,
    }
    assert payload["agent_timeout_multiplier"] == 1.2
    assert payload["verifier_timeout_multiplier"] == 1.3
    assert payload["agent_setup_timeout_multiplier"] == 1.4
    assert payload["environment_build_timeout_multiplier"] == 1.5
    assert payload["extra_instruction_paths"] == ["instructions/review.md"]
    assert job_config.job_name == "run-test"


def test_builder_preserves_environment_kwargs_and_marks_explicit_retention(settings):
    config = HarborConfigBuilder(settings).build(
        agent_config={"name": "oracle"},
        benchmark_name="terminal-bench",
        benchmark_version="2.0",
        n_tasks=1,
        n_attempts=1,
        n_concurrent=1,
        jobs_dir="/tmp/jobs",
        job_name="display-name",
        owner_run_id="run-owner",
        overrides={
            "environment": {
                "type": "docker",
                "delete": True,
                "kwargs": {"keep_containers": True, "custom": "value"},
            }
        },
    )

    assert config.environment["kwargs"] == {
        "keep_containers": True,
        "custom": "value",
        "ornnlab_instance_id": settings.instance_id,
        "ornnlab_run_id": "run-owner",
        "ornnlab_cleanup_policy": "retain",
    }


def test_builder_does_not_replace_user_custom_environment(settings):
    config = HarborConfigBuilder(settings).build(
        agent_config={"name": "oracle"},
        benchmark_name="terminal-bench",
        benchmark_version="2.0",
        n_tasks=1,
        n_attempts=1,
        n_concurrent=1,
        jobs_dir="/tmp/jobs",
        owner_run_id="run-owner",
        overrides={
            "environment": {
                "import_path": "example.environments:RemoteEnvironment",
                "kwargs": {"region": "test-region"},
            }
        },
    )

    assert config.environment == {
        "import_path": "example.environments:RemoteEnvironment",
        "kwargs": {"region": "test-region"},
    }


def test_capability_snapshot_records_default_subprocess_adapter(monkeypatch):
    monkeypatch.delenv("ORNNLAB_HARBOR_ENGINE", raising=False)

    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "subprocess"
    assert snapshot.config_format == "harbor.models.job.config.JobConfig"
    assert snapshot.supports_cancel is True
    assert "Job.run" in snapshot.api_symbols


def test_capability_snapshot_can_select_python_api_adapter(monkeypatch):
    monkeypatch.setenv("ORNNLAB_HARBOR_ENGINE", "python-api")

    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "python-api"


def test_subprocess_capability_snapshot_records_cancel_support(monkeypatch):
    monkeypatch.setenv("ORNNLAB_HARBOR_ENGINE", "subprocess")

    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "subprocess"
    assert snapshot.supports_cancel is True


def test_fake_engine_mode_is_rejected(monkeypatch):
    monkeypatch.setenv("ORNNLAB_HARBOR_ENGINE", "fake")

    try:
        HarborEngine()
    except ValueError as exc:
        assert "python-api, subprocess, cli, or real" in str(exc)
    else:
        raise AssertionError("fake Harbor engine mode must not be accepted")
