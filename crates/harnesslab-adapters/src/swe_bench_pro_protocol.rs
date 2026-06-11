use crate::{
    ProtocolAdapterDescriptor, ProtocolDataLifecycleContract, ProtocolFailureMapping,
    ProtocolOperation, ProtocolReadinessProbe, ProtocolReportMetadata, ProtocolReportSection,
    ProtocolRuntimeLifecycleContract, built_in_protocol_registry,
    swe_bench_pro_artifacts::swe_bench_pro_artifacts,
};
use harnesslab_core::{
    AdapterId, BenchmarkDescriptor, CapabilityId, FailureClass, FailureCode, HealthImpact,
};

pub fn swe_bench_pro_protocol_descriptor(
    descriptor: BenchmarkDescriptor,
) -> ProtocolAdapterDescriptor {
    let binding = built_in_protocol_registry()
        .binding_for_adapter_id(
            &AdapterId::new("harnesslab.swe-bench-pro.runtime")
                .expect("swe-bench-pro protocol adapter id must be valid"),
        )
        .expect("swe-bench-pro protocol adapter binding must exist")
        .clone();
    ProtocolAdapterDescriptor {
        readiness: readiness_probes(),
        failure_mapping: failure_mappings(),
        binding,
        descriptor,
        data_lifecycle: data_lifecycle(),
        runtime_lifecycle: runtime_lifecycle(),
        artifacts: swe_bench_pro_artifacts(),
        report_metadata: report_metadata(),
    }
}

fn data_lifecycle() -> ProtocolDataLifecycleContract {
    ProtocolDataLifecycleContract {
        inspect_data: operation(
            "inspect_data",
            "data.lifecycle",
            "benchmark.root",
            "data.state",
        ),
        prepare: operation("prepare", "data.lifecycle", "split", "prepared.benchmark"),
        list_tasks: operation(
            "list_tasks",
            "data.lifecycle",
            "prepared.benchmark",
            "task.list",
        ),
        create_task_plan: operation(
            "create_task_plan",
            "data.lifecycle",
            "task.descriptor",
            "task.plan",
        ),
        snapshot_task: operation(
            "snapshot_task",
            "replay.authority",
            "task.plan",
            "runtime.task.snapshot",
        ),
    }
}

fn runtime_lifecycle() -> ProtocolRuntimeLifecycleContract {
    ProtocolRuntimeLifecycleContract {
        preflight: operation(
            "runtime_preflight",
            "readiness.basic",
            "task.plan",
            "readiness.report",
        ),
        execute: operation("execute", "patch.evaluator", "task.plan", "task.result"),
        cleanup: None,
        snapshot: operation(
            "runtime_snapshot",
            "artifacts.basic",
            "task.result",
            "runtime.snapshot",
        ),
        replay_validate: operation(
            "replay_validate",
            "replay.authority",
            "runtime.snapshot",
            "replay.decision",
        ),
    }
}

fn readiness_probes() -> Vec<ProtocolReadinessProbe> {
    vec![
        readiness_probe(
            "data_ready",
            "data.lifecycle",
            "prepare",
            "blocking",
            "ready|blocked",
            "benchmark data must be prepared before task planning",
            "prepare benchmark data before task planning",
            "data.state",
        ),
        readiness_probe(
            "runtime_preflight",
            "readiness.basic",
            "preflight",
            "blocking",
            "ready|blocked",
            "adapter runtime preflight must pass before execution",
            "fix adapter-specific profile labels or source material before running",
            "runtime.preflight.private",
        ),
        readiness_probe(
            "patch_evaluator_available",
            "patch.evaluator",
            "preflight",
            "blocking",
            "ready|blocked",
            "patch evaluator must be available",
            "install or configure the adapter evaluator",
            "patch.evaluator.private",
        ),
        readiness_probe(
            "host_agent_execution",
            "host.agent_execution",
            "preflight",
            "blocking",
            "ready|blocked",
            "host agent execution policy must allow this adapter",
            "use setup.run_as=current or a sandboxed agent path supported by the adapter",
            "host.agent.private",
        ),
    ]
}

fn failure_mappings() -> Vec<ProtocolFailureMapping> {
    vec![
        failure_mapping(
            "runtime_setup",
            "preflight",
            "setup",
            FailureClass::Benchmark,
            FailureCode::ExternalRunnerSetupFailed,
            HealthImpact::EnvironmentUnhealthy,
            "adapter runtime setup failed before benchmark execution",
            "runtime.setup.private",
        ),
        failure_mapping(
            "runtime_timeout",
            "execute",
            "timeout",
            FailureClass::Execution,
            FailureCode::ExternalRunnerTimeout,
            HealthImpact::Stall,
            "adapter runtime exceeded its configured timeout",
            "runtime.timeout.private",
        ),
        failure_mapping(
            "runtime_no_progress",
            "execute",
            "no_progress",
            FailureClass::Execution,
            FailureCode::ExternalRunnerNoProgress,
            HealthImpact::Stall,
            "adapter runtime stopped producing progress",
            "runtime.no_progress.private",
        ),
        failure_mapping(
            "patch_missing",
            "evaluate",
            "diff_capture",
            FailureClass::Benchmark,
            FailureCode::NoValidDiff,
            HealthImpact::None,
            "adapter did not produce a valid patch",
            "patch.diff.private",
        ),
        failure_mapping(
            "evaluator_error",
            "evaluate",
            "evaluator",
            FailureClass::Benchmark,
            FailureCode::EvaluatorError,
            HealthImpact::EnvironmentUnhealthy,
            "adapter evaluator failed",
            "evaluator.private",
        ),
    ]
}

fn report_metadata() -> ProtocolReportMetadata {
    ProtocolReportMetadata {
        score_fields: vec!["resolved", "patch_applied"],
        public_artifacts: vec![
            "external_runtime_public",
            "events",
            "result",
            "patch",
            "prediction",
            "prediction_eval",
            "evaluator_result",
            "verifier_stdout",
            "verifier_stderr",
        ],
        summary_fields: vec!["state", "failure_class", "failure_code", "patch_status"],
        detail_sections: vec![ProtocolReportSection {
            section_id: "patch_evaluation",
            public_artifact_refs: vec![
                "external_runtime_public",
                "patch",
                "prediction",
                "prediction_eval",
                "evaluator_result",
            ],
        }],
    }
}

fn operation(
    id: &'static str,
    capability: &'static str,
    input_contract: &'static str,
    output_contract: &'static str,
) -> ProtocolOperation {
    ProtocolOperation {
        id,
        capability: CapabilityId::new(capability).expect("valid swe-bench-pro capability id"),
        input_contract,
        output_contract,
    }
}

fn readiness_probe(
    id: &'static str,
    capability: &'static str,
    phase: &'static str,
    severity: &'static str,
    status_contract: &'static str,
    public_message: &'static str,
    remediation: &'static str,
    private_detail_contract: &'static str,
) -> ProtocolReadinessProbe {
    ProtocolReadinessProbe {
        id,
        capability: CapabilityId::new(capability).expect("valid swe-bench-pro capability id"),
        phase,
        severity,
        status_contract,
        public_message,
        remediation,
        private_detail_contract,
    }
}

fn failure_mapping(
    adapter_code: &'static str,
    adapter_phase: &'static str,
    adapter_subphase: &'static str,
    failure_class: FailureClass,
    failure_code: FailureCode,
    health_impact: HealthImpact,
    public_message: &'static str,
    private_diagnostics_contract: &'static str,
) -> ProtocolFailureMapping {
    ProtocolFailureMapping {
        adapter_code,
        adapter_phase,
        adapter_subphase,
        failure_class,
        failure_code,
        health_impact,
        public_message,
        private_diagnostics_contract,
    }
}
