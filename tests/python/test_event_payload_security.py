from __future__ import annotations

import json
import sqlite3

from ornnlab.services.event_history_redaction import redact_historical_event_payloads
from ornnlab.services.event_service import EventService
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

SECRET = "private-auth-token-value"


def test_event_service_never_persists_or_mirrors_harbor_agent_secrets(settings):
    event = EventService(settings).append(
        "run",
        "run-secret",
        "harbor.job.running",
        {
            "config": {
                "job_name": "safe-name",
                "agent": {"env": {"ANTHROPIC_AUTH_TOKEN": SECRET}},
            },
            "capability": {"lifecycle_mode": "subprocess"},
            "artifacts": {"config_path": "/safe/config.json"},
            "stats": {"n_input_tokens": 123, "n_output_tokens": 45},
        },
    )

    with sqlite.connect(settings) as conn:
        stored = conn.execute(
            "SELECT payload_json FROM experiment_events WHERE id = ?", (event.id,)
        ).fetchone()[0]
    mirror = settings.experiments_dir / "run-secret" / "ornnlab-events.jsonl"

    assert SECRET not in stored
    assert SECRET not in mirror.read_text(encoding="utf-8")
    assert "config" not in event.payload
    assert event.payload["capability"] == {"lifecycle_mode": "subprocess"}
    assert event.payload["stats"] == {"n_input_tokens": 123, "n_output_tokens": 45}


def test_historical_redaction_scrubs_database_and_event_mirrors(settings):
    payload = {
        "config": {"agent": {"env": {"ANTHROPIC_AUTH_TOKEN": SECRET}}},
        "capability": {"lifecycle_mode": "subprocess"},
    }
    mirror = settings.experiments_dir / "run-history" / "ornnlab-events.jsonl"
    mirror.parent.mkdir(parents=True)
    mirror.write_text(
        json.dumps({"event_type": "harbor.job.running", "payload": payload}) + "\n",
        encoding="utf-8",
    )
    with sqlite.connect(settings) as conn:
        conn.execute(
            "INSERT INTO experiment_events(aggregate_type, aggregate_id, ts, event_type, "
            "severity, payload_json) VALUES ('run', 'run-history', '2026-07-22T00:00:00Z', "
            "'harbor.job.running', 'info', ?)",
            (json.dumps(payload),),
        )

    result = redact_historical_event_payloads(settings)

    with sqlite.connect(settings) as conn:
        stored = conn.execute(
            "SELECT payload_json FROM experiment_events WHERE aggregate_id = 'run-history'"
        ).fetchone()[0]
    assert result == {"databaseEvents": 1, "mirrorFiles": 1, "mirrorEvents": 1}
    assert SECRET not in stored
    assert SECRET not in mirror.read_text(encoding="utf-8")


def test_schema_migration_redacts_existing_database_events(tmp_path):
    settings = Settings(home=tmp_path)
    migrations = sorted(sqlite.MIGRATIONS_DIR.glob("00[1-8]_*.sql"))
    with sqlite3.connect(settings.db_path) as conn:
        conn.execute(
            "CREATE TABLE schema_migrations("
            "version text primary key, applied_at text not null default CURRENT_TIMESTAMP)"
        )
        for migration in migrations:
            conn.executescript(migration.read_text(encoding="utf-8"))
            conn.execute("INSERT INTO schema_migrations(version) VALUES (?)", (migration.stem,))
        conn.execute(
            "INSERT INTO experiment_events(aggregate_type, aggregate_id, ts, event_type, "
            "severity, payload_json) VALUES ('run', 'run-old', '2026-07-22T00:00:00Z', "
            "'harbor.job.running', 'info', ?)",
            (json.dumps({"config": {"agent": {"env": {"TOKEN": SECRET}}}}),),
        )

    assert sqlite.initialize(settings) == 9
    with sqlite3.connect(settings.db_path) as conn:
        payload = conn.execute("SELECT payload_json FROM experiment_events").fetchone()[0]

    assert SECRET not in payload
