use harnesslab_core::{BenchmarkPlan, RunSpec};
use harnesslab_infra::{DockerCliProvider, append_event, event};
use std::path::{Path, PathBuf};

pub(super) struct RunSandboxCleanup {
    run_id: String,
    events_path: PathBuf,
    enabled: bool,
}

impl RunSandboxCleanup {
    pub(super) fn start(run_dir: &Path, spec: &RunSpec, plan: &BenchmarkPlan) -> Self {
        let cleanup = Self {
            run_id: spec.run_id.clone(),
            events_path: run_dir.join("events.jsonl"),
            enabled: plan_requires_docker(plan),
        };
        cleanup.cleanup("pre_run");
        cleanup
    }

    fn cleanup(&self, phase: &str) {
        if !self.enabled {
            return;
        }
        let message = match DockerCliProvider::cleanup_orphans(&self.run_id) {
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
    }
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
mod tests {
    use super::*;
    use harnesslab_core::{
        ArtifactSpec, BenchmarkIdentity, ResourceHint, RunConfigOverrides, SandboxSpec, TaskPlan,
        VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
    };

    #[test]
    fn cleanup_001_plan_requires_docker_only_for_container_tasks() {
        assert!(!plan_requires_docker(&plan_with_image("host")));
        assert!(!plan_requires_docker(&plan_with_image("host-fixture")));
        assert!(plan_requires_docker(&plan_with_image("ubuntu:24.04")));
    }

    fn plan_with_image(image: &str) -> BenchmarkPlan {
        BenchmarkPlan {
            benchmark: BenchmarkIdentity {
                name: "fake".to_string(),
                version: "fixture".to_string(),
            },
            split: "smoke".to_string(),
            prepared_benchmark_ref: "fixture".to_string(),
            tasks: vec![TaskPlan {
                task_id: "task".to_string(),
                instruction: "instruction".to_string(),
                workspace_spec: WorkspaceSpec {
                    workspace_type: WorkspaceType::Empty,
                    target_path: "workspace".to_string(),
                    clean: true,
                },
                sandbox_spec: SandboxSpec {
                    image: image.to_string(),
                    mounts: Vec::new(),
                    env_vars: Vec::new(),
                    network: harnesslab_core::NetworkPolicy::None,
                    privileged: false,
                    resource_limits: ResourceHint {
                        cpu_cores: 1,
                        memory_mb: 128,
                    },
                },
                verifier_spec: VerifierSpec {
                    command: "true".to_string(),
                    working_dir: "workspace".to_string(),
                    timeout_sec: 1,
                    expected_exit_codes: vec![0],
                    environment_mode: VerifierEnvironment::HostProcess,
                    output_parser: "exit_code".to_string(),
                },
                artifact_spec: ArtifactSpec {
                    base_dir: "workspace".to_string(),
                    globs: Vec::new(),
                    required_paths: Vec::new(),
                    max_size_bytes: 1,
                },
                patch_spec: None,
            }],
            run_config_overrides: RunConfigOverrides {
                timeout_sec: None,
                network: None,
            },
            warnings: Vec::new(),
        }
    }
}
