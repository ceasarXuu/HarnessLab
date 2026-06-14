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
    assert state["runs"][0]["score"] == 1.0


def test_experiment_run_uses_persisted_queue(client):
    _create_agent(client)
    created = client.post(
        "/api/experiments",
        json={
            "name": "Batch",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench", "swebench-verified"],
            "benchmark_version": "2.0",
            "n_tasks": 2,
        },
    )
    experiment_id = created.json()["experiment"]["id"]

    run = client.post(f"/api/experiments/{experiment_id}/run")

    assert run.status_code == 200
    state = run.json()
    assert state["experiment"]["status"] == "completed"
    assert [item["status"] for item in state["runs"]] == ["completed", "completed"]


def test_run_cancel_marks_queued_run_terminal(client):
    _create_agent(client)
    created = client.post(
        "/api/experiments",
        json={
            "name": "Cancelable",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
        },
    )
    run_id = created.json()["runs"][0]["id"]

    cancelled = client.post(f"/api/runs/{run_id}/cancel")

    assert cancelled.status_code == 200
    assert cancelled.json()["status"] == "cancelled"
    events = client.get(f"/api/runs/{run_id}/events").json()
    assert events[0]["event_type"] == "experiment.cancelled"


def test_failure_classification_writes_report_and_failed_status(client):
    _create_agent(client)
    created = client.post(
        "/api/experiments",
        json={
            "name": "Failure",
            "agent_ids": ["oracle"],
            "benchmark_names": ["fake-docker-failure"],
        },
    )
    experiment_id = created.json()["experiment"]["id"]

    result = client.post(f"/api/experiments/{experiment_id}/run").json()

    assert result["experiment"]["status"] == "failed"
    assert result["runs"][0]["status"] == "failed"
    assert result["runs"][0]["failure_class"] == "docker_resource_failure"
    assert result["runs"][0]["report_path"].endswith("index.html")


def test_leaderboard_excludes_smoke_and_includes_comparable_run(client):
    _create_agent(client)
    smoke = client.post(
        "/api/experiments",
        json={
            "name": "Smoke",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
            "n_tasks": 1,
        },
    ).json()
    full = client.post(
        "/api/experiments",
        json={
            "name": "Full",
            "agent_ids": ["oracle"],
            "benchmark_names": ["terminal-bench"],
            "n_tasks": 2,
        },
    ).json()

    client.post(f"/api/experiments/{smoke['experiment']['id']}/run")
    client.post(f"/api/experiments/{full['experiment']['id']}/run")
    leaderboard = client.get("/api/leaderboard?benchmark=terminal-bench").json()

    assert [entry["id"] for entry in leaderboard] == [full["runs"][0]["id"]]
    assert leaderboard[0]["score"] == 1.0


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
