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
