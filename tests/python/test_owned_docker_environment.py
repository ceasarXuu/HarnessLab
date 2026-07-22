from ornnlab.services.owned_docker_environment import _ownership_compose_payload


def test_ownership_compose_payload_labels_main_and_sidecars():
    labels = {
        "ornnlab.managed": "true",
        "ornnlab.instance_id": "instance-1",
        "ornnlab.run_id": "run-1",
        "ornnlab.cleanup": "auto",
    }

    payload = _ownership_compose_payload(
        ["main", "database", "browser"],
        labels,
        networks=["default", "private"],
        volumes=["workspace"],
    )

    assert set(payload["services"]) == {"main", "database", "browser"}
    assert all(service["labels"] == labels for service in payload["services"].values())
    assert all(network["labels"] == labels for network in payload["networks"].values())
    assert payload["volumes"]["workspace"]["labels"] == labels
