use super::*;
use crate::UsageRecord;

#[test]
fn core_001_state_machine_allows_expected_lifecycle() {
    let path = [
        TaskState::Pending,
        TaskState::Preparing,
        TaskState::AgentRunning,
        TaskState::Evaluating,
        TaskState::Collecting,
        TaskState::Success,
    ];

    for pair in path.windows(2) {
        assert!(ensure_transition(pair[0], pair[1]).is_ok());
    }
}

#[test]
fn core_001_state_machine_allows_failure_and_interrupt_edges() {
    let transitions = [
        (TaskState::Collecting, TaskState::PartialSuccess),
        (TaskState::Collecting, TaskState::Failure),
        (TaskState::Preparing, TaskState::Failure),
        (TaskState::AgentRunning, TaskState::Failure),
        (TaskState::Evaluating, TaskState::Failure),
        (TaskState::Preparing, TaskState::Interrupted),
        (TaskState::AgentRunning, TaskState::Interrupted),
        (TaskState::Evaluating, TaskState::Interrupted),
        (TaskState::Collecting, TaskState::Interrupted),
    ];

    for (from, to) in transitions {
        assert!(can_transition(from, to));
    }
    assert!(!can_transition(TaskState::Success, TaskState::Failure));
}

#[test]
fn core_002_state_machine_rejects_terminal_to_running() {
    assert_eq!(
        ensure_transition(TaskState::Success, TaskState::AgentRunning),
        Err(ModelError::InvalidTransition {
            from: TaskState::Success,
            to: TaskState::AgentRunning
        })
    );
}

#[test]
fn core_003_failure_classifier_maps_agent_timeout() {
    let result = ProcessRecord {
        exit_code: None,
        termination_reason: TerminationReason::Timeout,
        stdout_path: "agent/stdout.log".to_string(),
        stderr_path: "agent/stderr.log".to_string(),
    };

    let failure = classify_agent_process(&result);

    assert_eq!(failure.class, FailureClass::Execution);
    assert_eq!(failure.code, Some(FailureCode::AgentTimeout));
}

#[test]
fn core_004_failure_classifier_maps_failed_verifier() {
    let result = EvaluationRecord {
        exit_code: Some(0),
        raw_score: 0.0,
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    };

    let failure = classify_evaluation_process(&result);

    assert_eq!(failure.class, FailureClass::Benchmark);
    assert_eq!(failure.code, Some(FailureCode::TestFailed));
}

#[test]
fn core_001_old_task_plan_snapshot_defaults_external_runner() {
    let json = r#"{
        "task_id": "task-1",
        "instruction": "do it",
        "workspace_spec": {"workspace_type":"empty","target_path":"workspace","clean":true},
        "sandbox_spec": {
            "image":"host",
            "mounts":[],
            "env_vars":[],
            "network":"none",
            "privileged":false,
            "resource_limits":{"cpu_cores":1,"memory_mb":128}
        },
        "verifier_spec": {
            "command":"true",
            "working_dir":"workspace",
            "timeout_sec":1,
            "expected_exit_codes":[0],
            "environment_mode":"host_process",
            "output_parser":"exit_code"
        },
        "artifact_spec": {
            "base_dir":"workspace",
            "globs":[],
            "required_paths":[],
            "max_size_bytes":1
        },
        "patch_spec": null
    }"#;

    let task: crate::TaskPlan = serde_json::from_str(json).unwrap();

    assert!(task.external_runner.is_none());
}

#[test]
fn orch_003_exit_code_mapping_covers_command_health() {
    assert_eq!(derive_exit_code(&[], true), 3);
    assert_eq!(derive_exit_code(&[], false), 3);
    assert_eq!(
        derive_exit_code(&[attempt(FailureClass::None, None)], false),
        0
    );
    let benchmark = attempt(FailureClass::Benchmark, Some(FailureCode::TestFailed));
    assert_eq!(derive_exit_code(&[benchmark], false), 0);
    let mut partial = attempt(FailureClass::None, None);
    partial.outcome = Outcome::PartialSuccess;
    assert_eq!(derive_exit_code(&[partial], false), 0);
    let mut interrupted = attempt(FailureClass::None, None);
    interrupted.state = TaskState::Interrupted;
    assert_eq!(derive_exit_code(&[interrupted], false), 130);
    let mut benchmark = attempt(FailureClass::Benchmark, Some(FailureCode::TestFailed));
    benchmark.task_id = "benchmark-task".to_string();
    let mut execution = attempt(FailureClass::Execution, Some(FailureCode::AgentTimeout));
    execution.task_id = "execution-task".to_string();
    let results = vec![benchmark, execution];

    assert_eq!(derive_exit_code(&results, false), 1);
}

#[test]
fn orch_001_summary_counts_partial_and_interrupted() {
    let mut partial = attempt(FailureClass::None, None);
    partial.outcome = Outcome::PartialSuccess;
    partial.benchmark_score = 0.5;
    partial.duration_ms = 7;
    let mut interrupted = attempt(FailureClass::Execution, Some(FailureCode::AgentTimeout));
    interrupted.task_id = "interrupted".to_string();
    interrupted.state = TaskState::Interrupted;
    interrupted.duration_ms = 3;

    let results = summarize_results("run", vec![partial, interrupted]);

    assert_eq!(results.summary.partial_success, 1);
    assert_eq!(results.summary.execution_failure, 1);
    assert_eq!(results.summary.interrupted, 1);
    assert_eq!(results.summary.total_duration_ms, 10);
    assert_eq!(results.summary.total_score, 0.5);
}

#[test]
fn orch_001_summary_and_exit_code_use_latest_attempt_per_task() {
    let mut failed = attempt(FailureClass::Execution, Some(FailureCode::AgentTimeout));
    failed.attempt = 1;
    failed.duration_ms = 3;
    let mut recovered = attempt(FailureClass::None, None);
    recovered.attempt = 2;
    recovered.benchmark_score = 1.0;
    recovered.duration_ms = 5;

    let results = vec![failed, recovered];

    assert_eq!(derive_exit_code(&results, false), 0);
    let summary = summarize_results("run", results).summary;
    assert_eq!(summary.total_tasks, 1);
    assert_eq!(summary.success, 1);
    assert_eq!(summary.execution_failure, 0);
    assert_eq!(summary.total_duration_ms, 5);
    assert_eq!(summary.total_score, 1.0);
}

#[test]
fn core_001_metadata_is_empty_until_schema_requires_fields() {
    assert!(metadata().is_empty());
}

#[test]
fn core_003_failure_classifier_maps_spawn_signal_and_nonzero() {
    for (reason, exit_code, expected) in [
        (
            TerminationReason::SpawnError,
            None,
            FailureCode::AgentSpawnError,
        ),
        (
            TerminationReason::Signaled,
            None,
            FailureCode::AgentSignaled,
        ),
        (
            TerminationReason::Completed,
            Some(2),
            FailureCode::AgentNonzeroExit,
        ),
    ] {
        let failure = classify_agent_process(&ProcessRecord {
            exit_code,
            termination_reason: reason,
            stdout_path: "stdout.log".to_string(),
            stderr_path: "stderr.log".to_string(),
        });
        assert_eq!(failure.code, Some(expected));
    }
    let missing_exit = classify_agent_process(&ProcessRecord {
        exit_code: None,
        termination_reason: TerminationReason::Completed,
        stdout_path: "stdout.log".to_string(),
        stderr_path: "stderr.log".to_string(),
    });
    assert_eq!(missing_exit.code, Some(FailureCode::AgentNonzeroExit));
}

#[test]
fn core_001_run_spec_validation_rejects_invalid_inputs() {
    let mut spec = valid_spec();
    spec.schema_version = 2;
    assert_eq!(
        validate_run_spec(&spec),
        Err(ModelError::UnsupportedSchema(2))
    );
    spec = valid_spec();
    spec.run_id = "bad/id".to_string();
    assert!(matches!(
        validate_run_spec(&spec),
        Err(ModelError::UnsafeRunId(_))
    ));
    spec = valid_spec();
    spec.run_id.clear();
    assert!(matches!(
        validate_run_spec(&spec),
        Err(ModelError::UnsafeRunId(_))
    ));
    spec = valid_spec();
    spec.agent_profile_ref.clear();
    assert_eq!(validate_run_spec(&spec), Err(ModelError::MissingAgent));
    spec = valid_spec();
    spec.benchmark.name.clear();
    assert_eq!(validate_run_spec(&spec), Err(ModelError::MissingBenchmark));
    spec = valid_spec();
    spec.benchmark.split.clear();
    assert_eq!(validate_run_spec(&spec), Err(ModelError::MissingBenchmark));
    spec = valid_spec();
    spec.execution.attempts = 0;
    assert_eq!(validate_run_spec(&spec), Err(ModelError::InvalidExecution));
    spec = valid_spec();
    spec.execution.concurrency = 0;
    assert_eq!(validate_run_spec(&spec), Err(ModelError::InvalidExecution));
}

#[test]
fn core_001_benchmark_plan_validation_rejects_empty_task_and_escape_paths() {
    let mut plan = valid_plan();
    plan.tasks[0].task_id = "repo/task".to_string();
    assert_eq!(validate_benchmark_plan(&plan), Ok(()));

    plan.tasks[0].task_id = " ".to_string();
    assert!(matches!(
        validate_benchmark_plan(&plan),
        Err(ModelError::UnsafeTaskId(_))
    ));

    plan = valid_plan();
    plan.tasks[0].artifact_spec.required_paths = vec!["../escape".to_string()];
    assert!(matches!(
        validate_benchmark_plan(&plan),
        Err(ModelError::UnsafeArtifactPath(_))
    ));
}

#[test]
fn core_001_benchmark_plan_validation_checks_runtime_snapshot_pairing() {
    let mut plan = valid_plan();
    plan.task_runtime_snapshots = vec![runtime_snapshot_for(&plan, "task-1")];
    assert_eq!(validate_benchmark_plan(&plan), Ok(()));

    let mut duplicate_task = valid_plan();
    duplicate_task.tasks.push(duplicate_task.tasks[0].clone());
    assert!(matches!(
        validate_benchmark_plan(&duplicate_task),
        Err(ModelError::InvalidTaskRuntimeSnapshot(_))
    ));

    let mut missing = valid_plan();
    missing.tasks.push(missing.tasks[0].clone());
    missing.tasks[1].task_id = "task-2".to_string();
    missing.task_runtime_snapshots = vec![runtime_snapshot_for(&missing, "task-1")];
    assert!(matches!(
        validate_benchmark_plan(&missing),
        Err(ModelError::InvalidTaskRuntimeSnapshot(_))
    ));

    let mut duplicate = valid_plan();
    duplicate.task_runtime_snapshots = vec![
        runtime_snapshot_for(&duplicate, "task-1"),
        runtime_snapshot_for(&duplicate, "task-1"),
    ];
    assert!(matches!(
        validate_benchmark_plan(&duplicate),
        Err(ModelError::InvalidTaskRuntimeSnapshot(_))
    ));

    let mut mismatch = valid_plan();
    mismatch.task_runtime_snapshots = vec![runtime_snapshot_for(&mismatch, "task-1")];
    mismatch.task_runtime_snapshots[0].split = "other".to_string();
    assert!(matches!(
        validate_benchmark_plan(&mismatch),
        Err(ModelError::InvalidTaskRuntimeSnapshot(_))
    ));
}

#[test]
fn core_001_old_benchmark_plan_snapshot_defaults_runtime_snapshots() {
    let mut plan_json = serde_json::to_value(valid_plan()).unwrap();
    plan_json
        .as_object_mut()
        .unwrap()
        .remove("task_runtime_snapshots");
    let plan: crate::BenchmarkPlan = serde_json::from_value(plan_json).unwrap();
    assert!(plan.task_runtime_snapshots.is_empty());
    assert_eq!(validate_benchmark_plan(&plan), Ok(()));
}

fn attempt(class: FailureClass, code: Option<FailureCode>) -> TaskAttemptResult {
    TaskAttemptResult {
        schema_version: 1,
        task_id: "task".to_string(),
        attempt: 1,
        provenance: AttemptProvenance::Original,
        state: if class == FailureClass::None {
            TaskState::Success
        } else {
            TaskState::Failure
        },
        outcome: if class == FailureClass::None {
            Outcome::Success
        } else {
            Outcome::Failure
        },
        failure_class: class,
        failure_code: code,
        health_impact: health_impact_for_failure(class, code),
        benchmark_score: 0.0,
        duration_ms: 10,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::unknown(),
        warnings: Vec::new(),
    }
}

fn runtime_snapshot_for(plan: &crate::BenchmarkPlan, task_id: &str) -> crate::RuntimeTaskSnapshot {
    crate::RuntimeTaskSnapshot {
        benchmark: plan.benchmark.clone(),
        split: plan.split.clone(),
        task_id: task_id.to_string(),
        source_ref: crate::SourceRef {
            benchmark: plan.benchmark.name.clone(),
            upstream_id: task_id.to_string(),
            checksum: "fnv64:test".to_string(),
        },
        upstream_metadata_hash: "fnv64:test".to_string(),
        instruction_hash: "fnv64:instruction".to_string(),
        task_plan_hash: "fnv64:task-plan".to_string(),
        external_runner: None,
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
        execution: ExecutionConfig {
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

fn valid_plan() -> crate::BenchmarkPlan {
    crate::BenchmarkPlan {
        benchmark: crate::BenchmarkIdentity {
            name: "fake".to_string(),
            version: "fixture".to_string(),
        },
        split: "success".to_string(),
        prepared_benchmark_ref: "fixture".to_string(),
        tasks: vec![crate::TaskPlan {
            task_id: "task-1".to_string(),
            instruction: "do it".to_string(),
            workspace_spec: crate::WorkspaceSpec {
                workspace_type: crate::WorkspaceType::Empty,
                target_path: ".".to_string(),
                clean: true,
            },
            sandbox_spec: crate::SandboxSpec {
                image: "host".to_string(),
                mounts: Vec::new(),
                env_vars: Vec::new(),
                network: NetworkPolicy::None,
                privileged: false,
                resource_limits: crate::ResourceHint {
                    cpu_cores: 1,
                    memory_mb: 512,
                },
            },
            verifier_spec: crate::VerifierSpec {
                command: "true".to_string(),
                working_dir: ".".to_string(),
                timeout_sec: 1,
                expected_exit_codes: vec![0],
                environment_mode: crate::VerifierEnvironment::HostProcess,
                output_parser: "exit_code".to_string(),
            },
            artifact_spec: crate::ArtifactSpec {
                base_dir: ".".to_string(),
                globs: Vec::new(),
                required_paths: vec!["result.txt".to_string()],
                max_size_bytes: 1024,
            },
            patch_spec: None,
            external_runner: None,
        }],
        task_runtime_snapshots: Vec::new(),
        run_config_overrides: crate::RunConfigOverrides {
            timeout_sec: None,
            network: None,
        },
        warnings: Vec::new(),
    }
}
