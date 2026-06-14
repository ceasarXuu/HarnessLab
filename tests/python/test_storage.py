from harnesslab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 3
    assert second == 3
    assert settings.db_path.exists()
