use super::schedule::AttemptWork;
use anyhow::Result;
use harnesslab_core::{
    FailureClass, FailureCode, Outcome, TaskAttemptResult, TaskState, task_dir_name,
};
use harnesslab_infra::{append_event, atomic_write_json, event};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

const AGENT_TIMEOUT_ABORT_THRESHOLD: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RunAbort {
    pub(super) reason: String,
    pub(super) task_id: String,
    pub(super) attempt: u32,
}

#[derive(Debug, Clone)]
pub(super) struct ReportHealth {
    pub(super) status: String,
    pub(super) reason: String,
}

pub(super) fn report_health(run_dir: &Path) -> ReportHealth {
    let Ok(value) = fs::read(run_dir.join("run-health.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .ok_or(())
    else {
        return ReportHealth {
            status: "ok".to_string(),
            reason: String::new(),
        };
    };
    ReportHealth {
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("ok")
            .to_string(),
        reason: value
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    }
}

#[derive(Debug, Clone)]
pub(super) struct RunMonitor {
    run_id: String,
    total_planned: usize,
    completed: usize,
    docker_network_failures: usize,
    agent_timeouts: usize,
    non_timeout_completed: usize,
    successes: usize,
    aborted: Option<RunAbort>,
}

#[derive(Debug, Clone, Serialize)]
struct RunHealthSnapshot<'a> {
    schema_version: u32,
    status: &'a str,
    reason: Option<&'a str>,
    abort_task_id: Option<&'a str>,
    abort_attempt: Option<u32>,
    total_planned: usize,
    completed: usize,
    successes: usize,
    non_timeout_completed: usize,
    docker_network_failures: usize,
    agent_timeouts: usize,
}

impl RunMonitor {
    pub(super) fn new(run_id: impl Into<String>, total_planned: usize) -> Self {
        Self {
            run_id: run_id.into(),
            total_planned,
            completed: 0,
            docker_network_failures: 0,
            agent_timeouts: 0,
            non_timeout_completed: 0,
            successes: 0,
            aborted: None,
        }
    }

    pub(super) fn record_result(
        &mut self,
        run_dir: &Path,
        result: &TaskAttemptResult,
    ) -> Result<Option<RunAbort>> {
        self.completed += 1;
        if result.state == TaskState::Success {
            self.successes += 1;
        }
        if result.failure_code != Some(FailureCode::AgentTimeout) {
            self.non_timeout_completed += 1;
        }
        if result.failure_code == Some(FailureCode::DockerNetworkPoolExhausted) {
            self.docker_network_failures += 1;
            self.abort(
                run_dir,
                result,
                "docker network pool exhausted; benchmark environment is unhealthy",
            )?;
        } else if result.failure_code == Some(FailureCode::AgentTimeout) {
            self.agent_timeouts += 1;
            if self.agent_timeouts >= AGENT_TIMEOUT_ABORT_THRESHOLD
                && self.non_timeout_completed == 0
            {
                self.abort(
                    run_dir,
                    result,
                    "agent timeout threshold reached before any successful task",
                )?;
            }
        }
        self.write_snapshot(run_dir)?;
        Ok(self.aborted.clone())
    }

    pub(super) fn interrupted_results(
        &self,
        run_dir: &Path,
        pending: &[AttemptWork],
    ) -> Result<Vec<TaskAttemptResult>> {
        let Some(abort) = &self.aborted else {
            return Ok(Vec::new());
        };
        let mut interrupted = Vec::new();
        for work in pending {
            let result = TaskAttemptResult {
                schema_version: 1,
                task_id: work.task.task_id.clone(),
                attempt: work.attempt,
                provenance: work.provenance,
                state: TaskState::Interrupted,
                outcome: Outcome::Failure,
                failure_class: FailureClass::Execution,
                failure_code: Some(FailureCode::RunHealthAborted),
                benchmark_score: 0.0,
                duration_ms: 0,
                agent: None,
                evaluation: None,
                patch: None,
                usage: harnesslab_core::UsageRecord::Unknown,
                warnings: Vec::new(),
            };
            let result_path = run_dir
                .join("tasks")
                .join(task_dir_name(&work.task.task_id)?)
                .join("attempts")
                .join(work.attempt.to_string())
                .join("result.json");
            atomic_write_json(&result_path, &result)?;
            append_event(
                &run_dir.join("events.jsonl"),
                &event(
                    &self.run_id,
                    Some(&work.task.task_id),
                    "task_interrupted",
                    &format!(
                        "not scheduled because run health monitor aborted: {}",
                        abort.reason
                    ),
                ),
                &[],
            )?;
            interrupted.push(result);
        }
        Ok(interrupted)
    }

    fn abort(&mut self, run_dir: &Path, result: &TaskAttemptResult, reason: &str) -> Result<()> {
        if self.aborted.is_some() {
            return Ok(());
        }
        self.aborted = Some(RunAbort {
            reason: reason.to_string(),
            task_id: result.task_id.clone(),
            attempt: result.attempt,
        });
        append_event(
            &run_dir.join("events.jsonl"),
            &event(
                &self.run_id,
                Some(&result.task_id),
                "run_health_aborted",
                &format!("{reason}; attempt={}", result.attempt),
            ),
            &[],
        )?;
        Ok(())
    }

    fn write_snapshot(&self, run_dir: &Path) -> Result<()> {
        let reason = self.aborted.as_ref().map(|abort| abort.reason.as_str());
        let abort_task_id = self.aborted.as_ref().map(|abort| abort.task_id.as_str());
        let abort_attempt = self.aborted.as_ref().map(|abort| abort.attempt);
        let snapshot = RunHealthSnapshot {
            schema_version: 1,
            status: if reason.is_some() { "invalid" } else { "ok" },
            reason,
            abort_task_id,
            abort_attempt,
            total_planned: self.total_planned,
            completed: self.completed,
            successes: self.successes,
            non_timeout_completed: self.non_timeout_completed,
            docker_network_failures: self.docker_network_failures,
            agent_timeouts: self.agent_timeouts,
        };
        atomic_write_json(&run_dir.join("run-health.json"), &snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{AttemptProvenance, NetworkPolicy, TaskPlan, UsageRecord};

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
        let health: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(health["status"], "invalid");
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

        let health: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(tmp.path().join("run-health.json")).unwrap(),
        )
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
        }
    }
}
