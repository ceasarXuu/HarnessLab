use crate::{BenchmarkAdapter, prepared_from_descriptor, stable_checksum};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkSplit, BenchmarkStyle, DataState, NetworkPolicy,
    PreparedBenchmark, ResourceHint, SandboxSpec, SourceRef, TaskDescriptor, TaskPlan,
    VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
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

    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String> {
        if task_id_for_split(split).is_none() {
            return Err(format!("unknown fake-terminal split {split}"));
        }
        Ok(prepared_from_descriptor(
            self.descriptor(),
            split,
            format!("fixture://fake-terminal/{split}"),
            1,
        ))
    }

    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String> {
        let task_id = task_id_for_split(&prepared.split)
            .ok_or_else(|| format!("unknown fake-terminal split {}", prepared.split))?;
        Ok(vec![TaskDescriptor {
            task_id: task_id.to_string(),
            split: prepared.split.clone(),
            estimated_timeout_sec: 5,
            resource_hint: ResourceHint {
                cpu_cores: 1,
                memory_mb: 256,
            },
            source_ref: SourceRef {
                benchmark: "fake-terminal".to_string(),
                upstream_id: task_id.to_string(),
                checksum: stable_checksum(&format!("fake-terminal:{task_id}")),
            },
        }])
    }

    fn create_task_plan(
        &self,
        _prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String> {
        terminal_task_for_id(&task.task_id)
            .ok_or_else(|| format!("unknown fake-terminal task {}", task.task_id))
    }
}

fn split(name: &str) -> BenchmarkSplit {
    BenchmarkSplit {
        name: name.to_string(),
        task_count: 1,
        data_state: DataState::Ready,
    }
}

fn task_id_for_split(split: &str) -> Option<&'static str> {
    match split {
        "success" => Some("fake-terminal-success"),
        "test-fail" => Some("fake-terminal-test-fail"),
        "agent-timeout" => Some("fake-terminal-agent-timeout"),
        "agent-crash" => Some("fake-terminal-agent-crash"),
        _ => None,
    }
}

fn terminal_task_for_id(id: &str) -> Option<TaskPlan> {
    match id {
        "fake-terminal-success" => Some(terminal_task(
            id,
            "Create result.txt with exactly: ok",
            "test \"$(cat result.txt 2>/dev/null)\" = ok",
            vec![0],
            5,
        )),
        "fake-terminal-test-fail" => Some(terminal_task(
            id,
            "Create result.txt with exactly: expected-fail",
            "printf 'normal benchmark failure\\n'",
            vec![99],
            5,
        )),
        "fake-terminal-agent-timeout" => Some(terminal_task(
            id,
            "Sleep longer than the timeout.",
            "true",
            vec![0],
            1,
        )),
        "fake-terminal-agent-crash" => {
            Some(terminal_task(id, "Exit non-zero.", "true", vec![0], 5))
        }
        _ => None,
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
        runtime_binding: None,
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
