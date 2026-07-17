from __future__ import annotations

import asyncio
import logging
import re
from collections.abc import Iterable
from pathlib import Path

import httpx

logger = logging.getLogger(__name__)

_REGISTRY_TIMEOUT_SECONDS = 6
_MANIFEST_ACCEPT = ", ".join(
    (
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.oci.image.manifest.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json",
        "application/vnd.docker.distribution.manifest.v2+json",
    )
)


class ContainerImagePlatformResolver:
    """Resolve declared image platforms without pulling image layers."""

    def __init__(self, concurrency: int = 4):
        self._cache: dict[str, list[str]] = {}
        self._semaphore = asyncio.Semaphore(concurrency)

    async def enrich_tasks(self, tasks: list[dict]) -> list[dict]:
        images = {
            image["reference"]
            for task in tasks
            if (environment := task.get("environment"))
            for image in environment.get("containerImages", [])
        }
        platform_lists = await asyncio.gather(*(self.resolve(image) for image in images))
        resolved = dict(zip(images, platform_lists, strict=True))
        for task in tasks:
            environment = task.get("environment")
            if environment is not None:
                for image in environment.get("containerImages", []):
                    image["platforms"] = resolved.get(image["reference"], [])
        return tasks

    async def resolve(self, image: str) -> list[str]:
        if image in self._cache:
            return self._cache[image]
        async with self._semaphore:
            if image not in self._cache:
                self._cache[image] = await _inspect_image(image)
        return self._cache[image]


_resolver = ContainerImagePlatformResolver()


async def resolve_local_task_environment(
    local_dataset: dict | None, dataset_ref: str, task_name: str
) -> dict | None:
    if not local_dataset or local_dataset["download"]["status"] != "downloaded":
        return None
    dataset_path = Path(local_dataset["download"]["path"]).resolve()
    task_path = dataset_path.joinpath(task_name).resolve()
    if task_path.parent != dataset_path:
        raise ValueError("invalid Dataset Task name")

    from harbor.models.task.task import Task

    from ornnlab.services.dataset_environment import parse_task_summary

    if not Task.is_valid_dir(task_path, disable_verification=True):
        raise ValueError("Dataset Task not found")
    task = parse_task_summary(task_path, dataset_ref)
    await _resolver.enrich_tasks([task])
    return task["environment"]


async def _inspect_image(image: str) -> list[str]:
    try:
        async with httpx.AsyncClient(
            follow_redirects=True, timeout=_REGISTRY_TIMEOUT_SECONDS
        ) as client:
            platforms = await _fetch_image_platforms(client, image)
        logger.info("Resolved container image platforms image=%s platforms=%s", image, platforms)
        return platforms
    except (httpx.HTTPError, RuntimeError, ValueError) as exc:
        logger.warning("Unable to resolve container image platforms image=%s error=%s", image, exc)
        return []


async def _fetch_image_platforms(client: httpx.AsyncClient, image: str) -> list[str]:
    registry, repository, reference = _parse_image_reference(image)
    manifest, headers = await _registry_json(
        client,
        f"https://{registry}/v2/{repository}/manifests/{reference}",
        repository,
        {"Accept": _MANIFEST_ACCEPT},
    )
    platforms = _platforms_from_manifest(manifest)
    if not platforms and (digest := (manifest.get("config") or {}).get("digest")):
        config, _ = await _registry_json(
            client,
            f"https://{registry}/v2/{repository}/blobs/{digest}",
            repository,
            headers,
        )
        platforms = _format_platforms([config])
    return platforms


async def _registry_json(
    client: httpx.AsyncClient,
    url: str,
    repository: str,
    headers: dict[str, str],
) -> tuple[dict, dict[str, str]]:
    response = await client.get(url, headers=headers)
    if response.status_code == 401:
        token = await _registry_token(
            client, response.headers.get("www-authenticate", ""), repository
        )
        headers = {**headers, "Authorization": f"Bearer {token}"}
        response = await client.get(url, headers=headers)
    response.raise_for_status()
    return response.json(), headers


async def _registry_token(client: httpx.AsyncClient, challenge: str, repository: str) -> str:
    if not challenge.lower().startswith("bearer "):
        raise RuntimeError("container registry requires unsupported authentication")
    params = dict(re.findall(r'(\w+)="([^"]+)"', challenge))
    realm = params.pop("realm", None)
    if not realm:
        raise RuntimeError("container registry authentication challenge has no realm")
    params.setdefault("scope", f"repository:{repository}:pull")
    response = await client.get(realm, params=params)
    response.raise_for_status()
    payload = response.json()
    token = payload.get("token") or payload.get("access_token")
    if not token:
        raise RuntimeError("container registry returned no access token")
    return token


def _parse_image_reference(image: str) -> tuple[str, str, str]:
    name, separator, digest = image.partition("@")
    reference = digest if separator else "latest"
    last_slash = name.rfind("/")
    last_colon = name.rfind(":")
    if not separator and last_colon > last_slash:
        name, reference = name[:last_colon], name[last_colon + 1 :]
    parts = name.split("/")
    if len(parts) > 1 and ("." in parts[0] or ":" in parts[0] or parts[0] == "localhost"):
        registry = parts.pop(0)
    else:
        registry = "registry-1.docker.io"
        if len(parts) == 1:
            parts.insert(0, "library")
    if registry in {"docker.io", "index.docker.io"}:
        registry = "registry-1.docker.io"
    if not all((registry, parts, reference)):
        raise ValueError("invalid container image reference")
    return registry, "/".join(parts), reference


def _platforms_from_manifest(manifest: dict) -> list[str]:
    return _format_platforms(item.get("platform") or {} for item in manifest.get("manifests", []))


def _format_platforms(items: Iterable[dict]) -> list[str]:
    values = set()
    for item in items:
        os_name = item.get("os")
        architecture = item.get("architecture")
        if not os_name or not architecture or architecture == "unknown":
            continue
        platform = f"{os_name}/{architecture}"
        if variant := item.get("variant"):
            platform = f"{platform}/{variant}"
        values.add(platform)
    return sorted(values)
