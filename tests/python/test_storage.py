import json
import sqlite3

from ornnlab.services.webui_profile_service import WebUiProfileService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite


def test_sqlite_initializes_idempotently(settings):
    first = sqlite.initialize(settings)
    second = sqlite.initialize(settings)

    assert first == 7
    assert second == 7
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
    assert "profile_type" not in agent_columns
    assert "webui_agent_configs" not in tables


def test_agent_configuration_migration_preserves_profiles_and_drops_system_presets(tmp_path):
    settings = Settings(home=tmp_path)
    migrations = sorted(sqlite.MIGRATIONS_DIR.glob("00[1-5]_*.sql"))
    existing = {
        "id": "qwen-reusable",
        "agentName": "Qwen reusable",
        "harness": "qwen-coder",
        "type": "custom",
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
        preset = {**existing, "id": "built-in:qwen-coder", "type": "built-in"}
        conn.execute(
            "INSERT INTO agents(id, name, kind, harbor_agent_name, status, profile_path, "
            "created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                preset["id"], "Qwen system preset", preset["harness"], preset["harness"],
                "draft", "/obsolete/built-in.json", "2026-07-16T00:00:00Z",
                "2026-07-16T00:00:00Z",
            ),
        )
        conn.execute(
            "INSERT INTO webui_agent_configs(agent_id, config_json) VALUES (?, ?)",
            (preset["id"], json.dumps(preset)),
        )

    assert sqlite.initialize(settings) == 7
    with sqlite3.connect(settings.db_path) as conn:
        stored = conn.execute(
            "SELECT config_json FROM agents WHERE id = ?", (existing["id"],)
        ).fetchone()[0]
        preset_count = conn.execute(
            "SELECT COUNT(*) FROM agents WHERE id = ?", (preset["id"],)
        ).fetchone()[0]

    assert json.loads(stored) == {key: value for key, value in existing.items() if key != "type"}
    assert preset_count == 0


def test_inherited_agent_environment_variable_compiles_to_harbor_template(settings):
    config = WebUiProfileService(settings).agent_harbor_config(
        {
            "env": [{"key": "OPENAI_API_KEY", "value": None}],
            "harness": "qwen-coder",
            "importPath": None,
            "kwargs": "",
            "mcpServers": [],
            "models": ["qwen3-coder"],
            "setupTimeoutSeconds": None,
            "skillSources": [],
            "timeoutSeconds": None,
            "maxTimeoutSeconds": None,
        }
    )

    assert config["env"] == {"OPENAI_API_KEY": "${OPENAI_API_KEY}"}
