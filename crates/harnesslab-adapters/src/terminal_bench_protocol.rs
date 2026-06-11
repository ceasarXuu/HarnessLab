use crate::{
    ProtocolAdapterDescriptor, ProtocolArtifactDeclaration, ProtocolDataLifecycleContract,
    ProtocolFailureMapping, ProtocolOperation, ProtocolReadinessProbe, ProtocolReportMetadata,
    ProtocolReportSection, ProtocolRuntimeLifecycleContract, built_in_protocol_registry,
};
use harnesslab_core::{
    AdapterId, BenchmarkDescriptor, CapabilityId, FailureClass, FailureCode, HealthImpact,
};

pub fn terminal_bench_protocol_descriptor(
    descriptor: BenchmarkDescriptor,
) -> ProtocolAdapterDescriptor {
    let binding = built_in_protocol_registry()
        .binding_for_adapter_id(
            &AdapterId::new("harnesslab.terminal-bench.runtime")
                .expect("terminal-bench protocol adapter id must be valid"),
        )
        .expect("terminal-bench protocol adapter binding must exist")
        .clone();
    ProtocolAdapterDescriptor {
        readiness: readiness_probes(),
        failure_mapping: failure_mappings(),
        binding,
        descriptor,
        data_lifecycle: data_lifecycle(),
        runtime_lifecycle: runtime_lifecycle(),
        artifacts: artifacts(),
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
        execute: operation("execute", "official.runner", "task.plan", "task.result"),
        cleanup: Some(operation(
            "runtime_cleanup",
            "cleanup.verdict_override",
            "run.dir",
            "cleanup.report",
        )),
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
            "official_runner_available",
            "official.runner",
            "preflight",
            "blocking",
            "ready|blocked",
            "official benchmark runner must be available",
            "install or configure the official benchmark runner",
            "official.runner.private",
        ),
        readiness_probe(
            "docker_available",
            "docker.orchestration",
            "preflight",
            "blocking",
            "ready|blocked",
            "Docker runtime must be available for this adapter",
            "start Docker and ensure the benchmark image can be launched",
            "docker.private",
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
        readiness_probe(
            "run_as_policy",
            "run_as.readiness",
            "preflight",
            "blocking",
            "ready|blocked",
            "requested run-as policy must be enforceable",
            "align setup.run_as with adapter-supported host execution policy",
            "run_as.private",
        ),
        readiness_probe(
            "cleanup_policy",
            "cleanup.verdict_override",
            "cleanup",
            "warning",
            "ready|warning",
            "cleanup status may affect final verdict",
            "inspect cleanup report when cleanup overrides a result",
            "cleanup.private",
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
    ]
}

fn artifacts() -> Vec<ProtocolArtifactDeclaration> {
    vec![
        artifact(
            "external_runtime_public",
            "attempt",
            "external-runtime.public.json",
            "runtime_snapshot",
            "public",
            "runtime_snapshot",
            true,
            "structured",
        ),
        artifact(
            "external_runtime_private",
            "attempt",
            "external-runtime.private.json",
            "runtime_snapshot",
            "private",
            "runtime_snapshot",
            true,
            "private_only",
        ),
        artifact(
            "events",
            "run",
            "events.jsonl",
            "event_log",
            "public",
            "runtime",
            false,
            "scan",
        ),
        artifact(
            "result",
            "attempt",
            "result.json",
            "result",
            "public",
            "result_parse",
            true,
            "scan",
        ),
        artifact(
            "official_results",
            "attempt",
            "official/terminal-bench/results.json",
            "result",
            "public",
            "result_parse",
            true,
            "scan",
        ),
        artifact(
            "cleanup_report",
            "attempt",
            "cleanup-report.json",
            "diagnostic_public",
            "public",
            "cleanup",
            false,
            "structured",
        ),
        artifact(
            "agent_stdout",
            "attempt",
            "agent/stdout.log",
            "diagnostic_public",
            "public",
            "execute",
            false,
            "scan",
        ),
        artifact(
            "agent_stderr",
            "attempt",
            "agent/stderr.log",
            "diagnostic_public",
            "public",
            "execute",
            false,
            "scan",
        ),
        artifact(
            "run_health",
            "run",
            "run-health.json",
            "diagnostic_public",
            "public",
            "monitor",
            false,
            "structured",
        ),
        artifact(
            "report",
            "run",
            "report.html",
            "report_public",
            "public",
            "report",
            false,
            "scan",
        ),
        artifact(
            "results",
            "run",
            "results.json",
            "result",
            "public",
            "report",
            true,
            "scan",
        ),
        artifact(
            "command",
            "run",
            "command.txt",
            "diagnostic_public",
            "public",
            "execute",
            false,
            "structured",
        ),
        artifact(
            "agent_profile_runtime",
            "run",
            "agent-profile.runtime.json",
            "diagnostic_public",
            "public",
            "preflight",
            false,
            "structured",
        ),
        artifact(
            "agent_profile_snapshot",
            "run",
            "agent-profile.snapshot.json",
            "diagnostic_public",
            "public",
            "preflight",
            true,
            "structured",
        ),
        artifact(
            "agent_runtime_materialized",
            "run",
            "agent-runtime.materialized.json",
            "diagnostic_public",
            "public",
            "preflight",
            true,
            "structured",
        ),
        artifact(
            "benchmark_snapshot",
            "run",
            "benchmark.snapshot.json",
            "runtime_snapshot",
            "public",
            "replay",
            true,
            "structured",
        ),
        artifact(
            "task_runtime_snapshot",
            "attempt",
            "task-runtime.snapshot.json",
            "runtime_snapshot",
            "public",
            "replay",
            true,
            "structured",
        ),
    ]
}

fn report_metadata() -> ProtocolReportMetadata {
    ProtocolReportMetadata {
        score_fields: vec!["accuracy"],
        public_artifacts: vec![
            "external_runtime_public",
            "events",
            "result",
            "official_results",
            "cleanup_report",
            "report",
            "results",
        ],
        summary_fields: vec!["state", "failure_class", "failure_code", "warnings"],
        detail_sections: vec![ProtocolReportSection {
            section_id: "runtime_diagnostics",
            public_artifact_refs: vec![
                "external_runtime_public",
                "official_results",
                "cleanup_report",
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
        capability: CapabilityId::new(capability).expect("valid terminal-bench capability id"),
        input_contract,
        output_contract,
    }
}

fn artifact(
    artifact_id: &'static str,
    scope: &'static str,
    path: &'static str,
    artifact_type: &'static str,
    visibility: &'static str,
    producer_phase: &'static str,
    required_for_replay: bool,
    redaction_policy: &'static str,
) -> ProtocolArtifactDeclaration {
    ProtocolArtifactDeclaration {
        artifact_id,
        scope,
        path,
        artifact_type,
        visibility,
        producer_phase,
        required_for_replay,
        redaction_policy,
        schema_version: "1",
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
        capability: CapabilityId::new(capability).expect("valid terminal-bench capability id"),
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
