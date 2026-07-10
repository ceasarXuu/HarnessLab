from __future__ import annotations

import os
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

from ornnlab.app import create_app
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


@pytest.fixture(autouse=True)
def default_harbor_subprocess_simulator() -> Iterator[None]:
    old_engine = os.environ.get("ORNNLAB_HARBOR_ENGINE")
    old_command = os.environ.get("ORNNLAB_HARBOR_SUBPROCESS_COMMAND")
    if os.environ.get("ORNNLAB_REAL_HARBOR") != "1":
        simulator = Path(__file__).with_name("harbor_cli_simulator.py")
        os.environ["ORNNLAB_HARBOR_ENGINE"] = "subprocess"
        os.environ["ORNNLAB_HARBOR_SUBPROCESS_COMMAND"] = f"{sys.executable} {simulator} run"
    yield
    _restore_env("ORNNLAB_HARBOR_ENGINE", old_engine)
    _restore_env("ORNNLAB_HARBOR_SUBPROCESS_COMMAND", old_command)


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
