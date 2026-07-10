from ornnlab.models.experiment import ExperimentCreate
from ornnlab.services.agent_service import AgentService
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.services.queue_service import QueueService


def _oracle_payload(name: str = "Oracle") -> dict:
    return {
        "schema_version": 2,
        "id": "oracle",
        "name": name,
        "kind": "oracle",
        "harbor": {"agent": "oracle"},
    }


def test_agent_create_and_compile(settings):
    service = AgentService(settings)

    created = service.create(_oracle_payload())
    compiled = service.compile("oracle")

    assert created["id"] == "oracle"
    assert compiled["agent_config"]["name"] == "oracle"
    assert service.list()[0]["status"] == "compiled"


def test_agent_update_and_soft_delete(settings):
    service = AgentService(settings)
    service.create(_oracle_payload())

    updated = service.update("oracle", _oracle_payload("Oracle Updated"))
    deleted = service.soft_delete("oracle")

    assert updated["name"] == "Oracle Updated"
    assert deleted["id"] == "oracle"
    assert service.list() == []


def test_agent_delete_blocks_queued_run(settings):
    agents = AgentService(settings)
    agents.create(_oracle_payload())
    experiment = ExperimentService(settings).create(
        ExperimentCreate(name="Queued", agent_ids=["oracle"], benchmark_names=["terminal-bench"])
    )
    QueueService(settings).enqueue_experiment(experiment["experiment"]["id"])

    try:
        agents.soft_delete("oracle")
    except RuntimeError as exc:
        assert str(exc) == "agent has queued or running runs"
    else:
        raise AssertionError("queued Agent must not be deleted")
