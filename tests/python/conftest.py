from __future__ import annotations

import os
from collections.abc import Iterator

import pytest
from fastapi.testclient import TestClient

from harnesslab.app import create_app
from harnesslab.settings import Settings


@pytest.fixture
def settings(tmp_path) -> Iterator[Settings]:
    old_home = os.environ.get("HARNESSLAB_HOME")
    os.environ["HARNESSLAB_HOME"] = str(tmp_path)
    yield Settings(home=tmp_path)
    if old_home is None:
        os.environ.pop("HARNESSLAB_HOME", None)
    else:
        os.environ["HARNESSLAB_HOME"] = old_home


@pytest.fixture
def client(settings: Settings) -> TestClient:
    return TestClient(create_app(settings))
