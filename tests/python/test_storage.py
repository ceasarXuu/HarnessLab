from harnesslab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 1
    assert second == 1
    assert settings.db_path.exists()
