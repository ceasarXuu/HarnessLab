use super::*;
#[path = "runtime_adapter_test_support.rs"]
mod support;
use harnesslab_core::{
    AgentKind, AttemptProvenance, BenchmarkId, BenchmarkIdentity, BenchmarkPlan,
    ExternalRunnerSpec, RunConfigOverrides, RuntimePreflightReport, RuntimeTaskSnapshot, SourceRef,
    TaskPlan, TaskRuntimeBinding, default_agent_profile,
};
use support::{
    SWE_BENCH_PRO_ADAPTER_ID, TERMINAL_BENCH_ADAPTER_ID, external_task,
    protocol_bound_terminal_task, registry_authority, run_spec,
};

#[test]
fn adapt_runtime_001_external_entrypoints_delegate_to_runtime_registry() {
    let external_entrypoint = include_str!("../external.rs");
    let runtime_adapter_entrypoint = include_str!("runtime_adapter.rs");
    let runner_entrypoint = include_str!("../../runner.rs");
    let cleanup_entrypoint = include_str!("../cleanup.rs");

    assert!(!runner_entrypoint.contains("runtime_adapter_for_adapter_id(\"terminal-bench"));
    assert!(!runner_entrypoint.contains("runtime_adapter_for_adapter_id(\"swe-bench-pro"));
    assert!(!cleanup_entrypoint.contains("runtime_adapter_for_adapter_id(\"terminal-bench"));
    assert!(!cleanup_entrypoint.contains("runtime_adapter_for_adapter_id(\"swe-bench-pro"));
    assert!(external_entrypoint.contains("preflight_external_task(profile, task)?"));
    assert!(runtime_adapter_entrypoint.contains("runtime_adapter_for_adapter_id"));
    assert!(external_entrypoint.contains("runtime_adapter_for_task(ctx.task)?"));
    assert!(external_entrypoint.contains("runtime_snapshot_source_ref("));
    assert!(external_entrypoint.contains("adapter.execute(&ctx)"));
    assert!(!external_entrypoint.contains("runtime_adapter_for_adapter_id(\"terminal-bench"));
    assert!(!cleanup_entrypoint.contains("terminal_bench"));
    assert!(!cleanup_entrypoint.contains("compose_cleanup"));
    assert!(!external_entrypoint.contains("terminal_bench::execute"));
    assert!(!external_entrypoint.contains("swe_bench_pro::execute"));
    assert!(!external_entrypoint.contains("terminal_bench::validate_profile"));
    assert_runtime_label_access_is_allowlisted();

    let mut terminal_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    terminal_profile.labels.insert(
        "terminal_bench_agent".to_string(),
        "claude-code".to_string(),
    );
    let protocol_report =
        preflight_external_task(&terminal_profile, &protocol_bound_terminal_task()).unwrap();
    assert_eq!(protocol_report.adapter_id, TERMINAL_BENCH_ADAPTER_ID);
    assert_eq!(
        protocol_report.protocol_adapter_id.as_deref(),
        Some("harnesslab.terminal-bench.runtime")
    );
    assert_eq!(protocol_report.protocol_version.as_deref(), Some("1"));
    assert!(!protocol_report.legacy_shim_used);

    let run_dir = tempfile::tempdir().unwrap();
    let attempt_dir = run_dir.path().join("tasks/tb-protocol-task/attempts/1");
    std::fs::create_dir_all(&attempt_dir).unwrap();
    let spec = run_spec(run_dir.path());
    let profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    let materialized = crate::agent_registry::materialize_profile(&profile).unwrap();
    let protocol_task = protocol_bound_terminal_task();
    write_runtime_authority(run_dir.path(), &protocol_task);
    let result = super::super::execute_external_task(super::super::ExternalTaskExecution {
        run_dir: run_dir.path(),
        spec: &spec,
        profile: &profile,
        report_profile: &profile,
        materialized_profile: &materialized,
        report_materialized_profile: &materialized,
        task: &protocol_task,
        attempt: 1,
        provenance: AttemptProvenance::Original,
        attempt_dir: &attempt_dir,
        started: std::time::Instant::now(),
    })
    .expect("protocol-bound task reaches runtime adapter execution");
    assert_eq!(
        result.failure_code,
        Some(harnesslab_core::FailureCode::ExternalRunnerSetupFailed)
    );

    let mut mismatched_task = protocol_bound_terminal_task();
    mismatched_task
        .runtime_binding
        .as_mut()
        .unwrap()
        .authority
        .benchmark_id = BenchmarkId::new("swe-bench-pro").unwrap();
    let error = preflight_external_task(&terminal_profile, &mismatched_task)
        .unwrap_err()
        .to_string();
    assert!(error.contains("invalid protocol runtime binding"));
    assert!(error.contains("protocol_authority_mismatch"));

    let mut mismatched_refs = protocol_bound_terminal_task();
    mismatched_refs.external_runner = Some(ExternalRunnerSpec {
        dataset_path: "legacy-dataset".to_string(),
        source_path: None,
        agent_timeout_sec: None,
    });
    let error = super::super::runtime_dataset_ref(&mismatched_refs)
        .unwrap_err()
        .to_string();
    assert!(error.contains("dataset_ref mismatch"));

    let mut missing_source_ref = external_task("swe-protocol-task", SWE_BENCH_PRO_ADAPTER_ID);
    missing_source_ref.runtime_binding = Some(TaskRuntimeBinding {
        authority: registry_authority("harnesslab.swe-bench-pro.runtime"),
        dataset_ref: "dataset".to_string(),
        task_ref: "source".to_string(),
        artifact_contract_id: "artifact.basic.v1".to_string(),
        readiness_contract_id: "readiness.basic.v1".to_string(),
    });
    missing_source_ref
        .external_runner
        .as_mut()
        .unwrap()
        .source_path = Some("legacy-source".to_string());
    let error = super::super::runtime_source_ref(&missing_source_ref)
        .unwrap_err()
        .to_string();
    assert!(error.contains("task_ref mismatch"));
}

#[test]
fn adapt_runtime_002_preflight_reports_and_enforces_current_compatibility() {
    let mut terminal_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    terminal_profile.labels.insert(
        "terminal_bench_agent_import_path".to_string(),
        "bench_agents.fake:Agent".to_string(),
    );
    let terminal_report = preflight_external_task(
        &terminal_profile,
        &external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID),
    )
    .unwrap();
    assert_preflight(
        terminal_report,
        TERMINAL_BENCH_ADAPTER_ID,
        Some("terminal-bench import agent host path"),
    );

    let mut terminal_sandbox_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    terminal_sandbox_profile.labels.insert(
        "terminal_bench_agent".to_string(),
        "claude-code".to_string(),
    );
    let terminal_sandbox_report = preflight_external_task(
        &terminal_sandbox_profile,
        &external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID),
    )
    .unwrap();
    assert_preflight(terminal_sandbox_report, TERMINAL_BENCH_ADAPTER_ID, None);

    let invalid_terminal_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    let terminal_blocked_report = preflight_external_task(
        &invalid_terminal_profile,
        &external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID),
    )
    .unwrap();
    assert_blocked_preflight(
        terminal_blocked_report,
        TERMINAL_BENCH_ADAPTER_ID,
        "terminal_bench_agent",
    );
    let terminal_blocked_error = super::super::validate_profile_for_plan(
        &invalid_terminal_profile,
        &[external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID)],
    )
    .unwrap_err()
    .to_string();
    assert!(terminal_blocked_error.contains("runtime preflight blocked"));
    assert!(terminal_blocked_error.contains("harnesslab.terminal-bench.runtime"));
    assert!(terminal_blocked_error.contains("task=tb-task"));
    assert!(terminal_blocked_error.contains("adapter_phase=preflight"));
    assert!(terminal_blocked_error.contains("readiness_status=blocked"));
    assert!(terminal_blocked_error.contains("blocking_reason="));
    assert!(terminal_blocked_error.contains("terminal_bench_agent"));
    assert!(terminal_blocked_error.contains("remediation="));

    let mut swe_profile = default_agent_profile("swe", AgentKind::Custom, "agent");
    swe_profile
        .labels
        .insert("swe_bench_pro_agent".to_string(), "gold".to_string());
    let swe_report = preflight_external_task(
        &swe_profile,
        &external_task("swe-task", SWE_BENCH_PRO_ADAPTER_ID),
    )
    .unwrap();
    assert_preflight(
        swe_report,
        SWE_BENCH_PRO_ADAPTER_ID,
        Some("swe-bench-pro gold host path"),
    );

    let swe_sandbox_profile = default_agent_profile("swe", AgentKind::Custom, "agent");
    let swe_sandbox_report = preflight_external_task(
        &swe_sandbox_profile,
        &external_task("swe-task", SWE_BENCH_PRO_ADAPTER_ID),
    )
    .unwrap();
    assert_preflight(swe_sandbox_report, SWE_BENCH_PRO_ADAPTER_ID, None);

    let mut swe_missing_source = external_task("swe-task", SWE_BENCH_PRO_ADAPTER_ID);
    swe_missing_source
        .external_runner
        .as_mut()
        .unwrap()
        .source_path = None;
    let swe_missing_source_report =
        preflight_external_task(&swe_sandbox_profile, &swe_missing_source).unwrap();
    assert_blocked_preflight(
        swe_missing_source_report,
        SWE_BENCH_PRO_ADAPTER_ID,
        "source_path",
    );

    let error = super::super::validate_profile_for_plan(
        &terminal_profile,
        &[external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID)],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("setup.run_as"));
    assert!(error.contains("terminal-bench import agent host path"));
    assert!(error.contains("tb-task"));

    let run_dir = tempfile::tempdir().unwrap();
    let spec = run_spec(run_dir.path());
    super::super::emit_runtime_preflight_reports(
        run_dir.path(),
        &spec,
        &terminal_sandbox_profile,
        &[external_task("tb-task", TERMINAL_BENCH_ADAPTER_ID)],
    )
    .unwrap();
    let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
    assert!(events.contains("\"event\":\"external_runner_preflight\""));
    assert!(events.contains("adapter_id=harnesslab.terminal-bench.runtime"));
    assert!(events.contains("adapter_phase=preflight"));
    assert!(events.contains("agent_bridge_mode=terminal-bench-official-agent"));
    assert!(events.contains("readiness_status=ready"));
    assert!(events.contains("host_execution_reason=none"));
    assert!(events.contains("blocking_reason=none"));
    assert!(events.contains("compatibility_exception=none"));
    assert!(events.contains("compatibility_label_keys=terminal_bench_agent"));
}

#[test]
fn adapt_runtime_002_swe_source_path_failure_snapshot_is_phase_accurate() {
    let run_dir = tempfile::tempdir().unwrap();
    let attempt_dir = run_dir.path().join("tasks/swe-task/attempts/1");
    std::fs::create_dir_all(&attempt_dir).unwrap();
    let spec = run_spec(run_dir.path());
    let profile = default_agent_profile("agent", AgentKind::Fake, "true");
    let materialized = crate::agent_registry::materialize_profile(&profile).unwrap();
    let mut task = external_task("swe-task", SWE_BENCH_PRO_ADAPTER_ID);
    let dataset_dir = run_dir.path().join("dataset");
    std::fs::create_dir_all(&dataset_dir).unwrap();
    let runner = task.external_runner.as_mut().unwrap();
    runner.dataset_path = dataset_dir.display().to_string();
    runner.source_path = None;
    task.runtime_binding = None;
    write_runtime_authority(run_dir.path(), &task);
    let ctx = super::super::ExternalTaskExecution {
        run_dir: run_dir.path(),
        spec: &spec,
        profile: &profile,
        report_profile: &profile,
        materialized_profile: &materialized,
        report_materialized_profile: &materialized,
        task: &task,
        attempt: 1,
        provenance: AttemptProvenance::Original,
        attempt_dir: &attempt_dir,
        started: std::time::Instant::now(),
    };

    let result = runtime_adapter_for_adapter_id(SWE_BENCH_PRO_ADAPTER_ID)
        .unwrap()
        .execute(&ctx)
        .expect("source path failure is structured");

    assert_eq!(
        result.failure_code,
        Some(harnesslab_core::FailureCode::ExternalRunnerSetupFailed)
    );
    let public: serde_json::Value = serde_json::from_slice(
        &std::fs::read(attempt_dir.join("external-runtime.public.json")).unwrap(),
    )
    .unwrap();
    let phases = public["commands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|command| command["phase"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(phases, vec!["source_path_validation"]);
    let public_artifacts = public["public_artifacts"].as_array().unwrap();
    for unexpected in [
        "swe-bench-pro/raw_sample.jsonl",
        "swe-bench-pro/instance.json",
        "swe-bench-pro/workspace-manifest.json",
        "prediction.jsonl",
        "patch.diff",
    ] {
        assert!(
            !public_artifacts
                .iter()
                .any(|artifact| artifact.as_str() == Some(unexpected)),
            "source_path failure should not advertise {unexpected}"
        );
    }
}

fn write_runtime_authority(run_dir: &std::path::Path, task: &TaskPlan) {
    let benchmark = BenchmarkIdentity {
        name: "swe-bench-pro".to_string(),
        version: "fixture".to_string(),
    };
    let runtime_snapshot = RuntimeTaskSnapshot {
        benchmark: benchmark.clone(),
        split: "smoke".to_string(),
        task_id: task.task_id.clone(),
        source_ref: SourceRef {
            benchmark: "swe-bench-pro".to_string(),
            upstream_id: task.task_id.clone(),
            checksum: "fixture".to_string(),
        },
        upstream_metadata_hash: "fixture".to_string(),
        instruction_hash: "fixture".to_string(),
        task_plan_hash: "fixture".to_string(),
        external_runner: task.external_runner.clone(),
        runtime_binding: task.runtime_binding.clone(),
        external_runtime_attempts: Vec::new(),
    };
    let task_dir = run_dir.join("tasks").join(&task.task_id);
    std::fs::create_dir_all(&task_dir).unwrap();
    harnesslab_infra::atomic_write_json(
        &task_dir.join("task-runtime.snapshot.json"),
        &runtime_snapshot,
    )
    .unwrap();
    harnesslab_infra::atomic_write_json(
        &run_dir.join("benchmark.snapshot.json"),
        &BenchmarkPlan {
            benchmark,
            split: "smoke".to_string(),
            prepared_benchmark_ref: "fixture".to_string(),
            tasks: vec![task.clone()],
            task_runtime_snapshots: vec![runtime_snapshot],
            run_config_overrides: RunConfigOverrides {
                timeout_sec: None,
                network: None,
            },
            warnings: Vec::new(),
        },
    )
    .unwrap();
}

fn assert_preflight(
    report: RuntimePreflightReport,
    adapter_id: &str,
    host_execution_reason: Option<&str>,
) {
    assert!(!report.task_id.is_empty());
    assert_eq!(report.adapter_id, adapter_id);
    assert!(!report.adapter_id.is_empty());
    assert!(!report.agent_bridge_mode.is_empty());
    assert_eq!(report.readiness_status, "ready");
    assert_eq!(report.blocking_reason, None);
    assert_eq!(
        report.host_execution_reason.as_deref(),
        host_execution_reason
    );
    if host_execution_reason.is_some() {
        assert_eq!(
            report.compatibility_exception.as_deref(),
            Some("host-agent-run-as-current-only")
        );
    }
    for key in &report.compatibility_label_keys {
        assert!(
            crate::runtime_compatibility::BENCHMARK_RUNTIME_LABEL_ALLOWLIST.contains(&key.as_str())
        );
    }
}

fn assert_blocked_preflight(
    report: RuntimePreflightReport,
    adapter_id: &str,
    blocking_reason: &str,
) {
    assert!(!report.task_id.is_empty());
    assert_eq!(report.adapter_id, adapter_id);
    assert_eq!(report.readiness_status, "blocked");
    assert!(report.host_execution_reason.is_none());
    assert!(report.compatibility_exception.is_none());
    assert!(
        report
            .blocking_reason
            .as_deref()
            .is_some_and(|reason| reason.contains(blocking_reason))
    );
}

#[test]
fn adapt_protocol_007_adapter_compatibility_profiles_are_generic_and_complete() {
    let terminal_import_profile = {
        let mut profile = default_agent_profile("tb", AgentKind::Custom, "agent");
        profile.labels.insert(
            "terminal_bench_agent_import_path".to_string(),
            "bench_agents.fake:Agent".to_string(),
        );
        profile
    };
    let terminal_sandbox_profile = {
        let mut profile = default_agent_profile("tb", AgentKind::Custom, "agent");
        profile.labels.insert(
            "terminal_bench_agent".to_string(),
            "claude-code".to_string(),
        );
        profile
    };
    let swe_gold_profile = {
        let mut profile = default_agent_profile("swe", AgentKind::Custom, "agent");
        profile
            .labels
            .insert("swe_bench_pro_agent".to_string(), "gold".to_string());
        profile
    };
    let swe_sandbox_profile = default_agent_profile("swe", AgentKind::Custom, "agent");

    let terminal_import_compats =
        super::super::super::adapter_compatibility_profiles(&terminal_import_profile);
    assert_eq!(
        terminal_import_compats.len(),
        2,
        "all registered adapters must produce a profile"
    );
    let terminal_bench_compat = terminal_import_compats
        .iter()
        .find(|c| c.bridge_mode.starts_with("terminal-bench"))
        .expect("terminal-bench profile must be present");
    assert_eq!(
        terminal_bench_compat.host_execution_reason,
        Some("terminal-bench import agent host path")
    );
    assert_eq!(
        terminal_bench_compat.bridge_mode,
        "terminal-bench-import-path"
    );
    assert!(
        terminal_bench_compat
            .consumed_label_keys
            .contains(&"terminal_bench_agent_import_path")
    );

    let swe_gold_compats = super::super::super::adapter_compatibility_profiles(&swe_gold_profile);
    let swe_bench_compat = swe_gold_compats
        .iter()
        .find(|c| c.bridge_mode.starts_with("swe-bench-pro"))
        .expect("swe-bench-pro profile must be present");
    assert_eq!(
        swe_bench_compat.host_execution_reason,
        Some("swe-bench-pro gold host path")
    );
    assert_eq!(swe_bench_compat.bridge_mode, "swe-bench-pro-gold");
    assert!(
        swe_bench_compat
            .consumed_label_keys
            .contains(&"swe_bench_pro_agent")
    );

    let terminal_sandbox_compats =
        super::super::super::adapter_compatibility_profiles(&terminal_sandbox_profile);
    let terminal_sandbox_compat = terminal_sandbox_compats
        .iter()
        .find(|c| c.bridge_mode.starts_with("terminal-bench"))
        .unwrap();
    assert_eq!(terminal_sandbox_compat.host_execution_reason, None);
    assert_eq!(
        terminal_sandbox_compat.bridge_mode,
        "terminal-bench-official-agent"
    );

    let swe_sandbox_compats =
        super::super::super::adapter_compatibility_profiles(&swe_sandbox_profile);
    let swe_sandbox_compat = swe_sandbox_compats
        .iter()
        .find(|c| c.bridge_mode.starts_with("swe-bench-pro"))
        .unwrap();
    assert_eq!(swe_sandbox_compat.host_execution_reason, None);
    assert_eq!(
        swe_sandbox_compat.bridge_mode,
        "swe-bench-pro-sandbox-agent"
    );
}

#[test]
fn adapt_protocol_007_doctor_run_as_consumes_profiles_without_benchmark_branching() {
    let doctor_run_as_source = include_str!("../../doctor_run_as.rs");
    let runner_source = include_str!("../../runner.rs");

    assert!(
        !doctor_run_as_source.contains("harnesslab.terminal-bench.runtime"),
        "doctor_run_as.rs must not branch on terminal-bench adapter_id"
    );
    assert!(
        !doctor_run_as_source.contains("harnesslab.swe-bench-pro.runtime"),
        "doctor_run_as.rs must not branch on swe-bench-pro adapter_id"
    );
    assert!(
        doctor_run_as_source.contains("adapter_compatibility_profiles"),
        "doctor_run_as.rs must consume adapter_compatibility_profiles"
    );
    assert!(
        !runner_source.contains("harnesslab.terminal-bench.runtime"),
        "runner.rs adapter_compatibility_profiles must not branch on terminal-bench adapter_id"
    );
    assert!(
        !runner_source.contains("harnesslab.swe-bench-pro.runtime"),
        "runner.rs adapter_compatibility_profiles must not branch on swe-bench-pro adapter_id"
    );
    assert!(
        runner_source.contains("runtime_adapters()"),
        "runner.rs must enumerate adapters dynamically via runtime_adapters()"
    );

    let mut terminal_import_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    terminal_import_profile.labels.insert(
        "terminal_bench_agent_import_path".to_string(),
        "bench_agents.fake:Agent".to_string(),
    );
    let host_reasons: Vec<_> =
        super::super::super::adapter_compatibility_profiles(&terminal_import_profile)
            .into_iter()
            .filter_map(|compat| compat.host_execution_reason)
            .collect();
    assert_eq!(host_reasons.len(), 1);
    assert_eq!(host_reasons[0], "terminal-bench import agent host path");

    let mut swe_gold_profile = default_agent_profile("swe", AgentKind::Custom, "agent");
    swe_gold_profile
        .labels
        .insert("swe_bench_pro_agent".to_string(), "gold".to_string());
    let host_reasons: Vec<_> =
        super::super::super::adapter_compatibility_profiles(&swe_gold_profile)
            .into_iter()
            .filter_map(|compat| compat.host_execution_reason)
            .collect();
    assert_eq!(host_reasons.len(), 1);
    assert_eq!(host_reasons[0], "swe-bench-pro gold host path");

    let sandbox_profile = default_agent_profile("sandbox", AgentKind::Custom, "agent");
    let host_reasons: Vec<_> =
        super::super::super::adapter_compatibility_profiles(&sandbox_profile)
            .into_iter()
            .filter_map(|compat| compat.host_execution_reason)
            .collect();
    assert!(
        host_reasons.is_empty(),
        "sandbox profiles must not produce host_execution_reason"
    );
}

fn assert_runtime_label_access_is_allowlisted() {
    let sources = [
        include_str!("runtime_adapter.rs"),
        include_str!("terminal_bench_adapter.rs"),
        include_str!("terminal_bench.rs"),
        include_str!("terminal_bench_env.rs"),
        include_str!("swe_bench_pro_adapter.rs"),
        include_str!("swe_bench_pro/agent.rs"),
        include_str!("../../doctor_run_as.rs"),
    ];
    for source in sources {
        assert!(
            !source.contains(".labels"),
            "benchmark runtime label reads must go through runtime_compatibility.rs"
        );
    }
}
