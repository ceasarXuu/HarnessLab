-- Backfill legacy external Dataset registrations to the 'local' registry sentinel.
-- B4 closure: external registrations created before the sentinel fix kept
-- registry_url = NULL, which the WebUI misread as a registry dataset and
-- exposed an in-place sync action. Align them with the 'local' sentinel used
-- by current imports and the mock catalog.
UPDATE webui_datasets
SET registry_url = 'local'
WHERE source = 'local' AND registry_url IS NULL;
