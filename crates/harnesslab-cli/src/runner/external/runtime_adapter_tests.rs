use super::*;
use harnesslab_core::{
    AgentKind, ArtifactSpec, AttemptProvenance, BenchmarkIdentity, BenchmarkPlan, BenchmarkRef,
    ExecutionConfig, ExternalRunnerSpec, NetworkPolicy, ResourceHint, RunConfigOverrides, RunPaths,
    RunSpec, RuntimePreflightReport, RuntimeTaskSnapshot, SandboxSpec, SourceRef, TaskPlan,
    VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType, default_agent_profile,
};

#[test]
fn adapt_runtime_001_external_entrypoints_delegate_to_runtime_registry() {
    assert_eq!(
        runtime_adapter_for(ExternalRunnerKind::TerminalBench).kind(),
        ExternalRunnerKind::TerminalBench
    );
    assert_eq!(
        runtime_adapter_for(ExternalRunnerKind::SweBenchPro).kind(),
        ExternalRunnerKind::SweBenchPro
    );

    let external_entrypoint = include_str!("../external.rs");
    let runner_entrypoint = include_str!("../../runner.rs");
    let cleanup_entrypoint = include_str!("../cleanup.rs");

    assert!(!external_entrypoint.contains("ExternalRunnerKind::TerminalBench"));
    assert!(!external_entrypoint.contains("ExternalRunnerKind::SweBenchPro"));
    assert!(!runner_entrypoint.contains("ExternalRunnerKind::TerminalBench"));
    assert!(!runner_entrypoint.contains("ExternalRunnerKind::SweBenchPro"));
    assert!(!cleanup_entrypoint.contains("ExternalRunnerKind::TerminalBench"));
    assert!(!cleanup_entrypoint.contains("ExternalRunnerKind::SweBenchPro"));
    assert!(external_entrypoint.contains("preflight_external_task(profile, task)?"));
    assert!(external_entrypoint.contains("let adapter = runtime_adapter_for(runner.kind)"));
    assert!(external_entrypoint.contains("adapter.execute(&ctx)"));
    assert!(!cleanup_entrypoint.contains("terminal_bench"));
    assert!(!cleanup_entrypoint.contains("compose_cleanup"));
    assert!(!external_entrypoint.contains("terminal_bench::execute"));
    assert!(!external_entrypoint.contains("swe_bench_pro::execute"));
    assert!(!external_entrypoint.contains("terminal_bench::validate_profile"));
    assert_runtime_label_access_is_allowlisted();
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
        &external_task("tb-task", ExternalRunnerKind::TerminalBench),
    )
    .unwrap();
    assert_preflight(
        terminal_report,
        ExternalRunnerKind::TerminalBench,
        Some("terminal-bench import agent host path"),
    );

    let mut terminal_sandbox_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    terminal_sandbox_profile.labels.insert(
        "terminal_bench_agent".to_string(),
        "claude-code".to_string(),
    );
    let terminal_sandbox_report = preflight_external_task(
        &terminal_sandbox_profile,
        &external_task("tb-task", ExternalRunnerKind::TerminalBench),
    )
    .unwrap();
    assert_preflight(
        terminal_sandbox_report,
        ExternalRunnerKind::TerminalBench,
        None,
    );

    let invalid_terminal_profile = default_agent_profile("tb", AgentKind::Custom, "agent");
    let terminal_blocked_report = preflight_external_task(
        &invalid_terminal_profile,
        &external_task("tb-task", ExternalRunnerKind::TerminalBench),
    )
    .unwrap();
    assert_blocked_preflight(
        terminal_blocked_report,
        ExternalRunnerKind::TerminalBench,
        "terminal_bench_agent",
    );
    let terminal_blocked_error = super::super::validate_profile_for_plan(
        &invalid_terminal_profile,
        &[external_task("tb-task", ExternalRunnerKind::TerminalBench)],
    )
    .unwrap_err()
    .to_string();
    assert!(terminal_blocked_error.contains("runtime preflight blocked"));
    assert!(terminal_blocked_error.contains("terminal-bench-runtime"));
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
        &external_task("swe-task", ExternalRunnerKind::SweBenchPro),
    )
    .unwrap();
    assert_preflight(
        swe_report,
        ExternalRunnerKind::SweBenchPro,
        Some("swe-bench-pro gold host path"),
    );

    let swe_sandbox_profile = default_agent_profile("swe", AgentKind::Custom, "agent");
    let swe_sandbox_report = preflight_external_task(
        &swe_sandbox_profile,
        &external_task("swe-task", ExternalRunnerKind::SweBenchPro),
    )
    .unwrap();
    assert_preflight(swe_sandbox_report, ExternalRunnerKind::SweBenchPro, None);

    let mut swe_missing_source = external_task("swe-task", ExternalRunnerKind::SweBenchPro);
    swe_missing_source
        .external_runner
        .as_mut()
        .unwrap()
        .source_path = None;
    let swe_missing_source_report =
        preflight_external_task(&swe_sandbox_profile, &swe_missing_source).unwrap();
    assert_blocked_preflight(
        swe_missing_source_report,
        ExternalRunnerKind::SweBenchPro,
        "source_path",
    );

    let error = super::super::validate_profile_for_plan(
        &terminal_profile,
        &[external_task("tb-task", ExternalRunnerKind::TerminalBench)],
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
        &[external_task("tb-task", ExternalRunnerKind::TerminalBench)],
    )
    .unwrap();
    let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
    assert!(events.contains("\"event\":\"external_runner_preflight\""));
    assert!(events.contains("adapter_id=terminal-bench-runtime"));
    assert!(events.contains("adapter_phase=preflight"));
    assert!(events.contains("runner_kind=TerminalBench"));
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
    let mut task = external_task("swe-task", ExternalRunnerKind::SweBenchPro);
    let dataset_dir = run_dir.path().join("dataset");
    std::fs::create_dir_all(&dataset_dir).unwrap();
    let runner = task.external_runner.as_mut().unwrap();
    runner.dataset_path = dataset_dir.display().to_string();
    runner.source_path = None;
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

    let result = runtime_adapter_for(ExternalRunnerKind::SweBenchPro)
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
    runner_kind: ExternalRunnerKind,
    host_execution_reason: Option<&str>,
) {
    assert!(!report.task_id.is_empty());
    assert_eq!(report.runner_kind, runner_kind);
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
    runner_kind: ExternalRunnerKind,
    blocking_reason: &str,
) {
    assert!(!report.task_id.is_empty());
    assert_eq!(report.runner_kind, runner_kind);
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

fn run_spec(run_dir: &std::path::Path) -> RunSpec {
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

fn external_task(task_id: &str, kind: ExternalRunnerKind) -> TaskPlan {
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
            kind,
            dataset_path: "dataset".to_string(),
            source_path: (kind == ExternalRunnerKind::SweBenchPro).then_some("source".to_string()),
            agent_timeout_sec: None,
        }),
    }
}
