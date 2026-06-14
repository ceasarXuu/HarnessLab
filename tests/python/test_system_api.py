def test_system_status_reports_core_fields(client):
    response = client.get("/api/system/status")

    assert response.status_code == 200
    payload = response.json()
    assert payload["db_schema_version"] == 2
    assert payload["data_dir"]
    assert "docker" in payload
    assert "harbor_version" in payload
