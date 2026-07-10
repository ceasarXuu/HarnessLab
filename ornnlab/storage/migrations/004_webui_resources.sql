CREATE TABLE webui_operations(
  id TEXT PRIMARY KEY,
  operation_type TEXT NOT NULL,
  status TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT,
  progress INTEGER,
  message TEXT,
  error_code TEXT,
  error_message TEXT,
  error_details_json TEXT,
  created_at TEXT NOT NULL,
  started_at TEXT,
  completed_at TEXT
);

CREATE TABLE webui_agent_configs(
  agent_id TEXT PRIMARY KEY,
  config_json TEXT NOT NULL,
  FOREIGN KEY(agent_id) REFERENCES agents(id)
);

CREATE TABLE webui_environment_profiles(
  id TEXT PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  profile_type TEXT NOT NULL,
  config_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE webui_datasets(
  ref TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  source TEXT NOT NULL,
  visibility TEXT NOT NULL,
  registry_url TEXT,
  local_path TEXT,
  task_count INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE webui_job_configs(
  run_id TEXT PRIMARY KEY,
  config_json TEXT NOT NULL,
  notes TEXT NOT NULL,
  environment_preset_id TEXT NOT NULL,
  FOREIGN KEY(run_id) REFERENCES runs(id)
);
