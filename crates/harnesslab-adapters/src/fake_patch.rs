use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, PatchSpec, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment,
    VerifierSpec, WorkspaceSpec, WorkspaceType,
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

    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
        let task = match split {
            "success" => patch_task(
                "fake-patch-success",
                "Change app.txt content from old to new.",
                "grep -q new app.txt",
            ),
            "no-diff" => patch_task(
                "fake-patch-no-diff",
                "Leave app.txt unchanged.",
                "grep -q new app.txt",
            ),
            "test-fail" => patch_task(
                "fake-patch-test-fail",
                "Change app.txt, but verifier expects impossible content.",
                "grep -q impossible app.txt",
            ),
            _ => return Err(format!("unknown fake-patch split {split}")),
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
