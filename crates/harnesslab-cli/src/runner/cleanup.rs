use harnesslab_core::{BenchmarkPlan, ExternalRunnerKind, RunSpec};
use harnesslab_infra::{CleanupResult, DockerCliProvider, append_event, event};
use std::fs;
use std::path::{Path, PathBuf};

type CleanupFn = fn(&str) -> Result<CleanupResult, String>;
type ComposeCleanupFn =
    fn(&Path, &str) -> Result<super::external::terminal_bench_cleanup::RunCleanupResult, String>;

#[derive(Debug, Clone)]
struct TerminalBenchCleanupTarget {
    run_dir: PathBuf,
    scan_run_id: String,
}

pub(super) struct RunSandboxCleanup {
    run_id: String,
    events_path: PathBuf,
    enabled: bool,
    cleanup_orphans: CleanupFn,
    cleanup_compose: ComposeCleanupFn,
    current_terminal_bench_run: Option<TerminalBenchCleanupTarget>,
    stale_terminal_bench_runs: Vec<TerminalBenchCleanupTarget>,
}

impl RunSandboxCleanup {
    pub(super) fn start(run_dir: &Path, spec: &RunSpec, plan: &BenchmarkPlan) -> Self {
        Self::start_with_cleanup(
            run_dir,
            spec,
            plan,
            docker_cleanup_orphans,
            docker_cleanup_compose,
        )
    }

    fn start_with_cleanup(
        run_dir: &Path,
        spec: &RunSpec,
        plan: &BenchmarkPlan,
        cleanup_orphans: CleanupFn,
        cleanup_compose: ComposeCleanupFn,
    ) -> Self {
        let cleanup = Self {
            run_id: spec.run_id.clone(),
            events_path: run_dir.join("events.jsonl"),
            enabled: plan_requires_docker(plan),
            cleanup_orphans,
            cleanup_compose,
            current_terminal_bench_run: plan_uses_terminal_bench(plan).then(|| {
                TerminalBenchCleanupTarget {
                    run_dir: run_dir.to_path_buf(),
                    scan_run_id: spec.run_id.clone(),
                }
            }),
            stale_terminal_bench_runs: stale_terminal_bench_cleanup_targets(run_dir, plan),
        };
        cleanup.cleanup("pre_run");
        cleanup
    }

    fn cleanup(&self, phase: &str) {
        if !self.enabled {
            return;
        }
        let message = match (self.cleanup_orphans)(&self.run_id) {
            Ok(result) => format!(
                "docker cleanup {phase}: removed {} sandbox container(s)",
                result.removed.len()
            ),
            Err(error) => format!("docker cleanup {phase} warning: {error}"),
        };
        let _ = append_event(
            &self.events_path,
            &event(&self.run_id, None, "docker_cleanup", &message),
            &[],
        );
        for target in self.terminal_bench_cleanup_targets_for_phase(phase) {
            let label = target
                .run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown-run");
            let message = match (self.cleanup_compose)(&target.run_dir, &target.scan_run_id) {
                Ok(result) => format!(
                    "terminal-bench docker cleanup {phase}: run={} scan_run_id={} tokens={} projects={} snapshot_projects={} matched_projects={} removed {} compose resource(s)",
                    label,
                    target.scan_run_id,
                    display_list(&result.tokens),
                    display_list(&result.projects),
                    result.snapshot_projects,
                    result.matched_projects,
                    result.removed.len()
                ),
                Err(error) => format!(
                    "terminal-bench docker cleanup {phase} warning: run={} scan_run_id={} error={}",
                    label, target.scan_run_id, error
                ),
            };
            let _ = append_event(
                &self.events_path,
                &event(
                    &self.run_id,
                    None,
                    "terminal_bench_docker_cleanup",
                    &message,
                ),
                &[],
            );
        }
    }

    fn terminal_bench_cleanup_targets_for_phase(
        &self,
        phase: &str,
    ) -> Vec<&TerminalBenchCleanupTarget> {
        let mut targets = Vec::new();
        if phase == "pre_run" {
            targets.extend(self.stale_terminal_bench_runs.iter());
        }
        if let Some(target) = &self.current_terminal_bench_run {
            targets.push(target);
        }
        targets
    }
}

fn docker_cleanup_orphans(run_id: &str) -> Result<CleanupResult, String> {
    DockerCliProvider::cleanup_orphans(run_id).map_err(|error| error.to_string())
}

fn docker_cleanup_compose(
    run_dir: &Path,
    run_id: &str,
) -> Result<super::external::terminal_bench_cleanup::RunCleanupResult, String> {
    super::external::terminal_bench_cleanup::cleanup_run_resources(run_dir, run_id)
        .map_err(|error| error.to_string())
}

impl Drop for RunSandboxCleanup {
    fn drop(&mut self) {
        self.cleanup("post_run");
    }
}

pub(super) fn plan_requires_docker(plan: &BenchmarkPlan) -> bool {
    plan.tasks
        .iter()
        .any(|task| !matches!(task.sandbox_spec.image.as_str(), "host" | "host-fixture"))
}

fn plan_uses_terminal_bench(plan: &BenchmarkPlan) -> bool {
    plan.tasks.iter().any(|task| {
        task.external_runner
            .as_ref()
            .is_some_and(|runner| runner.kind == ExternalRunnerKind::TerminalBench)
    })
}

fn stale_terminal_bench_cleanup_targets(
    run_dir: &Path,
    plan: &BenchmarkPlan,
) -> Vec<TerminalBenchCleanupTarget> {
    if !plan_uses_terminal_bench(plan) {
        return Vec::new();
    }
    let Some(runs_dir) = run_dir.parent() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(runs_dir) else {
        return Vec::new();
    };
    let mut targets = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path == run_dir {
            continue;
        }
        if let Some(target) = terminal_bench_cleanup_target(&path) {
            targets.push(target);
        }
    }
    targets.sort_by(|left, right| left.run_dir.cmp(&right.run_dir));
    targets
}

fn terminal_bench_cleanup_target(run_dir: &Path) -> Option<TerminalBenchCleanupTarget> {
    let file_name = run_dir.file_name()?.to_str()?.to_string();
    let snapshot_exists = run_dir
        .join("terminal-bench-compose-projects.json")
        .is_file();
    if let Some(run_id) = terminal_bench_run_id_from_spec(run_dir) {
        return Some(TerminalBenchCleanupTarget {
            run_dir: run_dir.to_path_buf(),
            scan_run_id: run_id,
        });
    }
    if snapshot_exists || looks_like_terminal_bench_run_dir(&file_name) {
        return Some(TerminalBenchCleanupTarget {
            run_dir: run_dir.to_path_buf(),
            scan_run_id: file_name,
        });
    }
    None
}

fn terminal_bench_run_id_from_spec(run_dir: &Path) -> Option<String> {
    let bytes = fs::read(run_dir.join("run.json")).ok()?;
    let spec = serde_json::from_slice::<RunSpec>(&bytes).ok()?;
    (spec.benchmark.name == "terminal-bench").then_some(spec.run_id)
}

fn looks_like_terminal_bench_run_dir(name: &str) -> bool {
    let Some(timestamp) = name.rsplit('-').next() else {
        return false;
    };
    let timestamp_bytes = timestamp.as_bytes();
    name.contains("-terminal-bench-")
        && timestamp.len() >= 10
        && timestamp.ends_with('Z')
        && timestamp_bytes
            .get(0..8)
            .is_some_and(|date| date.iter().all(u8::is_ascii_digit))
        && timestamp_bytes.get(8).is_some_and(|ch| *ch == b'T')
        && timestamp_bytes
            .get(9..timestamp.len().saturating_sub(1))
            .is_some_and(|tail| !tail.is_empty() && tail.iter().all(u8::is_ascii_digit))
}

fn display_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(",")
    }
}

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;
