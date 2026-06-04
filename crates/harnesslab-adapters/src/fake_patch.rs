use crate::{BenchmarkAdapter, prepared_from_descriptor, stable_checksum};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkSplit, BenchmarkStyle, DataState, NetworkPolicy,
    PatchSpec, PreparedBenchmark, ResourceHint, SandboxSpec, SourceRef, TaskDescriptor, TaskPlan,
    VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
};

pub struct FakePatchAdapter;

impl BenchmarkAdapter for FakePatchAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        BenchmarkDescriptor {
            name: "fake-patch".to_string(),
            style: BenchmarkStyle::Patch,
            version: "fixture".to_string(),
            homepage: "local".to_string(),
            splits: vec![split("success"), split("no-diff"), split("test-fail")],
        }
    }

    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String> {
        if task_id_for_split(split).is_none() {
            return Err(format!("unknown fake-patch split {split}"));
        }
        Ok(prepared_from_descriptor(
            self.descriptor(),
            split,
            format!("fixture://fake-patch/{split}"),
            1,
        ))
    }

    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String> {
        let task_id = task_id_for_split(&prepared.split)
            .ok_or_else(|| format!("unknown fake-patch split {}", prepared.split))?;
        Ok(vec![TaskDescriptor {
            task_id: task_id.to_string(),
            split: prepared.split.clone(),
            estimated_timeout_sec: 5,
            resource_hint: ResourceHint {
                cpu_cores: 1,
                memory_mb: 256,
            },
            source_ref: SourceRef {
                benchmark: "fake-patch".to_string(),
                upstream_id: task_id.to_string(),
                checksum: stable_checksum(&format!("fake-patch:{task_id}")),
            },
        }])
    }

    fn create_task_plan(
        &self,
        _prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String> {
        patch_task_for_id(&task.task_id)
            .ok_or_else(|| format!("unknown fake-patch task {}", task.task_id))
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
        "success" => Some("fake-patch-success"),
        "no-diff" => Some("fake-patch-no-diff"),
        "test-fail" => Some("fake-patch-test-fail"),
        _ => None,
    }
}

fn patch_task_for_id(id: &str) -> Option<TaskPlan> {
    match id {
        "fake-patch-success" => Some(patch_task(
            id,
            "Change app.txt content from old to new.",
            "grep -q new app.txt",
        )),
        "fake-patch-no-diff" => Some(patch_task(
            id,
            "Leave app.txt unchanged.",
            "grep -q new app.txt",
        )),
        "fake-patch-test-fail" => Some(patch_task(
            id,
            "Change app.txt, but verifier expects impossible content.",
            "grep -q impossible app.txt",
        )),
        _ => None,
    }
}

fn patch_task(id: &str, instruction: &str, verifier: &str) -> TaskPlan {
    TaskPlan {
        task_id: id.to_string(),
        instruction: instruction.to_string(),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::GitRepo,
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
            timeout_sec: 5,
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
        external_runner: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_bench_003_fake_patch_plan_has_patch_spec() {
        let plan = FakePatchAdapter.plan("success").unwrap();

        assert!(plan.tasks[0].patch_spec.is_some());
    }

    #[test]
    fn c_bench_003_fake_patch_covers_failure_splits() {
        assert!(FakePatchAdapter.plan("no-diff").is_ok());
        assert!(FakePatchAdapter.plan("test-fail").is_ok());
        assert!(FakePatchAdapter.plan("missing").is_err());
    }
}
