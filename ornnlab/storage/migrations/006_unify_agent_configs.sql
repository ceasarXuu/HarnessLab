PRAGMA foreign_keys = OFF;

CREATE TABLE agents_v2(
  id TEXT PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  harness TEXT NOT NULL,
  profile_type TEXT NOT NULL CHECK(profile_type IN ('built-in', 'custom')),
  status TEXT NOT NULL CHECK(status IN ('active', 'deleted')),
  config_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO agents_v2(
  id, name, harness, profile_type, status, config_json, created_at, updated_at
)
SELECT
  agents.id,
  agents.name,
  COALESCE(json_extract(webui_agent_configs.config_json, '$.harness'), agents.harbor_agent_name, agents.kind),
  COALESCE(json_extract(webui_agent_configs.config_json, '$.type'), 'custom'),
  CASE WHEN agents.status = 'deleted' THEN 'deleted' ELSE 'active' END,
  COALESCE(
    webui_agent_configs.config_json,
    json_object(
      'id', agents.id,
      'agentName', agents.name,
      'harness', COALESCE(agents.harbor_agent_name, agents.kind),
      'type', 'custom',
      'env', json('[]'),
      'kwargs', '',
      'mcpServers', json('[]'),
      'models', CASE
        WHEN agents.model_name IS NULL THEN json('[]')
        ELSE json_array(agents.model_name)
      END,
      'skillSources', json('[]')
    )
  ),
  agents.created_at,
  agents.updated_at
FROM agents
LEFT JOIN webui_agent_configs ON webui_agent_configs.agent_id = agents.id;

DROP TABLE webui_agent_configs;
DROP TABLE agents;
ALTER TABLE agents_v2 RENAME TO agents;

PRAGMA foreign_keys = ON;
