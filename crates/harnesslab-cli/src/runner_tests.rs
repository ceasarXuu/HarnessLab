use super::*;
use harnesslab_core::{
    AgentKind, AuthConfig, EvaluationRecord, FailureClass, FailureCode, InputMode, ModelError,
    NetworkPolicy, Outcome, ProcessRecord, TerminationReason, UsageConfig, WorkingDirMode,
    classify_agent_process, classify_evaluation_process, default_agent_profile, derive_exit_code,
    validate_run_spec,
};

#[test]
fn agt_001_renderer_covers_argument_and_file_modes() {
    let tmp = tempfile::tempdir().unwrap();
    let task = test_task();
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "echo {{instruction}}");
    profile.input_mode = InputMode::Argument;
    let rendered = render_command(&profile, &task, tmp.path()).unwrap();
    assert!(rendered.contains("'instruction'"));

    profile.input_mode = InputMode::File;
    let rendered = render_command(&profile, &task, tmp.path()).unwrap();
    assert!(rendered.contains("instruction.txt"));
}

#[test]
fn replay_001_missing_profile_file_is_error() {
    let tmp = tempfile::tempdir().unwrap();

    assert!(load_profile(tmp.path(), "missing").is_err());
}

#[test]
fn agt_001_profile_name_rejects_path_traversal_before_read() {
    let tmp = tempfile::tempdir().unwrap();

    let error = load_profile(tmp.path(), "../escape")
        .unwrap_err()
        .to_string();

    assert!(error.contains("invalid agent profile name"));
}

#[test]
fn agt_004_manual_profile_can_be_serialized() {
    let profile = AgentProfile {
        schema_version: 1,
        name: "manual".to_string(),
        kind: AgentKind::Custom,
        display_name: "Manual".to_string(),
        command: "true".to_string(),
        input_mode: InputMode::Stdin,
        working_dir: WorkingDirMode::Workspace,
        timeout_sec: 1,
        version_command: None,
        auth: AuthConfig {
            inherit: false,
            inherit_env: Vec::new(),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            mount_ssh_socket: false,
            mount_docker_socket: false,
        },
        usage: UsageConfig {
            parser: "none".to_string(),
        },
        labels: Default::default(),
    };

    assert!(toml::to_string(&profile).unwrap().contains("manual"));
}

#[test]
fn core_contracts_are_exercised_from_cli_crate_context() {
    let mut spec = valid_spec();
    spec.schema_version = 2;
    assert_eq!(
        validate_run_spec(&spec),
        Err(ModelError::UnsupportedSchema(2))
    );
    spec = valid_spec();
    spec.benchmark.split.clear();
    assert_eq!(validate_run_spec(&spec), Err(ModelError::MissingBenchmark));
    spec = valid_spec();
    spec.execution.concurrency = 0;
    assert_eq!(validate_run_spec(&spec), Err(ModelError::InvalidExecution));

    assert_eq!(
        classify_agent_process(&process(Some(0))).class,
        FailureClass::None
    );
    assert_eq!(
        classify_agent_process(&process(None)).code,
        Some(FailureCode::AgentNonzeroExit)
    );
    assert_eq!(
        classify_evaluation_process(&evaluation(Some(0), 0.0)).code,
        Some(FailureCode::TestFailed)
    );
    assert_eq!(
        classify_evaluation_process(&evaluation(None, 0.0)).code,
        Some(FailureCode::VerifierError)
    );
    assert_eq!(derive_exit_code(&[], false), 3);
    let mut partial = attempt(FailureClass::None, None);
    partial.outcome = Outcome::PartialSuccess;
    assert_eq!(derive_exit_code(&[partial], false), 4);
}

fn process(exit_code: Option<i32>) -> ProcessRecord {
    ProcessRecord {
        exit_code,
        termination_reason: TerminationReason::Completed,
        stdout_path: "stdout.log".to_string(),
        stderr_path: "stderr.log".to_string(),
    }
}

fn evaluation(exit_code: Option<i32>, raw_score: f64) -> EvaluationRecord {
    EvaluationRecord {
        exit_code,
        raw_score,
        stdout_path: "stdout.log".to_string(),
        stderr_path: "stderr.log".to_string(),
    }
}

fn attempt(class: FailureClass, code: Option<FailureCode>) -> TaskAttemptResult {
    TaskAttemptResult {
        schema_version: 1,
        task_id: "task".to_string(),
        attempt: 1,
        state: TaskState::Success,
        outcome: Outcome::Success,
        failure_class: class,
        failure_code: code,
        benchmark_score: 1.0,
        duration_ms: 1,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: Vec::new(),
    }
}

fn test_task() -> TaskPlan {
    harnesslab_core::TaskPlan {
        task_id: "task".to_string(),
        instruction: "instruction".to_string(),
        workspace_spec: harnesslab_core::WorkspaceSpec {
            workspace_type: harnesslab_core::WorkspaceType::Empty,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: harnesslab_core::SandboxSpec {
            image: "host".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::None,
            privileged: false,
            resource_limits: harnesslab_core::ResourceHint {
                cpu_cores: 1,
                memory_mb: 128,
            },
        },
        verifier_spec: harnesslab_core::VerifierSpec {
            command: "true".to_string(),
            working_dir: "workspace".to_string(),
            timeout_sec: 1,
            expected_exit_codes: vec![0],
            environment_mode: harnesslab_core::VerifierEnvironment::HostProcess,
            output_parser: "exit_code".to_string(),
        },
        artifact_spec: harnesslab_core::ArtifactSpec {
            base_dir: "workspace".to_string(),
            globs: Vec::new(),
            required_paths: Vec::new(),
            max_size_bytes: 1,
        },
        patch_spec: None,
    }
}

fn valid_spec() -> RunSpec {
    RunSpec {
        schema_version: 1,
        run_id: "run-1".to_string(),
        created_at: "2026-05-26T00:00:00Z".to_string(),
        agent_profile_ref: "fake".to_string(),
        benchmark: BenchmarkRef {
            name: "fake-terminal".to_string(),
            version: "fixture".to_string(),
            split: "success".to_string(),
        },
        execution: harnesslab_core::ExecutionConfig {
            concurrency: 1,
            attempts: 1,
            network: NetworkPolicy::None,
            timeout_sec: None,
        },
        paths: RunPaths {
            run_dir: "run".to_string(),
        },
        replay_source_run_id: None,
    }
}
