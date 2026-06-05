use anyhow::Result;
use harnesslab_core::{RuntimeTaskSnapshot, TaskPlan, task_dir_name};
use harnesslab_infra::atomic_write_json;
use std::fs;
use std::path::Path;

pub(super) fn write_task_snapshots(
    run_dir: &Path,
    task: &TaskPlan,
    runtime_snapshot: Option<&RuntimeTaskSnapshot>,
) -> Result<()> {
    let task_dir = run_dir.join("tasks").join(task_dir_name(&task.task_id)?);
    fs::create_dir_all(&task_dir)?;
    atomic_write_json(&task_dir.join("task.snapshot.json"), task)?;
    if let Some(snapshot) = runtime_snapshot {
        atomic_write_json(&task_dir.join("task-runtime.snapshot.json"), snapshot)?;
    }
    Ok(())
}
