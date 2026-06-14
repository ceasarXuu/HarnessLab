from harnesslab.services.queue_service import QueueService


def _oracle_payload():
    return {
        "schema_version": 2,
        "id": "oracle",
        "name": "Oracle",
        "kind": "oracle",
        "harbor": {"agent": "oracle"},
    }


def test_agent_create_and_compile(client):
    create = client.post("/api/agents", json=_oracle_payload())
    assert create.status_code == 200
    assert create.json()["id"] == "oracle"

    compile_response = client.post("/api/agents/oracle/compile")
    assert compile_response.status_code == 200
    assert compile_response.json()["agent_config"]["name"] == "oracle"

    agents = client.get("/api/agents").json()
    assert agents[0]["status"] == "compiled"


def test_agent_update_and_soft_delete(client):
    create = client.post("/api/agents", json=_oracle_payload())
    assert create.status_code == 200

    updated_payload = _oracle_payload() | {"name": "Oracle Updated"}
    updated = client.put("/api/agents/oracle", json=updated_payload)
    assert updated.status_code == 200
    assert updated.json()["name"] == "Oracle Updated"

    deleted = client.delete("/api/agents/oracle")
    assert deleted.status_code == 200
    assert client.get("/api/agents").json() == []


def test_agent_delete_blocks_queued_run(client, settings):
    client.post("/api/agents", json=_oracle_payload())
    created = client.post(
        "/api/experiments",
        json={
            "name": "Queued",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
        },
    )
    experiment_id = created.json()["experiment"]["id"]
    QueueService(settings).enqueue_experiment(experiment_id)

    blocked = client.delete("/api/agents/oracle")
    assert blocked.status_code == 409
