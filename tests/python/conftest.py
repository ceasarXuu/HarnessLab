from __future__ import annotations

import os
from collections.abc import Iterator

import pytest
from fastapi.testclient import TestClient

from ornnlab.app import create_app
from ornnlab.settings import Settings


@pytest.fixture
def settings(tmp_path) -> Iterator[Settings]:
    old_home = os.environ.get("ORNNLAB_HOME")
    os.environ["ORNNLAB_HOME"] = str(tmp_path)
    yield Settings(home=tmp_path)
    if old_home is None:
        os.environ.pop("ORNNLAB_HOME", None)
    else:
        os.environ["ORNNLAB_HOME"] = old_home


@pytest.fixture
def client(settings: Settings) -> Iterator[TestClient]:
    with TestClient(create_app(settings)) as active_client:
        yield active_client
