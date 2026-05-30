use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec,
    WorkspaceSpec, WorkspaceType,
};

pub struct FakeTerminalAdapter;

impl BenchmarkAdapter for FakeTerminalAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        BenchmarkDescriptor {
            name: "fake-terminal".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "fixture".to_string(),
            homepage: "local".to_string(),
            splits: vec![
                split("success"),
                split("test-fail"),
                split("agent-timeout"),
                split("agent-crash"),
            ],
        }
    }

    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
        let task = match split {
            "success" => terminal_task(
                "fake-terminal-success",
                "Create result.txt with exactly: ok",
                "test \"$(cat result.txt 2>/dev/null)\" = ok",
                vec![0],
                5,
            ),
            "test-fail" => terminal_task(
                "fake-terminal-test-fail",
                "Create result.txt with exactly: expected-fail",
                "printf 'normal benchmark failure\\n'",
                vec![99],
                5,
            ),
            "agent-timeout" => terminal_task(
                "fake-terminal-agent-timeout",
                "Sleep longer than the timeout.",
                "true",
                vec![0],
                1,
            ),
            "agent-crash" => terminal_task(
                "fake-terminal-agent-crash",
                "Exit non-zero.",
                "true",
                vec![0],
                5,
            ),
            _ => return Err(format!("unknown fake-terminal split {split}")),
        };
        Ok(plan_from_tasks(self.descriptor(), split, vec![task]))
    }
}

fn split(name: &str) -> BenchmarkSplit {
    BenchmarkSplit {
        name: name.to_string(),
        task_count: 1,
        data_state: DataState::Ready,
    }
}

fn terminal_task(
    id: &str,
    instruction: &str,
    verifier: &str,
    expected_exit_codes: Vec<i32>,
    timeout_sec: u64,
) -> TaskPlan {
    TaskPlan {
        task_id: id.to_string(),
        instruction: instruction.to_string(),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::Empty,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: SandboxSpec {
            image: "host-fixture".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::None,
            privileged: false,
            resource_limits: ResourceHint {
                cpu_cores: 1,
                memory_mb: 256,
            },
        },
        verifier_spec: VerifierSpec {
            command: verifier.to_string(),
            working_dir: "workspace".to_string(),
            timeout_sec,
            expected_exit_codes,
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
        external_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_bench_002_fake_terminal_task_plan_is_serializable() {
        let plan = FakeTerminalAdapter.plan("success").unwrap();

        let json = serde_json::to_string(&plan.tasks[0]).unwrap();

        assert!(json.contains("fake-terminal-success"));
    }

    #[test]
    fn c_bench_002_fake_terminal_covers_error_splits() {
        assert!(FakeTerminalAdapter.plan("agent-crash").is_ok());
        assert!(FakeTerminalAdapter.plan("missing").is_err());
    }
}
