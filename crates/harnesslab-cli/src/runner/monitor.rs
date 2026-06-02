use super::schedule::AttemptWork;
use anyhow::Result;
use harnesslab_core::{
    FailureCode, HealthImpact, Outcome, TaskAttemptResult, TaskState, task_dir_name,
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
    environment_failures: usize,
    docker_network_failures: usize,
    agent_timeouts: usize,
    external_runner_no_progress: usize,
    external_runner_timeouts: usize,
    execution_stalls: usize,
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
    environment_failures: usize,
    docker_network_failures: usize,
    agent_timeouts: usize,
    external_runner_no_progress: usize,
    external_runner_timeouts: usize,
    execution_stalls: usize,
}

impl RunMonitor {
    pub(super) fn new(run_id: impl Into<String>, total_planned: usize) -> Self {
        Self {
            run_id: run_id.into(),
            total_planned,
            completed: 0,
            environment_failures: 0,
            docker_network_failures: 0,
            agent_timeouts: 0,
            external_runner_no_progress: 0,
            external_runner_timeouts: 0,
            execution_stalls: 0,
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
        if result.health_impact != HealthImpact::Stall {
            self.non_timeout_completed += 1;
        }
        if result.health_impact == HealthImpact::EnvironmentUnhealthy {
            self.environment_failures += 1;
            if result.failure_code == Some(FailureCode::DockerNetworkPoolExhausted) {
                self.docker_network_failures += 1;
            }
            self.abort(
                run_dir,
                result,
                environment_abort_reason(result.failure_code),
            )?;
        } else if result.health_impact == HealthImpact::Stall {
            self.execution_stalls += 1;
            if result.failure_code == Some(FailureCode::AgentTimeout) {
                self.agent_timeouts += 1;
            }
            if result.failure_code == Some(FailureCode::ExternalRunnerNoProgress) {
                self.external_runner_no_progress += 1;
            }
            if result.failure_code == Some(FailureCode::ExternalRunnerTimeout) {
                self.external_runner_timeouts += 1;
                self.abort(
                    run_dir,
                    result,
                    "external runner hard timeout; benchmark runner budget or setup is unhealthy",
                )?;
            }
            if self.execution_stalls >= AGENT_TIMEOUT_ABORT_THRESHOLD
                && self.non_timeout_completed == 0
            {
                let reason = if self.agent_timeouts == self.execution_stalls {
                    "agent timeout threshold reached before any successful task"
                } else {
                    "execution stall threshold reached before any successful task"
                };
                self.abort(run_dir, result, reason)?;
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
                failure_class: harnesslab_core::FailureClass::Execution,
                failure_code: Some(FailureCode::RunHealthAborted),
                health_impact: HealthImpact::None,
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
            environment_failures: self.environment_failures,
            docker_network_failures: self.docker_network_failures,
            agent_timeouts: self.agent_timeouts,
            external_runner_no_progress: self.external_runner_no_progress,
            external_runner_timeouts: self.external_runner_timeouts,
            execution_stalls: self.execution_stalls,
        };
        atomic_write_json(&run_dir.join("run-health.json"), &snapshot)
    }
}

fn environment_abort_reason(code: Option<FailureCode>) -> &'static str {
    match code {
        Some(FailureCode::DockerNetworkPoolExhausted) => {
            "docker network pool exhausted; benchmark environment is unhealthy"
        }
        Some(FailureCode::ExternalRunnerSetupFailed) => {
            "external runner setup failed; benchmark environment is unhealthy"
        }
        _ => "benchmark environment is unhealthy",
    }
}

#[cfg(test)]
#[path = "monitor_tests.rs"]
mod tests;
