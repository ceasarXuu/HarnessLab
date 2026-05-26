use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec,
    WorkspaceSpec, WorkspaceType,
};

pub struct TerminalBenchAdapter;

impl BenchmarkAdapter for TerminalBenchAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        BenchmarkDescriptor {
            name: "terminal-bench".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "2.x".to_string(),
            homepage: "https://www.tbench.ai/".to_string(),
            splits: vec![
                BenchmarkSplit {
                    name: "smoke".to_string(),
                    task_count: 1,
                    data_state: DataState::Ready,
                },
                BenchmarkSplit {
                    name: "full".to_string(),
                    task_count: 0,
                    data_state: DataState::NotDownloaded,
                },
            ],
        }
    }

    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
        match split {
            "smoke" => Ok(plan_from_tasks(
                self.descriptor(),
                split,
                vec![TaskPlan {
                    task_id: "terminal-bench-smoke".to_string(),
                    instruction: "Create result.txt with exactly: terminal-bench-smoke".to_string(),
                    workspace_spec: WorkspaceSpec {
                        workspace_type: WorkspaceType::Empty,
                        target_path: "workspace".to_string(),
                        clean: true,
                    },
                    sandbox_spec: SandboxSpec {
                        image: "ubuntu:24.04".to_string(),
                        mounts: Vec::new(),
                        env_vars: Vec::new(),
                        network: NetworkPolicy::None,
                        privileged: false,
                        resource_limits: ResourceHint {
                            cpu_cores: 1,
                            memory_mb: 512,
                        },
                    },
                    verifier_spec: VerifierSpec {
                        command: "test \"$(cat result.txt 2>/dev/null)\" = terminal-bench-smoke"
                            .to_string(),
                        working_dir: "workspace".to_string(),
                        timeout_sec: 30,
                        expected_exit_codes: vec![0],
                        environment_mode: VerifierEnvironment::HostProcess,
                        output_parser: "exit_code".to_string(),
                    },
                    artifact_spec: ArtifactSpec {
                        base_dir: "workspace".to_string(),
                        globs: vec!["**/*".to_string()],
                        required_paths: Vec::new(),
                        max_size_bytes: 1024 * 1024,
                    },
                    patch_spec: None,
                }],
            )),
            other => Err(format!("unknown terminal-bench split {other}")),
        }
    }
}
