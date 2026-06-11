use super::*;
use harnesslab_core::{
    AttemptProvenance, FailureClass, HealthImpact, NetworkPolicy, TaskPlan, UsageRecord,
    health_impact_for_failure,
};

#[test]
fn monitor_aborts_immediately_on_docker_network_pool_exhaustion() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 2);
    let result = attempt(FailureCode::DockerNetworkPoolExhausted);

    let abort = monitor.record_result(tmp.path(), &result).unwrap();

    assert_eq!(
        abort.unwrap().reason,
        "docker network pool exhausted; benchmark environment is unhealthy"
    );
    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["status"], "invalid");
    assert_eq!(health["environment_failures"], 1);
    assert_eq!(health["docker_network_failures"], 1);
}

#[test]
fn monitor_aborts_immediately_on_external_runner_setup_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 2);
    let result = attempt(FailureCode::ExternalRunnerSetupFailed);

    let abort = monitor.record_result(tmp.path(), &result).unwrap();

    assert_eq!(
        abort.unwrap().reason,
        "external runner setup failed; benchmark environment is unhealthy"
    );
    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["status"], "invalid");
    assert_eq!(health["environment_failures"], 1);
    assert_eq!(health["docker_network_failures"], 0);
}

#[test]
fn monitor_writes_interrupted_results_for_unscheduled_work() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 2);
    monitor
        .record_result(
            tmp.path(),
            &attempt(FailureCode::DockerNetworkPoolExhausted),
        )
        .unwrap();

    let interrupted = monitor
        .interrupted_results(
            tmp.path(),
            &[AttemptWork {
                task: task("task-b"),
                task_runtime_snapshot: None,
                attempt: 1,
                provenance: AttemptProvenance::Original,
            }],
        )
        .unwrap();

    assert_eq!(interrupted.len(), 1);
    assert_eq!(interrupted[0].state, TaskState::Interrupted);
    assert_eq!(
        interrupted[0].failure_code,
        Some(FailureCode::RunHealthAborted)
    );
    assert!(
        tmp.path()
            .join("tasks/task-b/attempts/1/result.json")
            .exists()
    );
}

#[test]
fn monitor_aborts_after_timeout_threshold_before_any_success() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 5);

    for index in 0..4 {
        let abort = monitor
            .record_result(
                tmp.path(),
                &attempt_with_task(
                    &format!("timeout-{index}"),
                    FailureCode::AgentTimeout,
                    TaskState::Failure,
                ),
            )
            .unwrap();
        assert!(abort.is_none());
    }

    let abort = monitor
        .record_result(
            tmp.path(),
            &attempt_with_task("timeout-4", FailureCode::AgentTimeout, TaskState::Failure),
        )
        .unwrap()
        .unwrap();

    assert_eq!(abort.task_id, "timeout-4");
    assert_eq!(
        abort.reason,
        "agent timeout threshold reached before any successful task"
    );
}

#[test]
fn monitor_aborts_after_external_runner_no_progress_threshold_before_any_success() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 5);

    for index in 0..4 {
        let abort = monitor
            .record_result(
                tmp.path(),
                &attempt_with_task(
                    &format!("stall-{index}"),
                    FailureCode::ExternalRunnerNoProgress,
                    TaskState::Failure,
                ),
            )
            .unwrap();
        assert!(abort.is_none());
    }

    let abort = monitor
        .record_result(
            tmp.path(),
            &attempt_with_task(
                "stall-4",
                FailureCode::ExternalRunnerNoProgress,
                TaskState::Failure,
            ),
        )
        .unwrap()
        .unwrap();

    assert_eq!(abort.task_id, "stall-4");
    assert_eq!(
        abort.reason,
        "execution stall threshold reached before any successful task"
    );
    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["external_runner_no_progress"], 5);
    assert_eq!(health["execution_stalls"], 5);
    assert_eq!(health["non_timeout_completed"], 0);
}

#[test]
fn monitor_aborts_immediately_on_external_runner_timeout() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 5);
    monitor
        .record_result(
            tmp.path(),
            &attempt_with_class("success", None, FailureClass::None, TaskState::Success),
        )
        .unwrap();

    let abort = monitor
        .record_result(
            tmp.path(),
            &attempt_with_task(
                "runner-timeout",
                FailureCode::ExternalRunnerTimeout,
                TaskState::Failure,
            ),
        )
        .unwrap()
        .unwrap();

    assert_eq!(abort.task_id, "runner-timeout");
    assert_eq!(
        abort.reason,
        "external runner hard timeout; benchmark runner budget or setup is unhealthy"
    );
    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["external_runner_timeouts"], 1);
    assert_eq!(health["execution_stalls"], 1);
    assert_eq!(health["agent_timeouts"], 0);
}

#[test]
fn monitor_aborts_after_generic_execution_stall_health_impact() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 5);

    for index in 0..4 {
        let mut result = attempt_with_class(
            &format!("generic-stall-{index}"),
            Some(FailureCode::EvaluatorError),
            FailureClass::Execution,
            TaskState::Failure,
        );
        result.health_impact = HealthImpact::Stall;
        assert!(
            monitor
                .record_result(tmp.path(), &result)
                .unwrap()
                .is_none()
        );
    }

    let mut result = attempt_with_class(
        "generic-stall-4",
        Some(FailureCode::EvaluatorError),
        FailureClass::Execution,
        TaskState::Failure,
    );
    result.health_impact = HealthImpact::Stall;
    let abort = monitor.record_result(tmp.path(), &result).unwrap().unwrap();

    assert_eq!(abort.task_id, "generic-stall-4");
    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["status"], "invalid");
    assert_eq!(
        health["reason"],
        "execution stall threshold reached before any successful task"
    );
    assert_eq!(health["execution_stalls"], 5);
    assert_eq!(health["agent_timeouts"], 0);
}

#[test]
fn monitor_does_not_abort_timeout_threshold_after_success() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 6);
    monitor
        .record_result(
            tmp.path(),
            &attempt_with_task("success", FailureCode::TestFailed, TaskState::Success),
        )
        .unwrap();

    for index in 0..5 {
        let abort = monitor
            .record_result(
                tmp.path(),
                &attempt_with_task(
                    &format!("timeout-{index}"),
                    FailureCode::AgentTimeout,
                    TaskState::Failure,
                ),
            )
            .unwrap();
        assert!(abort.is_none());
    }

    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["status"], "ok");
}

#[test]
fn monitor_does_not_abort_timeout_threshold_after_non_timeout_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 6);
    monitor
        .record_result(
            tmp.path(),
            &attempt_with_class(
                "test-failed",
                Some(FailureCode::TestFailed),
                FailureClass::Benchmark,
                TaskState::Failure,
            ),
        )
        .unwrap();

    for index in 0..5 {
        let abort = monitor
            .record_result(
                tmp.path(),
                &attempt_with_task(
                    &format!("timeout-{index}"),
                    FailureCode::AgentTimeout,
                    TaskState::Failure,
                ),
            )
            .unwrap();
        assert!(abort.is_none());
    }
}

#[test]
fn monitor_treats_benchmark_agent_timeout_as_completed_verdict() {
    let tmp = tempfile::tempdir().unwrap();
    let mut monitor = RunMonitor::new("run-1", 5);

    for index in 0..5 {
        let abort = monitor
            .record_result(
                tmp.path(),
                &attempt_with_class(
                    &format!("agent-timeout-{index}"),
                    Some(FailureCode::AgentTimeout),
                    FailureClass::Benchmark,
                    TaskState::Failure,
                ),
            )
            .unwrap();
        assert!(abort.is_none());
    }

    let health: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap())
            .unwrap();
    assert_eq!(health["status"], "ok");
    assert_eq!(health["agent_timeouts"], 0);
    assert_eq!(health["non_timeout_completed"], 5);
}

fn attempt(code: FailureCode) -> TaskAttemptResult {
    attempt_with_task("task-a", code, TaskState::Failure)
}

fn attempt_with_task(task_id: &str, code: FailureCode, state: TaskState) -> TaskAttemptResult {
    attempt_with_class(
        task_id,
        (state != TaskState::Success).then_some(code),
        if state == TaskState::Success {
            FailureClass::None
        } else {
            FailureClass::Execution
        },
        state,
    )
}

fn attempt_with_class(
    task_id: &str,
    code: Option<FailureCode>,
    class: FailureClass,
    state: TaskState,
) -> TaskAttemptResult {
    TaskAttemptResult {
        schema_version: 1,
        task_id: task_id.to_string(),
        attempt: 1,
        provenance: AttemptProvenance::Original,
        state,
        outcome: if state == TaskState::Success {
            Outcome::Success
        } else {
            Outcome::Failure
        },
        failure_class: class,
        failure_code: code,
        health_impact: health_impact_for_failure(class, code),
        benchmark_score: f64::from(state == TaskState::Success),
        duration_ms: 1,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: Vec::new(),
    }
}

fn task(task_id: &str) -> TaskPlan {
    TaskPlan {
        task_id: task_id.to_string(),
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
        runtime_binding: None,
    }
}
