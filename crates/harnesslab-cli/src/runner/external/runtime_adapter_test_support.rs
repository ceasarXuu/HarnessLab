use harnesslab_adapters::built_in_protocol_registry;
use harnesslab_core::{
    AdapterId, ArtifactSpec, BenchmarkRef, ExecutionConfig, ExternalRunnerSpec, NetworkPolicy,
    ResourceHint, RunPaths, RunSpec, SandboxSpec, TaskPlan, TaskRuntimeBinding,
    VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
};
use std::path::Path;

pub(super) const TERMINAL_BENCH_ADAPTER_ID: &str = "harnesslab.terminal-bench.runtime";
pub(super) const SWE_BENCH_PRO_ADAPTER_ID: &str = "harnesslab.swe-bench-pro.runtime";

pub(super) fn run_spec(run_dir: &Path) -> RunSpec {
    RunSpec {
        schema_version: 1,
        run_id: "runtime-preflight-test".to_string(),
        created_at: "2026-06-05T00:00:00Z".to_string(),
        agent_profile_ref: "agent".to_string(),
        benchmark: BenchmarkRef {
            name: "fixture".to_string(),
            version: "1".to_string(),
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

pub(super) fn external_task(task_id: &str, adapter_id: &str) -> TaskPlan {
    let source_path = (adapter_id == SWE_BENCH_PRO_ADAPTER_ID).then_some("source".to_string());
    TaskPlan {
        task_id: task_id.to_string(),
        instruction: "solve".to_string(),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::GitRepo,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: SandboxSpec {
            image: "ubuntu:latest".to_string(),
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
            command: "true".to_string(),
            working_dir: ".".to_string(),
            timeout_sec: 60,
            expected_exit_codes: vec![0],
            environment_mode: VerifierEnvironment::HostProcess,
            output_parser: "exit_code".to_string(),
        },
        artifact_spec: ArtifactSpec {
            base_dir: ".".to_string(),
            globs: Vec::new(),
            required_paths: Vec::new(),
            max_size_bytes: 1024,
        },
        patch_spec: None,
        external_runner: Some(ExternalRunnerSpec {
            dataset_path: "dataset".to_string(),
            source_path,
            agent_timeout_sec: None,
        }),
        runtime_binding: Some(TaskRuntimeBinding {
            authority: registry_authority(adapter_id),
            dataset_ref: "dataset".to_string(),
            task_ref: if adapter_id == SWE_BENCH_PRO_ADAPTER_ID {
                "source".to_string()
            } else {
                task_id.to_string()
            },
            artifact_contract_id: "artifact.basic.v1".to_string(),
            readiness_contract_id: "readiness.basic.v1".to_string(),
        }),
    }
}

pub(super) fn protocol_bound_terminal_task() -> TaskPlan {
    let mut task = external_task("tb-protocol-task", TERMINAL_BENCH_ADAPTER_ID);
    task.external_runner = None;
    task.runtime_binding = Some(TaskRuntimeBinding {
        authority: registry_authority(TERMINAL_BENCH_ADAPTER_ID),
        dataset_ref: "dataset://terminal-bench/smoke".to_string(),
        task_ref: "task://terminal-bench/smoke/tb-protocol-task".to_string(),
        artifact_contract_id: "artifact.basic.v1".to_string(),
        readiness_contract_id: "readiness.basic.v1".to_string(),
    });
    task
}

pub(super) fn registry_authority(adapter_id: &str) -> harnesslab_core::AdapterProtocolAuthority {
    built_in_protocol_registry()
        .binding_for_adapter_id(&AdapterId::new(adapter_id).expect("adapter id is valid"))
        .expect("protocol binding is registered")
        .authority()
}
