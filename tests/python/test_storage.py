from harnesslab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 2
    assert second == 2
    assert settings.db_path.exists()
