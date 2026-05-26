use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, PatchSpec, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment,
    VerifierSpec, WorkspaceSpec, WorkspaceType,
};

pub struct SweBenchProAdapter;

impl BenchmarkAdapter for SweBenchProAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        BenchmarkDescriptor {
            name: "swe-bench-pro".to_string(),
            style: BenchmarkStyle::Patch,
            version: "2026".to_string(),
            homepage: "https://www.swebench.com/".to_string(),
            splits: vec![
                BenchmarkSplit {
                    name: "smoke".to_string(),
                    task_count: 1,
                    data_state: DataState::Ready,
                },
                BenchmarkSplit {
                    name: "full".to_string(),
                    task_count: 0,
                    data_state: DataState::RequiresAuth,
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
                    task_id: "swe-bench-pro-smoke".to_string(),
                    instruction: "Modify app.txt from old to swe-bench-pro-smoke.".to_string(),
                    workspace_spec: WorkspaceSpec {
                        workspace_type: WorkspaceType::GitRepo,
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
                        command: "grep -q swe-bench-pro-smoke app.txt".to_string(),
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
                    patch_spec: Some(PatchSpec {
                        diff_path: "patch.diff".to_string(),
                        prediction_path: "prediction.jsonl".to_string(),
                    }),
                }],
            )),
            other => Err(format!("unknown swe-bench-pro split {other}")),
        }
    }
}
