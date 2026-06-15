CREATE TABLE agents(
  id TEXT PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  kind TEXT NOT NULL,
  harbor_agent_name TEXT,
  harbor_import_path TEXT,
  model_name TEXT,
  status TEXT NOT NULL,
  profile_path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE experiments(
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  requested_run_count INTEGER NOT NULL,
  mode TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE runs(
  id TEXT PRIMARY KEY,
  experiment_id TEXT NOT NULL,
  status TEXT NOT NULL,
  run_order INTEGER NOT NULL,
  agent_id TEXT NOT NULL,
  agent_snapshot_hash TEXT NOT NULL,
  benchmark_name TEXT NOT NULL,
  benchmark_version TEXT,
  split TEXT,
  task_filter_hash TEXT,
  n_tasks INTEGER,
  n_attempts INTEGER NOT NULL,
  n_concurrent INTEGER NOT NULL,
  harbor_job_name TEXT,
  harbor_job_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  job_dir TEXT,
  result_path TEXT,
  report_path TEXT,
  failure_class TEXT,
  failure_code TEXT,
  failure_summary TEXT,
  leaderboard_eligible INTEGER NOT NULL,
  comparability_key TEXT,
  FOREIGN KEY(experiment_id) REFERENCES experiments(id)
);

CREATE TABLE queue_items(
  run_id TEXT PRIMARY KEY,
  queue_position INTEGER NOT NULL,
  state TEXT NOT NULL,
  enqueued_at TEXT NOT NULL,
  dequeued_at TEXT,
  finished_at TEXT,
  FOREIGN KEY(run_id) REFERENCES runs(id)
);

CREATE TABLE experiment_events(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  ts TEXT NOT NULL,
  event_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  mirror_file TEXT,
  mirror_offset INTEGER
);

CREATE TABLE templates(
  id TEXT PRIMARY KEY,
  name TEXT UNIQUE NOT NULL,
  config_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
