use anyhow::{Result, bail};
use harnesslab_core::{BenchmarkPlan, RunSpec, RuntimeTaskSnapshot, task_dir_name};
use harnesslab_infra::read_json;
use std::path::Path;

pub(super) fn replay_spec_from_source(
    source: &RunSpec,
    run_id: String,
    created_at: String,
    run_dir: &Path,
) -> RunSpec {
    let mut spec = source.clone();
    spec.run_id = run_id;
    spec.created_at = created_at;
    spec.paths.run_dir = run_dir.display().to_string();
    spec.replay_source_run_id = Some(source.run_id.clone());
    spec
}

pub(super) fn replay_plan_from_source(source: &Path, spec: &RunSpec) -> Result<BenchmarkPlan> {
    let snapshot = source.join("benchmark.snapshot.json");
    if snapshot.exists() {
        return read_json(&snapshot);
    }
    bail!(
        "replay blocker: benchmark.snapshot.json missing for {}/{}; cannot safely replay without authoritative benchmark snapshot",
        spec.benchmark.name,
        spec.benchmark.split
    )
}

pub(super) fn validate_replay_task_runtime_snapshots(
    source: &Path,
    plan: &BenchmarkPlan,
) -> Result<()> {
    let requires_task_runtime_snapshot = plan.tasks.iter().any(|task| {
        task.external_runner.is_some()
            || plan.task_runtime_snapshots.iter().any(|snapshot| {
                snapshot.task_id == task.task_id && snapshot.external_runner.is_some()
            })
    });
    if !requires_task_runtime_snapshot {
        return Ok(());
    }
    if plan.task_runtime_snapshots.is_empty() {
        bail!(
            "replay blocker: task_runtime_snapshots missing for external benchmark {}/{}; cannot safely replay without task runtime authority",
            plan.benchmark.name,
            plan.split
        );
    }
    for task in &plan.tasks {
        let expected = task_runtime_snapshot_for(plan, &task.task_id)?;
        if task.external_runner != expected.external_runner {
            bail!(
                "replay blocker: task external_runner mismatch for task {}; cannot safely replay external benchmark task with divergent runtime authority",
                task.task_id
            );
        }
        if task.external_runner.is_none() && expected.external_runner.is_none() {
            continue;
        }
        let snapshot_path = source
            .join("tasks")
            .join(task_dir_name(&task.task_id)?)
            .join("task-runtime.snapshot.json");
        if !snapshot_path.exists() {
            bail!(
                "replay blocker: task-runtime.snapshot.json missing for task {}; cannot safely replay external benchmark task without task runtime authority",
                task.task_id
            );
        }
        let actual: RuntimeTaskSnapshot = read_json(&snapshot_path)?;
        if &actual != expected {
            bail!(
                "replay blocker: task-runtime.snapshot.json mismatch for task {}; cannot safely replay external benchmark task with divergent runtime authority",
                task.task_id
            );
        }
    }
    Ok(())
}

fn task_runtime_snapshot_for<'a>(
    plan: &'a BenchmarkPlan,
    task_id: &str,
) -> Result<&'a RuntimeTaskSnapshot> {
    plan.task_runtime_snapshots
        .iter()
        .find(|snapshot| snapshot.task_id == task_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "replay blocker: task runtime snapshot missing for task {task_id}; cannot safely replay external benchmark task without task runtime authority"
            )
        })
}
