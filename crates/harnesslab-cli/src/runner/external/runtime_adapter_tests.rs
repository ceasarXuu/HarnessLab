use super::*;
use harnesslab_core::{
    AgentKind, ArtifactSpec, ExternalRunnerSpec, NetworkPolicy, ResourceHint,
    RuntimePreflightReport, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec,
    WorkspaceSpec, WorkspaceType, default_agent_profile,
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

    assert!(!external_entrypoint.contains("ExternalRunnerKind::TerminalBench"));
    assert!(!external_entrypoint.contains("ExternalRunnerKind::SweBenchPro"));
    assert!(!runner_entrypoint.contains("ExternalRunnerKind::TerminalBench"));
    assert!(!runner_entrypoint.contains("ExternalRunnerKind::SweBenchPro"));
    assert!(external_entrypoint.contains("preflight_external_task(profile, task)?"));
    assert!(external_entrypoint.contains("runtime_adapter_for(runner.kind).execute(ctx)"));
    assert!(!external_entrypoint.contains("terminal_bench::execute"));
    assert!(!external_entrypoint.contains("swe_bench_pro::execute"));
    assert!(!external_entrypoint.contains("terminal_bench::validate_profile"));
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
    let terminal_error = preflight_external_task(
        &invalid_terminal_profile,
        &external_task("tb-task", ExternalRunnerKind::TerminalBench),
    )
    .unwrap_err()
    .to_string();
    assert!(terminal_error.contains("terminal_bench_agent"));

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

    let error = super::super::validate_profile_for_plan(
        &terminal_profile,
        &[external_task("tb-task", ExternalRunnerKind::TerminalBench)],
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("setup.run_as"));
    assert!(error.contains("terminal-bench import agent host path"));
    assert!(error.contains("tb-task"));
}

fn assert_preflight(
    report: RuntimePreflightReport,
    runner_kind: ExternalRunnerKind,
    host_execution_reason: Option<&str>,
) {
    assert!(!report.task_id.is_empty());
    assert_eq!(report.runner_kind, runner_kind);
    assert_eq!(
        report.host_execution_reason.as_deref(),
        host_execution_reason
    );
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
