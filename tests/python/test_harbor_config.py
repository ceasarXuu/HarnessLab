from harnesslab.services.harbor_engine import HarborConfigBuilder, HarborEngine


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

    assert config.job_name == "harnesslab-terminal-bench-2-0"
    assert config.dataset["name"] == "terminal-bench@2.0"
    assert config.agent["name"] == "oracle"


def test_builder_emits_valid_harbor_job_config_payload(settings, monkeypatch):
    from harbor.models.job.config import JobConfig

    monkeypatch.setenv("HARNESSLAB_TEST_ENV", "present")
    config = HarborConfigBuilder(settings).build(
        agent_config={
            "name": "oracle",
            "agent_timeout_sec": 120,
            "env": {"HARNESSLAB_TEST_ENV": None},
        },
        benchmark_name="terminal-bench",
        benchmark_version="2.0",
        n_tasks=1,
        n_attempts=1,
        n_concurrent=1,
        jobs_dir="/tmp/jobs",
        job_name="run-test",
    )

    payload = HarborConfigBuilder(settings).to_job_config_payload(config)
    job_config = JobConfig.model_validate(payload)

    assert payload["agents"][0]["override_timeout_sec"] == 120
    assert payload["agents"][0]["env"] == {"HARNESSLAB_TEST_ENV": "present"}
    assert payload["datasets"][0] == {
        "name": "terminal-bench",
        "version": "2.0",
        "n_tasks": 1,
    }
    assert job_config.job_name == "run-test"


def test_capability_snapshot_records_default_fake_adapter():
    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "fake"
    assert snapshot.config_format == "harbor.models.job.config.JobConfig"
    assert snapshot.supports_cancel is False
    assert "Job.run" in snapshot.api_symbols


def test_capability_snapshot_can_select_python_api_adapter(monkeypatch):
    monkeypatch.setenv("HARNESSLAB_HARBOR_ENGINE", "python-api")

    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "python-api"
