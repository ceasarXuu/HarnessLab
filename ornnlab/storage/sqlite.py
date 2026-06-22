from __future__ import annotations

import sqlite3
from collections.abc import Iterable
from pathlib import Path

from ornnlab.settings import Settings

MIGRATIONS_DIR = Path(__file__).with_name("migrations")

_ensured_dirs: set[str] = set()


def connect(settings: Settings) -> sqlite3.Connection:
    home_str = str(settings.home)
    if home_str not in _ensured_dirs:
        settings.ensure_dirs()
        _ensured_dirs.add(home_str)
    conn = sqlite3.connect(settings.db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA busy_timeout = 5000")
    return conn


def initialize(settings: Settings) -> int:
    with connect(settings) as conn:
        conn.execute("PRAGMA journal_mode = WAL")
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations("
            "version text primary key, applied_at text not null default CURRENT_TIMESTAMP)"
        )
        applied = {row["version"] for row in conn.execute("SELECT version FROM schema_migrations")}
        latest = 0
        for migration in sorted(MIGRATIONS_DIR.glob("*.sql")):
            version = migration.stem
            latest = max(latest, int(version.split("_", 1)[0]))
            if version in applied:
                continue
            conn.executescript(migration.read_text(encoding="utf-8"))
            conn.execute("INSERT INTO schema_migrations(version) VALUES (?)", (version,))
        return latest


def rows(conn: sqlite3.Connection, query: str, params: Iterable[object] = ()) -> list[dict]:
    return [dict(row) for row in conn.execute(query, tuple(params)).fetchall()]
