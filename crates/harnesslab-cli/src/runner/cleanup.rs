use harnesslab_core::{BenchmarkPlan, RunSpec};
use harnesslab_infra::{CleanupResult, DockerCliProvider, append_event, event};
use std::path::{Path, PathBuf};

type CleanupFn = fn(&str) -> Result<CleanupResult, String>;

pub(super) struct RunSandboxCleanup {
    run_id: String,
    events_path: PathBuf,
    enabled: bool,
    cleanup_orphans: CleanupFn,
}

impl RunSandboxCleanup {
    pub(super) fn start(run_dir: &Path, spec: &RunSpec, plan: &BenchmarkPlan) -> Self {
        Self::start_with_cleanup(run_dir, spec, plan, docker_cleanup_orphans)
    }

    fn start_with_cleanup(
        run_dir: &Path,
        spec: &RunSpec,
        plan: &BenchmarkPlan,
        cleanup_orphans: CleanupFn,
    ) -> Self {
        let cleanup = Self {
            run_id: spec.run_id.clone(),
            events_path: run_dir.join("events.jsonl"),
            enabled: plan_requires_docker(plan),
            cleanup_orphans,
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
    }
}

fn docker_cleanup_orphans(run_id: &str) -> Result<CleanupResult, String> {
    DockerCliProvider::cleanup_orphans(run_id).map_err(|error| error.to_string())
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
        ArtifactSpec, BenchmarkIdentity, BenchmarkRef, ExecutionConfig, NetworkPolicy,
        ResourceHint, RunConfigOverrides, RunPaths, SandboxSpec, TaskPlan, VerifierEnvironment,
        VerifierSpec, WorkspaceSpec, WorkspaceType,
    };

    #[test]
    fn cleanup_001_plan_requires_docker_only_for_container_tasks() {
        assert!(!plan_requires_docker(&plan_with_image("host")));
        assert!(!plan_requires_docker(&plan_with_image("host-fixture")));
        assert!(plan_requires_docker(&plan_with_image("ubuntu:24.04")));
    }

    #[test]
    fn cleanup_002_docker_plan_writes_pre_and_post_cleanup_events() {
        let run_dir = tempfile::tempdir().unwrap();
        let spec = run_spec(run_dir.path());
        let plan = plan_with_image("ubuntu:24.04");

        {
            let _cleanup =
                RunSandboxCleanup::start_with_cleanup(run_dir.path(), &spec, &plan, ok_cleanup);
        }

        let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
        let records = events
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.lines().count(), 2);
        assert_eq!(records[0]["event"], "docker_cleanup");
        assert_eq!(
            records[0]["message"],
            "docker cleanup pre_run: removed 1 sandbox container(s)"
        );
        assert_eq!(records[1]["event"], "docker_cleanup");
        assert_eq!(
            records[1]["message"],
            "docker cleanup post_run: removed 1 sandbox container(s)"
        );
    }

    #[test]
    fn cleanup_003_non_docker_plan_writes_no_events() {
        let run_dir = tempfile::tempdir().unwrap();
        let spec = run_spec(run_dir.path());
        let plan = plan_with_image("host-fixture");

        {
            let _cleanup =
                RunSandboxCleanup::start_with_cleanup(run_dir.path(), &spec, &plan, panic_cleanup);
        }

        assert!(!run_dir.path().join("events.jsonl").exists());
    }

    #[test]
    fn cleanup_004_cleanup_warning_is_recorded() {
        let run_dir = tempfile::tempdir().unwrap();
        let spec = run_spec(run_dir.path());
        let plan = plan_with_image("ubuntu:24.04");

        {
            let _cleanup = RunSandboxCleanup::start_with_cleanup(
                run_dir.path(),
                &spec,
                &plan,
                warning_cleanup,
            );
        }

        let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
        assert!(events.contains("docker cleanup pre_run warning: cleanup unavailable"));
        assert!(events.contains("docker cleanup post_run warning: cleanup unavailable"));
    }

    fn run_spec(run_dir: &Path) -> RunSpec {
        RunSpec {
            schema_version: 1,
            run_id: "cleanup-test".to_string(),
            created_at: "2026-05-27T00:00:00Z".to_string(),
            agent_profile_ref: "fake".to_string(),
            benchmark: BenchmarkRef {
                name: "fake".to_string(),
                version: "fixture".to_string(),
                split: "smoke".to_string(),
            },
            execution: ExecutionConfig {
                concurrency: 1,
                attempts: 1,
                network: NetworkPolicy::None,
                timeout_sec: None,
            },
            paths: RunPaths {
                run_dir: run_dir.display().to_string(),
            },
            replay_source_run_id: None,
        }
    }

    fn ok_cleanup(run_id: &str) -> Result<CleanupResult, String> {
        assert_eq!(run_id, "cleanup-test");
        Ok(CleanupResult {
            removed: vec!["container-1".to_string()],
        })
    }

    fn panic_cleanup(_run_id: &str) -> Result<CleanupResult, String> {
        panic!("cleanup must not run for host plans")
    }

    fn warning_cleanup(run_id: &str) -> Result<CleanupResult, String> {
        assert_eq!(run_id, "cleanup-test");
        Err("cleanup unavailable".to_string())
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
                external_runner: None,
            }],
            run_config_overrides: RunConfigOverrides {
                timeout_sec: None,
                network: None,
            },
            warnings: Vec::new(),
        }
    }
}
