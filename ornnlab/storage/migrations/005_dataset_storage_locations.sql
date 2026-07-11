ALTER TABLE webui_datasets ADD COLUMN storage_kind TEXT NOT NULL DEFAULT 'managed';

UPDATE webui_datasets
SET storage_kind = CASE WHEN source = 'local' THEN 'external' ELSE 'managed' END;

CREATE TABLE webui_dataset_preferences(
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE webui_dataset_downloads(
  ref TEXT PRIMARY KEY,
  destination_path TEXT NOT NULL,
  parent_path TEXT NOT NULL,
  created_at TEXT NOT NULL
);
