JOB_SELECT = """
    SELECT runs.*, experiments.name AS experiment_name,
           agents.name AS agent_profile_name, webui_job_configs.config_json,
           webui_job_configs.notes AS job_notes
    FROM runs
    JOIN experiments ON experiments.id = runs.experiment_id
    JOIN agents ON agents.id = runs.agent_id
    LEFT JOIN webui_job_configs ON webui_job_configs.run_id = runs.id
"""
