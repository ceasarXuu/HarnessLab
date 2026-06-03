use super::*;
use harnesslab_core::{
    AgentKind, AttemptProvenance, AuthConfig, EvaluationRecord, FailureClass, FailureCode,
    InputMode, ModelError, NetworkPolicy, Outcome, ProcessRecord, TerminationReason, UsageConfig,
    UsageRecord, WorkingDirMode, classify_agent_process, classify_evaluation_process,
    default_agent_profile, derive_exit_code, health_impact_for_failure, validate_run_spec,
};

#[test]
fn agt_001_renderer_covers_argument_and_file_modes() {
    let tmp = tempfile::tempdir().unwrap();
    let task = test_task();
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "echo {{instruction}}");
    profile.input_mode = InputMode::Argument;
    let rendered = render_command(&profile, &task, tmp.path(), false).unwrap();
    assert!(rendered.contains("'instruction'"));

    profile.input_mode = InputMode::File;
    let rendered = render_command(&profile, &task, tmp.path(), false).unwrap();
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
        usage: UsageConfig::default(),
        setup: Default::default(),
        skills: Default::default(),
        tools: Default::default(),
        hooks: Default::default(),
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
    assert_eq!(derive_exit_code(&[partial], false), 0);
}

#[test]
fn run_004_planned_attempts_repeat_each_task_by_configured_attempts() {
    let plan = test_plan(vec![task_with_id("task-a"), task_with_id("task-b")]);

    let attempts = planned_attempts(&plan, 2);
    let keys = attempts
        .iter()
        .map(|attempt| (attempt.task.task_id.as_str(), attempt.attempt))
        .collect::<Vec<_>>();

    assert_eq!(
        keys,
        vec![("task-a", 1), ("task-a", 2), ("task-b", 1), ("task-b", 2)]
    );
}

#[test]
fn replay_002_resume_keeps_completed_attempts_and_schedules_missing_only() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = test_plan(vec![task_with_id("task-a")]);
    let completed = attempt(FailureClass::None, None);
    atomic_write_json(
        &attempt_result_path(tmp.path(), "task-a", 1),
        &TaskAttemptResult {
            task_id: "task-a".to_string(),
            attempt: 1,
            ..completed
        },
    )
    .unwrap();

    let (loaded, pending) = partition_attempts(tmp.path(), &plan, 2).unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].task_id, "task-a");
    assert_eq!(loaded[0].attempt, 1);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].task.task_id, "task-a");
    assert_eq!(pending[0].attempt, 2);
}

#[test]
fn replay_002_resume_failed_completed_attempt_schedules_recovery_attempt() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = test_plan(vec![task_with_id("task-a")]);
    let mut failed = attempt(FailureClass::Execution, Some(FailureCode::AgentTimeout));
    failed.state = TaskState::Failure;
    failed.outcome = Outcome::Failure;
    atomic_write_json(
        &attempt_result_path(tmp.path(), "task-a", 1),
        &TaskAttemptResult {
            task_id: "task-a".to_string(),
            attempt: 1,
            ..failed
        },
    )
    .unwrap();

    let (loaded, pending) = partition_attempts(tmp.path(), &plan, 1).unwrap();

    assert_eq!(loaded.len(), 1);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].task.task_id, "task-a");
    assert_eq!(pending[0].attempt, 2);
}

#[test]
fn replay_002_resume_does_not_create_unbounded_recovery_attempts() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = test_plan(vec![task_with_id("task-a")]);
    for attempt_number in [1, 2] {
        let mut failed = attempt(FailureClass::Execution, Some(FailureCode::AgentTimeout));
        failed.state = TaskState::Failure;
        failed.outcome = Outcome::Failure;
        atomic_write_json(
            &attempt_result_path(tmp.path(), "task-a", attempt_number),
            &TaskAttemptResult {
                task_id: "task-a".to_string(),
                attempt: attempt_number,
                ..failed
            },
        )
        .unwrap();
    }

    let (loaded, pending) = partition_attempts(tmp.path(), &plan, 1).unwrap();

    assert_eq!(loaded.len(), 2);
    assert!(pending.is_empty());
}

#[test]
fn replay_002_resume_ignores_run_health_aborted_placeholders() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = test_plan(vec![task_with_id("task-a")]);
    for attempt_number in [1, 2] {
        let mut aborted = attempt(FailureClass::Execution, Some(FailureCode::RunHealthAborted));
        aborted.state = TaskState::Interrupted;
        aborted.outcome = Outcome::Failure;
        atomic_write_json(
            &attempt_result_path(tmp.path(), "task-a", attempt_number),
            &TaskAttemptResult {
                task_id: "task-a".to_string(),
                attempt: attempt_number,
                ..aborted
            },
        )
        .unwrap();
    }

    let (loaded, pending) = partition_attempts(tmp.path(), &plan, 2).unwrap();

    assert!(loaded.is_empty());
    assert_eq!(
        pending.iter().map(|work| work.attempt).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn replay_002_resume_uses_encoded_task_dir_for_slash_bearing_task_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = test_plan(vec![task_with_id("task/slash")]);
    let completed = attempt(FailureClass::None, None);
    atomic_write_json(
        &attempt_result_path(tmp.path(), "task/slash", 1),
        &TaskAttemptResult {
            task_id: "task/slash".to_string(),
            attempt: 1,
            ..completed
        },
    )
    .unwrap();

    let (loaded, pending) = partition_attempts(tmp.path(), &plan, 1).unwrap();

    assert_eq!(loaded[0].task_id, "task/slash");
    assert!(
        tmp.path()
            .join("tasks/task%2Fslash/attempts/1/result.json")
            .exists()
    );
    assert!(pending.is_empty());
}

#[test]
fn replay_003_replay_spec_preserves_execution_config_and_links_source() {
    let mut source = valid_spec();
    source.run_id = "source-run".to_string();
    source.execution.attempts = 3;
    source.execution.concurrency = 7;
    source.execution.network = NetworkPolicy::Full;
    let run_dir = std::path::PathBuf::from("/tmp/replay-run");

    let replay = replay_spec_from_source(
        &source,
        "replay-run".to_string(),
        "2026-05-27T00:00:00Z".to_string(),
        &run_dir,
    );

    assert_eq!(replay.run_id, "replay-run");
    assert_eq!(replay.created_at, "2026-05-27T00:00:00Z");
    assert_eq!(replay.execution, source.execution);
    assert_eq!(replay.agent_profile_ref, source.agent_profile_ref);
    assert_eq!(replay.benchmark, source.benchmark);
    assert_eq!(replay.replay_source_run_id, Some("source-run".to_string()));
    assert_eq!(replay.paths.run_dir, "/tmp/replay-run");
}

#[test]
fn run_005_docker_request_uses_run_network_and_task_sandbox_spec() {
    let workspace = std::path::PathBuf::from("/tmp/ws");
    let mut spec = valid_spec();
    spec.execution.network = NetworkPolicy::Full;
    let mut task = task_with_id("task/docker");
    task.sandbox_spec.image = "ubuntu:24.04".to_string();
    task.sandbox_spec.mounts = vec!["/cache:/cache:ro".to_string()];
    task.sandbox_spec.env_vars = vec!["A=B".to_string()];
    task.sandbox_spec.resource_limits.cpu_cores = 3;
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "true");
    profile.auth.inherit_env = vec!["OPENAI_API_KEY".to_string()];
    profile.auth.include_paths = vec!["/auth:/root/.auth:ro".to_string()];

    let request = docker_create_request(&spec, &profile, &task, 2, &workspace);

    assert_eq!(request.run_id, "run-1");
    assert_eq!(request.task_id, "task/docker");
    assert_eq!(request.attempt, 2);
    assert_eq!(request.image, "ubuntu:24.04");
    assert_eq!(request.workspace_host_path, workspace);
    assert_eq!(request.workspace_container_path, "/workspace");
    assert_eq!(request.network, NetworkPolicy::Full);
    assert_eq!(
        request.mounts,
        vec!["/cache:/cache:ro", "/auth:/root/.auth:ro"]
    );
    assert_eq!(request.env_vars, vec!["A=B", "OPENAI_API_KEY"]);
    assert_eq!(request.cpu_cores, 3);
}

#[test]
fn run_005_host_fixture_does_not_request_docker() {
    let task = test_task();

    assert!(!task_requires_docker(&task));

    let mut docker_task = task_with_id("docker-task");
    docker_task.sandbox_spec.image = "debian:stable-slim".to_string();
    assert!(task_requires_docker(&docker_task));
}

#[test]
fn run_006_run_agent_host_executes_inside_workspace() {
    let tmp = tempfile::tempdir().unwrap();
    let workspace = tmp.path().join("workspace");
    let profile = default_agent_profile("fake", AgentKind::Fake, "printf ok > result.txt");
    let materialized_profile = crate::agent_registry::materialize_profile(&profile).unwrap();
    let task = test_task();

    let result = run_agent(AgentRunRequest {
        spec: &valid_spec(),
        profile: &profile,
        report_profile: &profile,
        materialized_profile: &materialized_profile,
        report_materialized_profile: &materialized_profile,
        task: &task,
        attempt: 1,
        attempt_dir: tmp.path(),
        workspace: &workspace,
    })
    .unwrap();

    assert_eq!(result.process.exit_code, Some(0));
    assert_eq!(
        std::fs::read_to_string(workspace.join("result.txt")).unwrap(),
        "ok"
    );
    assert!(result.sandbox_failure.is_none());
}

#[test]
fn run_007_run_shell_reports_failed_command() {
    let tmp = tempfile::tempdir().unwrap();

    let error = run_shell(tmp.path(), "exit 9").unwrap_err().to_string();

    assert!(error.contains("command failed"));
}

#[test]
fn run_008_panic_message_preserves_string_payloads() {
    assert_eq!(panic_message(Box::new("borrowed panic")), "borrowed panic");
    assert_eq!(
        panic_message(Box::new("owned panic".to_string())),
        "owned panic"
    );
    assert_eq!(panic_message(Box::new(7_u8)), "non-string panic payload");
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
        provenance: AttemptProvenance::Original,
        state: TaskState::Success,
        outcome: Outcome::Success,
        failure_class: class,
        failure_code: code,
        health_impact: health_impact_for_failure(class, code),
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
        external_runner: None,
    }
}

fn task_with_id(task_id: &str) -> TaskPlan {
    let mut task = test_task();
    task.task_id = task_id.to_string();
    task
}

fn test_plan(tasks: Vec<TaskPlan>) -> harnesslab_core::BenchmarkPlan {
    harnesslab_core::BenchmarkPlan {
        benchmark: harnesslab_core::BenchmarkIdentity {
            name: "fake-terminal".to_string(),
            version: "fixture".to_string(),
        },
        split: "success".to_string(),
        prepared_benchmark_ref: "fixture".to_string(),
        tasks,
        run_config_overrides: harnesslab_core::RunConfigOverrides {
            timeout_sec: None,
            network: None,
        },
        warnings: Vec::new(),
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
