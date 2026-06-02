use anyhow::{Context, Result};
use harnesslab_core::RunSpec;
use harnesslab_infra::{CleanupResult, DockerCliProvider, append_event, event};
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

pub(super) fn cleanup_task_resources(
    run_dir: &Path,
    spec: &RunSpec,
    task_id: &str,
    phase: &str,
    official_run_id: &str,
    required: bool,
) -> Result<()> {
    match DockerCliProvider::compose_projects_matching(official_run_id).and_then(|projects| {
        if !projects.is_empty() {
            record_projects(run_dir, &projects)?;
        }
        DockerCliProvider::cleanup_compose_projects(&projects).map(|result| (projects, result))
    }) {
        Ok((projects, result)) => {
            append_cleanup_event(
                run_dir,
                spec,
                task_id,
                &cleanup_success_message(phase, official_run_id, &projects, &result),
            )?;
            Ok(())
        }
        Err(error) => {
            append_cleanup_event(
                run_dir,
                spec,
                task_id,
                &format!(
                    "terminal-bench cleanup {phase} warning: token={} error={}",
                    official_run_id, error
                ),
            )?;
            if required {
                Err(error).with_context(|| {
                    format!("terminal-bench cleanup {phase} failed for token {official_run_id}")
                })
            } else {
                Ok(())
            }
        }
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

fn cleanup_success_message(
    phase: &str,
    token: &str,
    projects: &[String],
    result: &CleanupResult,
) -> String {
    let containers = result
        .removed
        .iter()
        .filter(|item| item.starts_with("container:"))
        .count();
    let networks = result
        .removed
        .iter()
        .filter(|item| item.starts_with("network:"))
        .count();
    let details = if result.removed.is_empty() {
        "none".to_string()
    } else {
        result.removed.join(",")
    };
    format!(
        "terminal-bench cleanup {phase}: token={token} projects={} removed containers={containers} networks={networks} resources={details}",
        if projects.is_empty() {
            "none".to_string()
        } else {
            projects.join(",")
        }
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
            cleanup_success_message("post_task", "run-task-1", &[], &result),
            "terminal-bench cleanup post_task: token=run-task-1 projects=none removed containers=1 networks=2 resources=container:c1,network:n1,network:n2"
        );
    }

    #[test]
    fn terminal_bench_cleanup_message_makes_zero_removal_explicit() {
        assert_eq!(
            cleanup_success_message(
                "pre_task",
                "run-task-1",
                &["project-1".to_string()],
                &CleanupResult {
                    removed: Vec::new()
                },
            ),
            "terminal-bench cleanup pre_task: token=run-task-1 projects=project-1 removed containers=0 networks=0 resources=none"
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
