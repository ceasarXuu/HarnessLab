use anyhow::{Result, anyhow};
use harnesslab_core::{FailureClass, FailureCode, RunSpec, redact_public_value};
use harnesslab_infra::{CleanupResult, DockerCliProvider, append_event, atomic_write_json, event};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const PROJECT_SNAPSHOT: &str = "terminal-bench-compose-projects.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(super) struct ComposeProjectSnapshot {
    schema_version: u32,
    projects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runner) struct RunCleanupResult {
    pub(in crate::runner) removed: Vec<String>,
    pub(in crate::runner) tokens: Vec<String>,
    pub(in crate::runner) projects: Vec<String>,
    pub(in crate::runner) snapshot_projects: usize,
    pub(in crate::runner) matched_projects: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TaskCleanupOutcome {
    pub(super) phase: String,
    pub(super) required: bool,
    pub(super) token: String,
    pub(super) success: bool,
    pub(super) projects: Vec<String>,
    pub(super) removed: Vec<String>,
    pub(super) containers_removed: usize,
    pub(super) networks_removed: usize,
    pub(super) error: Option<String>,
}

pub(super) fn cleanup_task_resources(
    run_dir: &Path,
    spec: &RunSpec,
    task_id: &str,
    phase: &str,
    official_run_id: &str,
    required: bool,
    redaction_refs: &[String],
) -> Result<TaskCleanupOutcome> {
    let projects = match DockerCliProvider::compose_projects_matching(official_run_id) {
        Ok(projects) => projects,
        Err(error) => {
            let outcome = cleanup_error_outcome(
                phase,
                official_run_id,
                required,
                &[],
                error.to_string(),
                redaction_refs,
            );
            append_cleanup_event(
                run_dir,
                spec,
                task_id,
                &format!(
                    "terminal-bench cleanup {phase} warning: projects_count=0 removed_count=0 containers_removed=0 networks_removed=0 has_error=true"
                ),
            )?;
            return if required {
                Err(anyhow!(
                    "terminal-bench cleanup {phase} failed before project discovery"
                ))
            } else {
                Ok(outcome)
            };
        }
    };
    if !projects.is_empty() {
        record_projects(run_dir, &projects)?;
    }
    match DockerCliProvider::cleanup_compose_projects(&projects)
        .map(|result| (projects.clone(), result))
    {
        Ok((projects, result)) => {
            let outcome = cleanup_success_outcome(
                phase,
                official_run_id,
                required,
                &projects,
                &result,
                redaction_refs,
            );
            append_cleanup_event(
                run_dir,
                spec,
                task_id,
                &cleanup_success_message(phase, &outcome),
            )?;
            Ok(outcome)
        }
        Err(error) => {
            let message = error.to_string();
            let outcome = cleanup_error_outcome(
                phase,
                official_run_id,
                required,
                &projects,
                message.clone(),
                redaction_refs,
            );
            append_cleanup_event(
                run_dir,
                spec,
                task_id,
                &format!(
                    "terminal-bench cleanup {phase} warning: projects_count={} removed_count=0 containers_removed=0 networks_removed=0 has_error=true",
                    outcome.projects.len()
                ),
            )?;
            if required {
                Err(anyhow!("terminal-bench cleanup {phase} failed"))
            } else {
                Ok(outcome)
            }
        }
    }
}

pub(super) fn write_task_cleanup_report(
    attempt_dir: &Path,
    task_id: &str,
    attempt: u32,
    _official_run_id: &str,
    pre_task: &TaskCleanupOutcome,
    post_task: &TaskCleanupOutcome,
    official_failure_class: FailureClass,
    official_failure_code: Option<FailureCode>,
    final_failure_class: FailureClass,
    final_failure_code: Option<FailureCode>,
    cleanup_overrides_result: bool,
) -> Result<()> {
    let report = TaskCleanupReport {
        schema_version: 1,
        benchmark: "terminal-bench",
        task_id: task_id.to_string(),
        attempt,
        phases: vec![public_outcome(pre_task), public_outcome(post_task)],
        official_failure: CleanupFailureSnapshot {
            class: official_failure_class,
            code: official_failure_code,
        },
        final_failure: CleanupFailureSnapshot {
            class: final_failure_class,
            code: final_failure_code,
        },
        final_verdict_effect: final_verdict_effect(post_task, cleanup_overrides_result),
    };
    atomic_write_json(&attempt_dir.join("cleanup-report.json"), &report)
}

#[derive(Serialize)]
struct TaskCleanupReport {
    schema_version: u32,
    benchmark: &'static str,
    task_id: String,
    attempt: u32,
    phases: Vec<PublicTaskCleanupOutcome>,
    official_failure: CleanupFailureSnapshot,
    final_failure: CleanupFailureSnapshot,
    final_verdict_effect: &'static str,
}

#[derive(Serialize)]
struct PublicTaskCleanupOutcome {
    phase: String,
    required: bool,
    success: bool,
    projects_count: usize,
    removed_count: usize,
    containers_removed: usize,
    networks_removed: usize,
    has_error: bool,
}

#[derive(Serialize)]
struct CleanupFailureSnapshot {
    class: FailureClass,
    code: Option<FailureCode>,
}

fn public_outcome(outcome: &TaskCleanupOutcome) -> PublicTaskCleanupOutcome {
    PublicTaskCleanupOutcome {
        phase: outcome.phase.clone(),
        required: outcome.required,
        success: outcome.success,
        projects_count: outcome.projects.len(),
        removed_count: outcome.removed.len(),
        containers_removed: outcome.containers_removed,
        networks_removed: outcome.networks_removed,
        has_error: outcome.error.is_some(),
    }
}

pub(in crate::runner) fn cleanup_run_resources(
    run_dir: &Path,
    run_id: &str,
) -> Result<RunCleanupResult> {
    let snapshot_projects = recorded_projects(run_dir)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut projects = snapshot_projects.clone();
    let mut matched_projects = BTreeSet::new();
    let tokens = cleanup_match_tokens(run_id);
    for token in &tokens {
        let matches = DockerCliProvider::compose_projects_matching(token)?;
        matched_projects.extend(matches.iter().cloned());
        projects.extend(matches);
    }
    let projects = projects.into_iter().collect::<Vec<_>>();
    let cleanup = DockerCliProvider::cleanup_compose_projects(&projects)?;
    Ok(RunCleanupResult {
        removed: cleanup.removed,
        tokens,
        projects,
        snapshot_projects: snapshot_projects.len(),
        matched_projects: matched_projects.len(),
    })
}

fn cleanup_success_outcome(
    phase: &str,
    token: &str,
    required: bool,
    projects: &[String],
    result: &CleanupResult,
    redaction_refs: &[String],
) -> TaskCleanupOutcome {
    let secret_refs = secret_refs(redaction_refs);
    let removed = result
        .removed
        .iter()
        .map(|item| redact_public_value(item, &secret_refs))
        .collect::<Vec<_>>();
    TaskCleanupOutcome {
        phase: phase.to_string(),
        required,
        token: token.to_string(),
        success: true,
        projects: projects
            .iter()
            .map(|item| redact_public_value(item, &secret_refs))
            .collect(),
        removed,
        containers_removed: removed_with_prefix(result, "container:"),
        networks_removed: removed_with_prefix(result, "network:"),
        error: None,
    }
}

fn cleanup_error_outcome(
    phase: &str,
    token: &str,
    required: bool,
    projects: &[String],
    error: String,
    redaction_refs: &[String],
) -> TaskCleanupOutcome {
    let secret_refs = secret_refs(redaction_refs);
    TaskCleanupOutcome {
        phase: phase.to_string(),
        required,
        token: token.to_string(),
        success: false,
        projects: projects
            .iter()
            .map(|item| redact_public_value(item, &secret_refs))
            .collect(),
        removed: Vec::new(),
        containers_removed: 0,
        networks_removed: 0,
        error: Some(redact_public_value(&error, &secret_refs)),
    }
}

fn secret_refs(redaction_refs: &[String]) -> Vec<&str> {
    redaction_refs
        .iter()
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .collect()
}

fn final_verdict_effect(
    post_task: &TaskCleanupOutcome,
    cleanup_overrides_result: bool,
) -> &'static str {
    if cleanup_overrides_result {
        "cleanup_overrode_result"
    } else if post_task.error.is_some() {
        "cleanup_warning_only"
    } else {
        "none"
    }
}

fn removed_with_prefix(result: &CleanupResult, prefix: &str) -> usize {
    result
        .removed
        .iter()
        .filter(|item| item.starts_with(prefix))
        .count()
}

fn cleanup_match_tokens(run_id: &str) -> Vec<String> {
    let trimmed = run_id.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut tokens = BTreeSet::new();
    tokens.insert(trimmed.to_string());
    tokens.insert(trimmed.to_ascii_lowercase());
    tokens.insert(terminal_bench_token(trimmed));
    tokens.into_iter().collect()
}

fn terminal_bench_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn append_cleanup_event(
    run_dir: &Path,
    spec: &RunSpec,
    task_id: &str,
    message: &str,
) -> Result<()> {
    append_event(
        &run_dir.join("events.jsonl"),
        &event(
            &spec.run_id,
            Some(task_id),
            "terminal_bench_cleanup",
            message,
        ),
        &[],
    )
}

fn cleanup_success_message(phase: &str, outcome: &TaskCleanupOutcome) -> String {
    format!(
        "terminal-bench cleanup {phase}: projects_count={} removed_count={} containers_removed={} networks_removed={} has_error={}",
        outcome.projects.len(),
        outcome.removed.len(),
        outcome.containers_removed,
        outcome.networks_removed,
        outcome.error.is_some()
    )
}

fn record_projects(run_dir: &Path, projects: &[String]) -> Result<()> {
    let mut all = recorded_projects(run_dir)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    all.extend(projects.iter().cloned());
    let snapshot = ComposeProjectSnapshot {
        schema_version: 1,
        projects: all.into_iter().collect(),
    };
    harnesslab_infra::atomic_write_json(&run_dir.join(PROJECT_SNAPSHOT), &snapshot)
}

fn recorded_projects(run_dir: &Path) -> Result<Vec<String>> {
    let path = run_dir.join(PROJECT_SNAPSHOT);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)?;
    let snapshot: ComposeProjectSnapshot = serde_json::from_slice(&bytes)?;
    Ok(snapshot.projects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_bench_cleanup_message_splits_resource_counts() {
        let result = CleanupResult {
            removed: vec![
                "container:c1".to_string(),
                "network:n1".to_string(),
                "network:n2".to_string(),
            ],
        };

        assert_eq!(
            cleanup_success_message(
                "post_task",
                &cleanup_success_outcome("post_task", "run-task-1", false, &[], &result, &[])
            ),
            "terminal-bench cleanup post_task: projects_count=0 removed_count=3 containers_removed=1 networks_removed=2 has_error=false"
        );
    }

    #[test]
    fn terminal_bench_cleanup_message_makes_zero_removal_explicit() {
        assert_eq!(
            cleanup_success_message(
                "pre_task",
                &cleanup_success_outcome(
                    "pre_task",
                    "run-task-1",
                    true,
                    &["project-1".to_string()],
                    &CleanupResult {
                        removed: Vec::new()
                    },
                    &[]
                ),
            ),
            "terminal-bench cleanup pre_task: projects_count=1 removed_count=0 containers_removed=0 networks_removed=0 has_error=false"
        );
    }

    #[test]
    fn terminal_bench_project_snapshot_merges_projects() {
        let tmp = tempfile::tempdir().unwrap();

        record_projects(tmp.path(), &["b".to_string(), "a".to_string()]).unwrap();
        record_projects(tmp.path(), &["b".to_string(), "c".to_string()]).unwrap();

        assert_eq!(recorded_projects(tmp.path()).unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn terminal_bench_cleanup_match_tokens_include_lowercase_run_id() {
        assert_eq!(
            cleanup_match_tokens("Agent-Terminal-20260602T032823Z"),
            vec![
                "Agent-Terminal-20260602T032823Z".to_string(),
                "agent-terminal-20260602t032823z".to_string()
            ]
        );
        assert!(cleanup_match_tokens(" ").is_empty());
    }

    #[test]
    fn terminal_bench_cleanup_match_tokens_include_official_normalized_token() {
        assert_eq!(
            cleanup_match_tokens("Agent.Terminal_Bench-20260602T032823Z"),
            vec![
                "Agent.Terminal_Bench-20260602T032823Z".to_string(),
                "agent-terminal-bench-20260602t032823z".to_string(),
                "agent.terminal_bench-20260602t032823z".to_string(),
            ]
        );
    }
}
