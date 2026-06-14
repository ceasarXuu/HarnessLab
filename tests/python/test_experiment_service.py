def _create_agent(client):
    response = client.post(
        "/api/agents",
        json={
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
        },
    )
    assert response.status_code == 200


def test_experiment_create_and_fake_run(client):
    _create_agent(client)
    created = client.post(
        "/api/experiments",
        json={
            "name": "Smoke",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
            "benchmark_version": "2.0",
            "n_tasks": 1,
        },
    )
    assert created.status_code == 200
    experiment_id = created.json()["experiment"]["id"]

    run = client.post(f"/api/experiments/{experiment_id}/run")

    assert run.status_code == 200
    state = run.json()
    assert state["experiment"]["status"] == "completed"
    assert state["runs"][0]["status"] == "completed"
    assert state["runs"][0]["report_path"].endswith("index.html")


def test_sse_event_replay_returns_event_ids(client):
    _create_agent(client)
    created = client.post(
        "/api/experiments",
        json={
            "name": "Smoke",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
        },
    )
    experiment_id = created.json()["experiment"]["id"]

    events = client.get(f"/api/experiments/{experiment_id}/events")

    assert events.status_code == 200
    assert events.json()[0]["event_type"] == "experiment.created"
