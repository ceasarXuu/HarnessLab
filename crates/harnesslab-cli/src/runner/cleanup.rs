use harnesslab_core::{BenchmarkPlan, RunSpec};
use harnesslab_infra::{CleanupResult, DockerCliProvider, append_event, event};
use std::path::{Path, PathBuf};

type CleanupFn = fn(&str) -> Result<CleanupResult, String>;
type RuntimeCleanupFn = fn(
    &super::external::RuntimeCleanupTarget,
) -> Result<super::external::RuntimeCleanupReport, String>;

pub(super) struct RunSandboxCleanup {
    run_id: String,
    events_path: PathBuf,
    enabled: bool,
    cleanup_orphans: CleanupFn,
    cleanup_runtime: RuntimeCleanupFn,
    pre_run_runtime_targets: Vec<super::external::RuntimeCleanupTarget>,
    post_run_runtime_targets: Vec<super::external::RuntimeCleanupTarget>,
}

impl RunSandboxCleanup {
    pub(super) fn start(run_dir: &Path, spec: &RunSpec, plan: &BenchmarkPlan) -> Self {
        Self::start_with_cleanup(
            run_dir,
            spec,
            plan,
            docker_cleanup_orphans,
            cleanup_runtime_resources,
        )
    }

    fn start_with_cleanup(
        run_dir: &Path,
        spec: &RunSpec,
        plan: &BenchmarkPlan,
        cleanup_orphans: CleanupFn,
        cleanup_runtime: RuntimeCleanupFn,
    ) -> Self {
        let cleanup = Self {
            run_id: spec.run_id.clone(),
            events_path: run_dir.join("events.jsonl"),
            enabled: plan_requires_docker(plan),
            cleanup_orphans,
            cleanup_runtime,
            pre_run_runtime_targets: super::external::runtime_cleanup_targets_for_phase(
                run_dir,
                spec,
                plan,
                super::external::RuntimeCleanupPhase::PreRun,
            ),
            post_run_runtime_targets: super::external::runtime_cleanup_targets_for_phase(
                run_dir,
                spec,
                plan,
                super::external::RuntimeCleanupPhase::PostRun,
            ),
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
                "docker cleanup {phase}: removed_count={} has_error=false",
                result.removed.len()
            ),
            Err(_) => format!("docker cleanup {phase}: removed_count=0 has_error=true"),
        };
        let _ = append_event(
            &self.events_path,
            &event(&self.run_id, None, "docker_cleanup", &message),
            &[],
        );
        for target in self.runtime_cleanup_targets_for_phase(phase) {
            let message = match (self.cleanup_runtime)(target) {
                Ok(result) => format!(
                    "{} {phase}: runner_kind={:?} tokens_count={} projects_count={} snapshot_projects={} matched_projects={} removed_count={} has_error=false",
                    target.message_prefix,
                    target.runner_kind,
                    result.tokens.len(),
                    result.projects.len(),
                    result.snapshot_projects,
                    result.matched_projects,
                    result.removed.len(),
                ),
                Err(_) => format!(
                    "{} {phase} warning: runner_kind={:?} has_error=true",
                    target.message_prefix, target.runner_kind
                ),
            };
            let _ = append_event(
                &self.events_path,
                &event(&self.run_id, None, target.event_name, &message),
                &[],
            );
        }
    }

    fn runtime_cleanup_targets_for_phase(
        &self,
        phase: &str,
    ) -> &[super::external::RuntimeCleanupTarget] {
        match phase {
            "pre_run" => &self.pre_run_runtime_targets,
            _ => &self.post_run_runtime_targets,
        }
    }
}

fn docker_cleanup_orphans(run_id: &str) -> Result<CleanupResult, String> {
    DockerCliProvider::cleanup_orphans(run_id).map_err(|error| error.to_string())
}

fn cleanup_runtime_resources(
    target: &super::external::RuntimeCleanupTarget,
) -> Result<super::external::RuntimeCleanupReport, String> {
    super::external::cleanup_runtime_resources(target)
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

#[cfg(test)]
#[path = "cleanup_tests.rs"]
mod tests;
