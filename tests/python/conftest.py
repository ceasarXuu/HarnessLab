from __future__ import annotations

import gc
import os
import sys
from collections.abc import Iterator
from pathlib import Path
from types import SimpleNamespace

import pytest
from fastapi.testclient import TestClient

from ornnlab.app import create_app
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def pytest_sessionfinish(session, exitstatus) -> None:
    """Collect lingering asyncio subprocess transports while the interpreter is
    still healthy.

    On Windows, a proactor subprocess transport that is only reclaimed at
    interpreter shutdown raises inside ``__del__`` (I/O operation on closed
    pipe), which makes CPython finalize with exit code 1 even after a fully
    green session. The ``PytestUnraisableExceptionWarning`` filter (see
    pyproject.toml) stops the warning record from pinning the transport; this
    collection then releases it safely before finalization.
    """
    gc.collect()


@pytest.fixture(autouse=True)
def job_tasks_registry_unavailable(monkeypatch) -> Iterator[None]:
    """Keep Job trial task-name resolution deterministic without network access."""

    class _UnavailableRegistry:
        async def get_dataset_metadata(self, _ref):
            raise RuntimeError("registry unavailable in tests")

    factory = SimpleNamespace(create=lambda: _UnavailableRegistry())
    monkeypatch.setattr(
        "ornnlab.services.webui_job_tasks._registry_client_factory", lambda: factory
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_dataset_service._registry_client_factory", lambda: factory
    )
    monkeypatch.setattr(
        "ornnlab.services.webui_dataset_tasks._registry_client_factory", lambda: factory
    )
    yield


@pytest.fixture(autouse=True)
def default_harbor_subprocess_simulator() -> Iterator[None]:
    old_engine = os.environ.get("ORNNLAB_HARBOR_ENGINE")
    old_command = os.environ.get("ORNNLAB_HARBOR_SUBPROCESS_COMMAND")
    old_proxy_mode = os.environ.get("ORNNLAB_DOCKER_PROXY_MODE")
    os.environ["ORNNLAB_DOCKER_PROXY_MODE"] = "off"
    if os.environ.get("ORNNLAB_REAL_HARBOR") != "1":
        simulator = Path(__file__).with_name("harbor_cli_simulator.py")
        os.environ["ORNNLAB_HARBOR_ENGINE"] = "subprocess"
        os.environ["ORNNLAB_HARBOR_SUBPROCESS_COMMAND"] = f"{sys.executable} {simulator} run"
    yield
    _restore_env("ORNNLAB_HARBOR_ENGINE", old_engine)
    _restore_env("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", old_command)
    _restore_env("ORNNLAB_DOCKER_PROXY_MODE", old_proxy_mode)


@pytest.fixture
def settings(tmp_path) -> Iterator[Settings]:
    old_home = os.environ.get("ORNNLAB_HOME")
    os.environ["ORNNLAB_HOME"] = str(tmp_path)
    configured = Settings(home=tmp_path)
    sqlite.initialize(configured)
    yield configured
    if old_home is None:
        os.environ.pop("ORNNLAB_HOME", None)
    else:
        os.environ["ORNNLAB_HOME"] = old_home


@pytest.fixture
def client(settings: Settings) -> Iterator[TestClient]:
    with TestClient(create_app(settings)) as active_client:
        yield active_client


def _restore_env(name: str, value: str | None) -> None:
    if value is None:
        os.environ.pop(name, None)
    else:
        os.environ[name] = value
