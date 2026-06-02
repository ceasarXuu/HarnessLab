use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Pending,
    Preparing,
    AgentRunning,
    Evaluating,
    Collecting,
    Success,
    PartialSuccess,
    Failure,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    PartialSuccess,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    None,
    Execution,
    Benchmark,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCode {
    SandboxCreateFailed,
    WorkspacePrepFailed,
    AgentSpawnError,
    AgentTimeout,
    AgentCleanupFailed,
    ExternalRunnerNoProgress,
    ExternalRunnerTimeout,
    AgentSignaled,
    AgentNonzeroExit,
    ArtifactCollectionFailed,
    VerifierTimeout,
    VerifierError,
    EvaluatorError,
    TestFailed,
    NoValidDiff,
    PatchApplyFailed,
    UsageUnknown,
    UsageParserFailed,
    DockerNetworkPoolExhausted,
    RunHealthAborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HealthImpact {
    #[default]
    None,
    Stall,
    EnvironmentUnhealthy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub class: FailureClass,
    pub code: Option<FailureCode>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessRecord {
    pub exit_code: Option<i32>,
    pub termination_reason: TerminationReason,
    pub stdout_path: String,
    pub stderr_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationReason {
    Completed,
    SpawnError,
    Timeout,
    NoProgress,
    Signaled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub exit_code: Option<i32>,
    pub raw_score: f64,
    pub stdout_path: String,
    pub stderr_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchRecord {
    pub diff_path: String,
    pub prediction_path: Option<String>,
    pub status: PatchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchStatus {
    NotApplicable,
    Empty,
    Captured,
    ApplyFailed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttemptResult {
    pub schema_version: u32,
    pub task_id: String,
    pub attempt: u32,
    #[serde(default)]
    pub provenance: AttemptProvenance,
    pub state: TaskState,
    pub outcome: Outcome,
    pub failure_class: FailureClass,
    pub failure_code: Option<FailureCode>,
    #[serde(default)]
    pub health_impact: HealthImpact,
    pub benchmark_score: f64,
    pub duration_ms: u64,
    pub agent: Option<ProcessRecord>,
    pub evaluation: Option<EvaluationRecord>,
    pub patch: Option<PatchRecord>,
    pub usage: crate::UsageRecord,
    pub warnings: Vec<FailureCode>,
}

pub fn health_impact_for_failure(
    failure_class: FailureClass,
    failure_code: Option<FailureCode>,
) -> HealthImpact {
    match (failure_class, failure_code) {
        (_, Some(FailureCode::DockerNetworkPoolExhausted)) => HealthImpact::EnvironmentUnhealthy,
        (
            FailureClass::Execution,
            Some(
                FailureCode::AgentTimeout
                | FailureCode::ExternalRunnerNoProgress
                | FailureCode::ExternalRunnerTimeout,
            ),
        ) => HealthImpact::Stall,
        _ => HealthImpact::None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AttemptProvenance {
    #[default]
    Original,
    Resumed,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResults {
    pub schema_version: u32,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    pub tasks: Vec<TaskAttemptResult>,
    pub summary: RunSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub total_tasks: usize,
    pub success: usize,
    pub partial_success: usize,
    pub benchmark_failure: usize,
    pub execution_failure: usize,
    pub interrupted: usize,
    pub total_duration_ms: u64,
    pub total_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSpec {
    pub schema_version: u32,
    pub run_id: String,
    pub created_at: String,
    pub agent_profile_ref: String,
    pub benchmark: BenchmarkRef,
    pub execution: ExecutionConfig,
    pub paths: RunPaths,
    #[serde(default)]
    pub replay_source_run_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkRef {
    pub name: String,
    pub version: String,
    pub split: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub concurrency: usize,
    pub attempts: u32,
    pub network: NetworkPolicy,
    pub timeout_sec: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    Full,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunPaths {
    pub run_dir: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ModelError {
    #[error("unsupported schema_version {0}")]
    UnsupportedSchema(u32),
    #[error("unsafe run_id {0}")]
    UnsafeRunId(String),
    #[error("agent_profile_ref is required")]
    MissingAgent,
    #[error("benchmark name and split are required")]
    MissingBenchmark,
    #[error("unsafe task_id {0}")]
    UnsafeTaskId(String),
    #[error("unsafe artifact path {0}")]
    UnsafeArtifactPath(String),
    #[error("attempts and concurrency must be >= 1")]
    InvalidExecution,
    #[error("invalid transition {from:?} -> {to:?}")]
    InvalidTransition { from: TaskState, to: TaskState },
}

pub fn validate_run_spec(spec: &RunSpec) -> Result<(), ModelError> {
    if spec.schema_version != 1 {
        return Err(ModelError::UnsupportedSchema(spec.schema_version));
    }
    if !is_filesystem_safe_id(&spec.run_id) {
        return Err(ModelError::UnsafeRunId(spec.run_id.clone()));
    }
    if spec.agent_profile_ref.trim().is_empty() {
        return Err(ModelError::MissingAgent);
    }
    if spec.benchmark.name.trim().is_empty() || spec.benchmark.split.trim().is_empty() {
        return Err(ModelError::MissingBenchmark);
    }
    if spec.execution.attempts == 0 || spec.execution.concurrency == 0 {
        return Err(ModelError::InvalidExecution);
    }
    Ok(())
}

pub fn validate_benchmark_plan(plan: &crate::BenchmarkPlan) -> Result<(), ModelError> {
    for task in &plan.tasks {
        if crate::task_dir_name(&task.task_id).is_err() {
            return Err(ModelError::UnsafeTaskId(task.task_id.clone()));
        }
        for path in &task.artifact_spec.required_paths {
            if crate::report_artifact_path(path).is_err() {
                return Err(ModelError::UnsafeArtifactPath(path.clone()));
            }
        }
        if let Some(patch) = &task.patch_spec {
            for path in [&patch.diff_path, &patch.prediction_path] {
                if crate::report_artifact_path(path).is_err() {
                    return Err(ModelError::UnsafeArtifactPath(path.clone()));
                }
            }
        }
    }
    Ok(())
}

pub fn ensure_transition(from: TaskState, to: TaskState) -> Result<(), ModelError> {
    if can_transition(from, to) {
        Ok(())
    } else {
        Err(ModelError::InvalidTransition { from, to })
    }
}

pub fn can_transition(from: TaskState, to: TaskState) -> bool {
    matches!(
        (from, to),
        (TaskState::Pending, TaskState::Preparing)
            | (TaskState::Preparing, TaskState::AgentRunning)
            | (TaskState::AgentRunning, TaskState::Evaluating)
            | (TaskState::Evaluating, TaskState::Collecting)
            | (TaskState::Collecting, TaskState::Success)
            | (TaskState::Collecting, TaskState::PartialSuccess)
            | (TaskState::Collecting, TaskState::Failure)
            | (TaskState::Preparing, TaskState::Failure)
            | (TaskState::AgentRunning, TaskState::Failure)
            | (TaskState::Evaluating, TaskState::Failure)
            | (TaskState::Preparing, TaskState::Interrupted)
            | (TaskState::AgentRunning, TaskState::Interrupted)
            | (TaskState::Evaluating, TaskState::Interrupted)
            | (TaskState::Collecting, TaskState::Interrupted)
    )
}

pub fn classify_agent_process(result: &ProcessRecord) -> Failure {
    match result.termination_reason {
        TerminationReason::SpawnError => execution(FailureCode::AgentSpawnError),
        TerminationReason::Timeout => execution(FailureCode::AgentTimeout),
        TerminationReason::NoProgress => execution(FailureCode::AgentTimeout),
        TerminationReason::Signaled => execution(FailureCode::AgentSignaled),
        TerminationReason::Completed if result.exit_code.unwrap_or(1) != 0 => {
            execution(FailureCode::AgentNonzeroExit)
        }
        TerminationReason::Completed => none(),
    }
}

pub fn classify_evaluation_process(result: &EvaluationRecord) -> Failure {
    if result.exit_code == Some(0) && result.raw_score >= 1.0 {
        none()
    } else if result.exit_code == Some(0) {
        benchmark(FailureCode::TestFailed)
    } else {
        benchmark(FailureCode::VerifierError)
    }
}

pub fn derive_exit_code(results: &[TaskAttemptResult], run_level_failed: bool) -> i32 {
    if run_level_failed || results.is_empty() {
        return 3;
    }
    let effective = effective_attempts(results);
    if effective
        .iter()
        .any(|result| result.state == TaskState::Interrupted)
    {
        return 130;
    }
    if effective
        .iter()
        .any(|result| result.failure_class == FailureClass::Execution)
    {
        return 1;
    }
    0
}

pub fn summarize_results(run_id: impl Into<String>, tasks: Vec<TaskAttemptResult>) -> RunResults {
    let effective = effective_attempts(&tasks);
    let mut summary = RunSummary {
        total_tasks: effective.len(),
        success: 0,
        partial_success: 0,
        benchmark_failure: 0,
        execution_failure: 0,
        interrupted: 0,
        total_duration_ms: 0,
        total_score: 0.0,
    };
    for task in effective {
        match task.outcome {
            Outcome::Success => summary.success += 1,
            Outcome::PartialSuccess => summary.partial_success += 1,
            Outcome::Failure => {}
        }
        match task.failure_class {
            FailureClass::Benchmark => summary.benchmark_failure += 1,
            FailureClass::Execution => summary.execution_failure += 1,
            _ => {}
        }
        if task.state == TaskState::Interrupted {
            summary.interrupted += 1;
        }
        summary.total_duration_ms += task.duration_ms;
        summary.total_score += task.benchmark_score;
    }
    RunResults {
        schema_version: 1,
        run_id: run_id.into(),
        report_path: None,
        tasks,
        summary,
    }
}

fn effective_attempts(results: &[TaskAttemptResult]) -> Vec<&TaskAttemptResult> {
    let mut latest = BTreeMap::new();
    for result in results {
        latest
            .entry(result.task_id.as_str())
            .and_modify(|current: &mut &TaskAttemptResult| {
                if result.attempt > current.attempt {
                    *current = result;
                }
            })
            .or_insert(result);
    }
    latest.into_values().collect()
}

pub fn metadata() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn is_filesystem_safe_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn none() -> Failure {
    Failure {
        class: FailureClass::None,
        code: None,
        message: "ok".to_string(),
    }
}

fn execution(code: FailureCode) -> Failure {
    Failure {
        class: FailureClass::Execution,
        code: Some(code),
        message: format!("{code:?}"),
    }
}

fn benchmark(code: FailureCode) -> Failure {
    Failure {
        class: FailureClass::Benchmark,
        code: Some(code),
        message: format!("{code:?}"),
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
