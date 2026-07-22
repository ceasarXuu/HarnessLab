UPDATE experiment_events
SET payload_json = json_remove(payload_json, '$.config')
WHERE event_type = 'harbor.job.running'
  AND json_valid(payload_json)
  AND json_type(payload_json, '$.config') IS NOT NULL;
