from __future__ import annotations

import asyncio
import json
from collections.abc import Collection
from pathlib import Path

from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.container_proxy_runtime import (
    ContainerProxyRuntime,
    RuntimeProxyPolicy,
)
from ornnlab.services.experiment_service import (
    ExperimentService,
    _automatic_proxy_allowed,
    _explicit_proxy_names,
)
from ornnlab.services.leaderboard_service import LeaderboardService
from ornnlab.services.template_service import TemplateService
from tests.python.support import create_test_agent


def test_experiment_create_and_run_through_harbor_subprocess(settings):
    _create_agent(settings)
    proxy = RecordingProxyRuntime()
    service = ExperimentService(settings, proxy)
    created = service.create(_request("Smoke", ["terminal-bench"], n_tasks=1))

    state = asyncio.run(service.run(created["experiment"]["id"]))

    run = state["runs"][0]
    assert state["experiment"]["status"] == "completed"
    assert run["status"] == "completed"
    assert run["report_path"].endswith("index.html")
    assert run["score"] == 1.0
    job_dir = Path(run["job_dir"])
    harbor_config = json.loads((job_dir / "harbor.config.json").read_text())
    capability = json.loads((job_dir / "harbor.capability.json").read_text())
    result = json.loads((job_dir / "result.json").read_text())
    assert harbor_config["job_name"] == run["id"]
    assert harbor_config["agents"][0]["name"] == "oracle"
    assert harbor_config["datasets"][0]["name"] == "terminal-bench"
    assert capability["lifecycle_mode"] == "subprocess"
    assert result["status"] == "completed"
    assert proxy.policy_close_count == 1


def test_proxy_policy_closes_when_config_build_fails(settings, monkeypatch):
    _create_agent(settings)
    proxy = RecordingProxyRuntime()
    service = ExperimentService(settings, proxy)
    created = service.create(_request("Build failure", ["terminal-bench"], n_tasks=1))
    monkeypatch.setattr(
        service.builder,
        "build",
        lambda *_args, **_kwargs: (_ for _ in ()).throw(RuntimeError("config failed")),
    )

    state = asyncio.run(service.run(created["experiment"]["id"]))

    assert state["runs"][0]["status"] == "failed"
    assert proxy.policy_close_count == 1


def test_environment_allowlist_disables_auto_proxy_in_run_path(settings, monkeypatch):
    _create_agent(settings)
    proxy = RecordingProxyRuntime()
    service = ExperimentService(settings, proxy)
    created = service.create(_request("Allowlist", ["terminal-bench"], n_tasks=1))
    monkeypatch.setattr(
        service,
        "_webui_run_config",
        lambda _run_id: {
            "harbor_overrides": {
                "environment": {
                    "type": "docker",
                    "extra_allowed_hosts": ["pypi.org"],
                    "env": {"HTTPS_PROXY": "http://profile-proxy:8080"},
                }
            }
        },
    )

    state = asyncio.run(service.run(created["experiment"]["id"]))

    assert state["runs"][0]["status"] == "completed"
    assert proxy.automatic_proxy_allowed_values == [False]
    config = json.loads(
        (Path(state["runs"][0]["job_dir"]) / "harbor.config.json").read_text()
    )
    assert config["environment"]["env"]["HTTPS_PROXY"] == "http://profile-proxy:8080"
    assert "env" not in config["agents"][0]


def test_experiment_run_uses_persisted_queue(settings):
    _create_agent(settings)
    service = ExperimentService(settings)
    created = service.create(_request("Batch", ["terminal-bench", "swebench-verified"], n_tasks=2))

    state = asyncio.run(service.run(created["experiment"]["id"]))

    assert state["experiment"]["status"] == "completed"
    assert [item["status"] for item in state["runs"]] == ["completed", "completed"]


def test_run_cancel_marks_queued_run_terminal(settings):
    _create_agent(settings)
    service = ExperimentService(settings)
    created = service.create(_request("Cancelable", ["terminal-bench"]))
    run_id = created["runs"][0]["id"]

    cancelled = service.cancel_run(run_id)
    events = service.events.list_after(run_id)

    assert cancelled["status"] == "cancelled"
    assert events[0].event_type == "experiment.cancelled"


def test_failure_classification_writes_report_and_failed_status(settings):
    _create_agent(settings)
    service = ExperimentService(settings)
    created = service.create(_request("Failure", ["simulated-docker-failure"]))

    result = asyncio.run(service.run(created["experiment"]["id"]))

    run = result["runs"][0]
    assert result["experiment"]["status"] == "failed"
    assert run["failure_class"] == "docker_resource_failure"
    assert run["report_path"].endswith("index.html")
    assert (
        service.reports.read_summary(run["report_path"])["failure_class"]
        == "docker_resource_failure"
    )


def test_leaderboard_excludes_smoke_and_includes_comparable_run(settings):
    _create_agent(settings)
    service = ExperimentService(settings)
    smoke = service.create(_request("Smoke", ["terminal-bench"], n_tasks=1))
    full = service.create(_request("Full", ["terminal-bench"], n_tasks=2))

    asyncio.run(service.run(smoke["experiment"]["id"]))
    asyncio.run(service.run(full["experiment"]["id"]))
    leaderboard = LeaderboardService(settings).list("terminal-bench")

    assert [entry["id"] for entry in leaderboard] == [full["runs"][0]["id"]]
    assert leaderboard[0]["score"] == 1.0


def test_events_delete_clone_and_template_services(settings):
    _create_agent(settings)
    service = ExperimentService(settings)
    created = service.create(_request("Reusable", ["terminal-bench"], n_tasks=2))
    experiment_id = created["experiment"]["id"]

    assert service.events.list_after(experiment_id)[0].event_type == "experiment.created"
    cloned = service.clone(experiment_id)
    template = service.save_template(experiment_id, "Reusable template")
    deleted = TemplateService(settings).soft_delete(template["id"])
    removed = service.soft_delete(experiment_id)

    assert cloned["experiment"]["mode"] == "clone"
    assert cloned["runs"][0]["n_tasks"] == 2
    assert template["config"]["agent_ids"] == ["oracle"]
    assert deleted["id"] == template["id"]
    assert removed["experiment"]["id"] == experiment_id


def test_explicit_proxy_names_include_agent_and_environment_groups():
    names = _explicit_proxy_names(
        {"env": {"HTTPS_PROXY": "http://agent-proxy:8080"}},
        {"environment": {"env": {"NO_PROXY": "internal.example"}}},
    )

    assert names == {"HTTPS_PROXY", "NO_PROXY"}


def test_agent_or_environment_allowlist_disables_automatic_proxy():
    assert _automatic_proxy_allowed({"extra_allowed_hosts": ["pypi.org"]}, {}) is False
    assert (
        _automatic_proxy_allowed(
            {}, {"environment": {"extra_allowed_hosts": ["pypi.org"]}}
        )
        is False
    )
    assert _automatic_proxy_allowed({}, {"environment": {"type": "docker"}}) is True


def _create_agent(settings) -> None:
    create_test_agent(settings)


def _request(name: str, benchmarks: list[str], n_tasks: int | None = None) -> ExperimentCreate:
    return ExperimentCreate(
        name=name,
        agent_ids=["oracle"],
        benchmark_names=benchmarks,
        benchmark_version="2.0",
        n_tasks=n_tasks,
    )


class RecordingProxyRuntime(ContainerProxyRuntime):
    def __init__(self) -> None:
        self.policy_close_count = 0
        self.automatic_proxy_allowed_values: list[bool] = []

    async def prepare_policy(
        self,
        explicit_proxy_names: Collection[str] = (),
        *,
        automatic_proxy_allowed: bool = True,
    ) -> RuntimeProxyPolicy:
        self.automatic_proxy_allowed_values.append(automatic_proxy_allowed)

        async def release() -> None:
            self.policy_close_count += 1

        return RuntimeProxyPolicy({}, {}, 0, _release=release)
