from ornnlab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 4
    assert second == 4
    assert settings.db_path.exists()
