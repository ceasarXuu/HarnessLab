from __future__ import annotations

from ornnlab.services import docker_proxy_target


def test_discovers_local_rootful_linux_gateway(monkeypatch) -> None:
    calls: list[list[str]] = []

    def output(arguments: list[str], _description: str) -> str:
        calls.append(arguments)
        if arguments == ["context", "show"]:
            return "default"
        if arguments[:2] == ["context", "inspect"]:
            return "unix:///var/run/docker.sock"
        if "network" in arguments:
            return "172.17.0.1"
        raise AssertionError(arguments)

    monkeypatch.setattr(docker_proxy_target, "_docker_output", output)
    monkeypatch.setattr(
        docker_proxy_target,
        "_docker_json",
        lambda *_: {
            "OSType": "linux",
            "OperatingSystem": "Ubuntu 24.04",
            "SecurityOptions": ["name=seccomp"],
        },
    )
    monkeypatch.setattr(docker_proxy_target.platform, "system", lambda: "Linux")

    target = docker_proxy_target.discover_docker_proxy_target({})

    assert target.kind == "local-rootful-linux"
    assert target.bind_address == "172.17.0.1"
    network_index = calls[-1].index("network")
    assert calls[-1][network_index : network_index + 3] == ["network", "inspect", "bridge"]


def test_docker_host_override_is_classified_as_remote_without_bridge_lookup(monkeypatch) -> None:
    calls: list[list[str]] = []

    def output(arguments: list[str], _description: str) -> str:
        calls.append(arguments)
        return "default"

    monkeypatch.setattr(docker_proxy_target, "_docker_output", output)
    monkeypatch.setattr(
        docker_proxy_target,
        "_docker_json",
        lambda *_: {
            "OSType": "linux",
            "OperatingSystem": "Ubuntu 24.04",
            "SecurityOptions": [],
        },
    )

    target = docker_proxy_target.discover_docker_proxy_target(
        {"DOCKER_HOST": "ssh://builder.example"}
    )

    assert target.kind == "remote-or-virtualized"
    assert target.endpoint_kind == "remote-ssh"
    assert not any("network" in call for call in calls)


def test_docker_context_overrides_docker_host_for_every_daemon_query(monkeypatch) -> None:
    output_calls: list[list[str]] = []
    json_calls: list[list[str]] = []

    def output(arguments: list[str], _description: str) -> str:
        output_calls.append(arguments)
        if arguments[:3] == ["context", "inspect", "remote-builder"]:
            return "ssh://builder.example"
        raise AssertionError(arguments)

    def docker_json(arguments: list[str], _description: str) -> dict:
        json_calls.append(arguments)
        return {
            "OSType": "linux",
            "OperatingSystem": "Ubuntu 24.04",
            "SecurityOptions": [],
        }

    monkeypatch.setattr(docker_proxy_target, "_docker_output", output)
    monkeypatch.setattr(docker_proxy_target, "_docker_json", docker_json)

    target = docker_proxy_target.discover_docker_proxy_target(
        {
            "DOCKER_CONTEXT": "remote-builder",
            "DOCKER_HOST": "unix:///var/run/docker.sock",
        }
    )

    assert target.kind == "remote-or-virtualized"
    assert target.endpoint_kind == "remote-ssh"
    assert json_calls == [
        ["--context", "remote-builder", "info", "--format", "{{json .}}"]
    ]
    assert not any("network" in call for call in output_calls)


def test_rootless_daemon_is_classified_before_gateway_lookup(monkeypatch) -> None:
    calls: list[list[str]] = []

    def output(arguments: list[str], _description: str) -> str:
        calls.append(arguments)
        if arguments == ["context", "show"]:
            return "rootless"
        return "unix:///run/user/1000/docker.sock"

    monkeypatch.setattr(docker_proxy_target, "_docker_output", output)
    monkeypatch.setattr(
        docker_proxy_target,
        "_docker_json",
        lambda *_: {
            "OSType": "linux",
            "OperatingSystem": "Ubuntu 24.04",
            "SecurityOptions": ["name=rootless"],
        },
    )

    target = docker_proxy_target.discover_docker_proxy_target({})

    assert target.kind == "rootless"
    assert not any("network" in call for call in calls)


def test_docker_desktop_is_not_treated_as_native_local_daemon(monkeypatch) -> None:
    monkeypatch.setattr(
        docker_proxy_target,
        "_docker_output",
        lambda arguments, _: (
            "desktop-linux"
            if arguments == ["context", "show"]
            else "unix:///home/user/.docker/desktop/docker.sock"
        ),
    )
    monkeypatch.setattr(
        docker_proxy_target,
        "_docker_json",
        lambda *_: {
            "OSType": "linux",
            "OperatingSystem": "Docker Desktop",
            "SecurityOptions": [],
        },
    )

    target = docker_proxy_target.discover_docker_proxy_target({})

    assert target.kind == "docker-desktop"
    assert target.bind_address is None
