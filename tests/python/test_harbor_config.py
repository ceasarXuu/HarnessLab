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

    assert config.dataset["name"] == "terminal-bench@2.0"
    assert config.agent["name"] == "oracle"


def test_capability_snapshot_records_lifecycle_spike_state():
    snapshot = HarborEngine().capability_snapshot()

    assert snapshot.lifecycle_mode == "python_api_spike_required"
    assert "Job.run" in snapshot.api_symbols
