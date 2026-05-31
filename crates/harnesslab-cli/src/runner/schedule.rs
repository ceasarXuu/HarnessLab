use anyhow::Result;
use harnesslab_core::{
    AttemptProvenance, FailureCode, Outcome, TaskAttemptResult, TaskPlan, TaskState, task_dir_name,
};
use harnesslab_infra::read_json;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct AttemptWork {
    pub(super) task: TaskPlan,
    pub(super) attempt: u32,
    pub(super) provenance: AttemptProvenance,
}

pub(super) fn partition_attempts(
    run_dir: &Path,
    plan: &harnesslab_core::BenchmarkPlan,
    attempts: u32,
) -> Result<(Vec<TaskAttemptResult>, Vec<AttemptWork>)> {
    let mut completed = Vec::new();
    let mut pending = Vec::new();
    for task in &plan.tasks {
        let existing = existing_attempts(run_dir, &task.task_id)?
            .into_iter()
            .filter(|result| result.failure_code != Some(FailureCode::RunHealthAborted))
            .collect::<Vec<_>>();
        let next_planned = planned_attempts_for_task(task, attempts)
            .into_iter()
            .filter(|work| !existing.iter().any(|result| result.attempt == work.attempt))
            .collect::<Vec<_>>();
        if next_planned.is_empty() && should_schedule_recovery_attempt(&existing, attempts) {
            pending.push(AttemptWork {
                task: task.clone(),
                attempt: existing
                    .iter()
                    .map(|result| result.attempt)
                    .max()
                    .unwrap_or(0)
                    + 1,
                provenance: AttemptProvenance::Recovery,
            });
        } else {
            pending.extend(next_planned);
        }
        completed.extend(existing);
    }
    Ok((completed, pending))
}

#[cfg(test)]
pub(super) fn planned_attempts(
    plan: &harnesslab_core::BenchmarkPlan,
    attempts: u32,
) -> Vec<AttemptWork> {
    plan.tasks
        .iter()
        .flat_map(|task| planned_attempts_for_task(task, attempts))
        .collect()
}

#[cfg(test)]
pub(super) fn attempt_result_path(
    run_dir: &Path,
    task_id: &str,
    attempt: u32,
) -> std::path::PathBuf {
    let task_dir = task_dir_name(task_id).unwrap_or_else(|_| "_invalid-task-id".to_string());
    run_dir
        .join("tasks")
        .join(task_dir)
        .join("attempts")
        .join(attempt.to_string())
        .join("result.json")
}

fn planned_attempts_for_task(task: &TaskPlan, attempts: u32) -> Vec<AttemptWork> {
    (1..=attempts.max(1))
        .map(|attempt| AttemptWork {
            task: task.clone(),
            attempt,
            provenance: AttemptProvenance::Original,
        })
        .collect()
}

fn existing_attempts(run_dir: &Path, task_id: &str) -> Result<Vec<TaskAttemptResult>> {
    let attempts_dir = run_dir
        .join("tasks")
        .join(task_dir_name(task_id)?)
        .join("attempts");
    if !attempts_dir.exists() {
        return Ok(Vec::new());
    }
    let mut attempts: Vec<TaskAttemptResult> = Vec::new();
    for entry in fs::read_dir(attempts_dir)? {
        let path = entry?.path().join("result.json");
        if path.exists() {
            attempts.push(read_json(&path)?);
        }
    }
    attempts.sort_by_key(|result| result.attempt);
    Ok(attempts)
}

fn should_schedule_recovery_attempt(existing: &[TaskAttemptResult], attempts: u32) -> bool {
    let configured_attempts_consumed = existing.len() == attempts.max(1) as usize;
    if existing.is_empty() || !configured_attempts_consumed {
        return false;
    }
    existing
        .iter()
        .any(|result| result.state == TaskState::Interrupted)
        || existing
            .iter()
            .all(|result| result.outcome == Outcome::Failure)
}
