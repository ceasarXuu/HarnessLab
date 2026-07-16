PRAGMA foreign_keys = OFF;

CREATE TABLE agents_v3(
  id TEXT PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  harness TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active', 'deleted')),
  config_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO agents_v3(id, name, harness, status, config_json, created_at, updated_at)
SELECT id, name, harness, status, json_remove(config_json, '$.type'), created_at, updated_at
FROM agents
WHERE profile_type = 'custom';

DROP TABLE agents;
ALTER TABLE agents_v3 RENAME TO agents;

PRAGMA foreign_keys = ON;
