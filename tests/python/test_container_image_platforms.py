import asyncio

import httpx

from ornnlab.services import container_image_platforms
from ornnlab.services.container_image_platforms import (
    ContainerImagePlatformResolver,
    _fetch_image_platforms,
    _parse_image_reference,
    _platforms_from_manifest,
)


def test_extracts_platforms_from_multi_architecture_manifest():
    payload = {
        "manifests": [
            {"platform": {"architecture": "arm64", "os": "linux", "variant": "v8"}},
            {"platform": {"architecture": "amd64", "os": "linux"}},
            {"platform": {"architecture": "unknown", "os": "unknown"}},
        ]
    }

    assert _platforms_from_manifest(payload) == ["linux/amd64", "linux/arm64/v8"]


def test_parses_registry_and_docker_hub_image_references():
    assert _parse_image_reference("ghcr.io/example/task:2.0") == (
        "ghcr.io",
        "example/task",
        "2.0",
    )
    assert _parse_image_reference("python:3.13") == (
        "registry-1.docker.io",
        "library/python",
        "3.13",
    )


def test_resolver_enriches_tasks_and_caches_by_image(monkeypatch):
    calls = []

    async def inspect(image: str) -> list[str]:
        calls.append(image)
        return ["linux/amd64"]

    monkeypatch.setattr(container_image_platforms, "_inspect_image", inspect)
    resolver = ContainerImagePlatformResolver()
    tasks = [
        {"environment": {"dockerImage": "example/task:1.0"}},
        {"environment": {"dockerImage": "example/task:1.0"}},
        {"environment": {"dockerImage": None}},
        {"environment": None},
    ]

    asyncio.run(resolver.enrich_tasks(tasks))
    asyncio.run(resolver.enrich_tasks(tasks))

    assert calls == ["example/task:1.0"]
    assert tasks[0]["environment"]["imagePlatforms"] == ["linux/amd64"]
    assert tasks[1]["environment"]["imagePlatforms"] == ["linux/amd64"]
    assert tasks[2]["environment"]["imagePlatforms"] == []


def test_fetches_single_architecture_image_config_after_bearer_authentication():
    def handle(request: httpx.Request) -> httpx.Response:
        if request.url.path == "/token":
            return httpx.Response(200, json={"token": "registry-token"})
        if request.url.path.endswith("/manifests/2.0"):
            if request.headers.get("authorization") != "Bearer registry-token":
                return httpx.Response(
                    401,
                    headers={
                        "www-authenticate": (
                            'Bearer realm="https://auth.example/token",service="registry.example"'
                        )
                    },
                )
            return httpx.Response(200, json={"config": {"digest": "sha256:config"}})
        if request.url.path.endswith("/blobs/sha256:config"):
            return httpx.Response(200, json={"os": "linux", "architecture": "amd64"})
        return httpx.Response(404)

    async def fetch() -> list[str]:
        transport = httpx.MockTransport(handle)
        async with httpx.AsyncClient(transport=transport) as client:
            return await _fetch_image_platforms(client, "registry.example/example/task:2.0")

    assert asyncio.run(fetch()) == ["linux/amd64"]
