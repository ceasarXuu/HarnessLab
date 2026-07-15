import json
import sqlite3

from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 6
    assert second == 6
    assert settings.db_path.exists()


def test_agent_configuration_has_one_canonical_storage_model(settings):
    with sqlite.connect(settings) as conn:
        agent_columns = {
            row["name"] for row in conn.execute("PRAGMA table_info(agents)").fetchall()
        }
        tables = {
            row["name"]
            for row in conn.execute("SELECT name FROM sqlite_master WHERE type = 'table'")
        }

    assert "config_json" in agent_columns
    assert "profile_path" not in agent_columns
    assert "webui_agent_configs" not in tables


def test_agent_configuration_migration_preserves_existing_webui_profile(tmp_path):
    settings = Settings(home=tmp_path)
    migrations = sorted(sqlite.MIGRATIONS_DIR.glob("00[1-5]_*.sql"))
    existing = {
        "id": "built-in:qwen-coder",
        "agentName": "Qwen reusable",
        "harness": "qwen-coder",
        "type": "built-in",
        "env": [{"key": "OPENAI_API_KEY", "value": None}],
        "kwargs": "reasoning_effort=high",
        "mcpServers": [],
        "models": ["qwen3-coder"],
        "skillSources": [],
    }
    with sqlite3.connect(settings.db_path) as conn:
        conn.execute(
            "CREATE TABLE schema_migrations("
            "version text primary key, applied_at text not null default CURRENT_TIMESTAMP)"
        )
        for migration in migrations:
            conn.executescript(migration.read_text(encoding="utf-8"))
            conn.execute("INSERT INTO schema_migrations(version) VALUES (?)", (migration.stem,))
        conn.execute(
            "INSERT INTO agents(id, name, kind, harbor_agent_name, status, profile_path, "
            "created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                existing["id"], existing["agentName"], existing["harness"],
                existing["harness"], "draft", "/obsolete/profile.json",
                "2026-07-16T00:00:00Z", "2026-07-16T00:00:00Z",
            ),
        )
        conn.execute(
            "INSERT INTO webui_agent_configs(agent_id, config_json) VALUES (?, ?)",
            (existing["id"], json.dumps(existing)),
        )

    assert sqlite.initialize(settings) == 6
    with sqlite3.connect(settings.db_path) as conn:
        stored = conn.execute(
            "SELECT config_json FROM agents WHERE id = ?", (existing["id"],)
        ).fetchone()[0]

    assert json.loads(stored) == existing
