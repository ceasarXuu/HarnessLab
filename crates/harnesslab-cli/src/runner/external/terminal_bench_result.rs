use super::ExternalTaskExecution;
use anyhow::Result;
use harnesslab_core::{
    EvaluationRecord, FailureClass, FailureCode, Outcome, TaskAttemptResult, TaskState,
    UsageRecord, health_impact_for_failure,
};
use harnesslab_infra::{append_event, atomic_write_json, event};
use serde_json::Value;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const ADAPTER_TIMEOUT_MARKER: &str = "agent command timed out;";
const ADAPTER_CLEANUP_SUCCEEDED_MARKER: &str = "succeeded=True";
const ADAPTER_CLEANUP_FAILED_MARKER: &str = "succeeded=False";
const ADAPTER_LIVE_CHILDREN_MARKER: &str = "left live child processes";
const MAX_LOG_BYTES: u64 = 256 * 1024;

pub(super) fn parse_terminal_bench_result(
    attempt_dir: &Path,
    result_path: &Path,
    task_id: &str,
) -> Result<(
    EvaluationRecord,
    UsageRecord,
    FailureClass,
    Option<FailureCode>,
    f64,
)> {
    let value = read_result_json(result_path)?;
    let score = value
        .get("accuracy")
        .and_then(Value::as_f64)
        .or_else(|| resolved_score(&value))
        .unwrap_or(0.0);
    write_verifier_logs(attempt_dir, result_path, &value, "")?;
    let mut evaluation = EvaluationRecord {
        exit_code: Some(0),
        raw_score: score,
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    };
    let usage = terminal_bench_usage(&value);
    if adapter_cleanup_failed(result_path) {
        evaluation.raw_score = 0.0;
        Ok((
            evaluation,
            usage,
            FailureClass::Execution,
            Some(FailureCode::AgentCleanupFailed),
            0.0,
        ))
    } else if score >= 1.0 {
        Ok((evaluation, usage, FailureClass::None, None, score))
    } else if let Some((failure_class, failure_code)) = terminal_bench_failure(&value, task_id) {
        Ok((evaluation, usage, failure_class, Some(failure_code), score))
    } else if adapter_agent_timeout_cleanup_succeeded(result_path) {
        Ok((
            evaluation,
            usage,
            FailureClass::Benchmark,
            Some(FailureCode::AgentTimeout),
            score,
        ))
    } else {
        Ok((
            evaluation,
            usage,
            FailureClass::Benchmark,
            Some(FailureCode::TestFailed),
            score,
        ))
    }
}

pub(super) fn missing_evaluation(
    attempt_dir: &Path,
    result_path: &Path,
    reason: &str,
) -> Result<EvaluationRecord> {
    let message = format!(
        "terminal-bench official results unavailable at {}: {reason}",
        result_path.display()
    );
    write_verifier_logs(attempt_dir, result_path, &Value::Null, &message)?;
    Ok(EvaluationRecord {
        exit_code: None,
        raw_score: 0.0,
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    })
}

pub(super) fn setup_failed_result(
    ctx: &ExternalTaskExecution<'_>,
    result_path: &Path,
    reason: &str,
) -> Result<TaskAttemptResult> {
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_setup_failed",
            reason,
        ),
        &[],
    )?;
    let evaluation = missing_evaluation(ctx.attempt_dir, result_path, reason)?;
    let failure_class = FailureClass::Execution;
    let failure_code = Some(FailureCode::ExternalRunnerSetupFailed);
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: TaskState::Failure,
        outcome: Outcome::Failure,
        failure_class,
        failure_code,
        health_impact: health_impact_for_failure(failure_class, failure_code),
        benchmark_score: 0.0,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: None,
        evaluation: Some(evaluation),
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: Vec::new(),
    };
    write_task_result(ctx, &result)?;
    Ok(result)
}

pub(super) fn write_task_result(
    ctx: &ExternalTaskExecution<'_>,
    result: &TaskAttemptResult,
) -> Result<()> {
    let task_dir = harnesslab_core::task_dir_name(&ctx.task.task_id)?;
    atomic_write_json(
        &ctx.run_dir
            .join("tasks")
            .join(task_dir)
            .join("attempts")
            .join(ctx.attempt.to_string())
            .join("result.json"),
        result,
    )
}

pub(super) fn read_result_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) fn terminal_bench_result_warnings(
    result_path: &Path,
    task_id: &str,
    official_failure_class: FailureClass,
) -> Vec<FailureCode> {
    let mut warnings = Vec::new();
    if official_failure_class != FailureClass::None {
        return warnings;
    }
    let Ok(value) = read_result_json(result_path) else {
        return warnings;
    };
    let Some(results) = value.get("results").and_then(Value::as_array) else {
        return warnings;
    };
    warnings.extend(
        results
            .iter()
            .filter(|result| result.get("task_id").and_then(Value::as_str) == Some(task_id))
            .filter_map(warning_code_for_success),
    );
    warnings
}

fn write_verifier_logs(
    attempt_dir: &Path,
    result_path: &Path,
    value: &Value,
    stderr: &str,
) -> Result<()> {
    let verifier_dir = attempt_dir.join("verifier");
    fs::create_dir_all(&verifier_dir)?;
    let mut stdout = format!("official_results_path={}\n", result_path.display());
    if !value.is_null() {
        stdout.push_str(&serde_json::to_string_pretty(value)?);
        stdout.push('\n');
    }
    fs::write(verifier_dir.join("stdout.log"), stdout)?;
    fs::write(verifier_dir.join("stderr.log"), stderr)?;
    Ok(())
}

fn resolved_score(value: &Value) -> Option<f64> {
    let resolved = value.get("n_resolved")?.as_f64()?;
    let unresolved = value.get("n_unresolved")?.as_f64()?;
    let total = resolved + unresolved;
    (total > 0.0).then_some(resolved / total)
}

fn terminal_bench_failure(value: &Value, task_id: &str) -> Option<(FailureClass, FailureCode)> {
    for result in value.get("results").and_then(Value::as_array)? {
        if result.get("task_id").and_then(Value::as_str) != Some(task_id) {
            continue;
        }
        return failure_mode_code(result.get("failure_mode").and_then(Value::as_str)?);
    }
    None
}

fn adapter_agent_timeout_cleanup_succeeded(result_path: &Path) -> bool {
    let root = result_path.parent().unwrap_or(result_path);
    find_agent_error_logs(root).into_iter().any(|path| {
        read_log_tail(&path)
            .map(|content| {
                content.contains(ADAPTER_TIMEOUT_MARKER)
                    && content.contains(ADAPTER_CLEANUP_SUCCEEDED_MARKER)
            })
            .unwrap_or(false)
    })
}

fn adapter_cleanup_failed(result_path: &Path) -> bool {
    let root = result_path.parent().unwrap_or(result_path);
    find_agent_logs(root).into_iter().any(|path| {
        read_log_tail(&path)
            .map(|content| {
                content.contains(ADAPTER_CLEANUP_FAILED_MARKER)
                    || content.contains(ADAPTER_LIVE_CHILDREN_MARKER)
            })
            .unwrap_or(false)
    })
}

fn find_agent_error_logs(root: &Path) -> Vec<PathBuf> {
    find_agent_logs(root)
        .into_iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("agent_error.log"))
        .collect()
}

fn find_agent_logs(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut logs = Vec::new();
    while let Some(path) = pending.pop() {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                pending.push(entry.path());
            }
        } else if matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("agent_error.log" | "agent_cleanup.log")
        ) {
            logs.push(path);
        }
    }
    logs
}

fn read_log_tail(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > MAX_LOG_BYTES {
        file.seek(SeekFrom::Start(len - MAX_LOG_BYTES))?;
    }
    let mut bytes = Vec::new();
    file.take(MAX_LOG_BYTES).read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn warning_code_for_success(result: &Value) -> Option<FailureCode> {
    failure_mode_code(result.get("failure_mode").and_then(Value::as_str)?).map(|(_, code)| code)
}

fn failure_mode_code(mode: &str) -> Option<(FailureClass, FailureCode)> {
    match mode {
        "agent_timeout" => Some((FailureClass::Benchmark, FailureCode::AgentTimeout)),
        "parse_error" => Some((FailureClass::Benchmark, FailureCode::AgentOutputParseError)),
        "test_timeout" => Some((FailureClass::Benchmark, FailureCode::VerifierTimeout)),
        _ => None,
    }
}

fn terminal_bench_usage(value: &Value) -> UsageRecord {
    let mut input_tokens = 0;
    let mut output_tokens = 0;
    let Some(results) = value.get("results").and_then(Value::as_array) else {
        return UsageRecord::Unknown;
    };
    for result in results {
        input_tokens += result
            .get("total_input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        output_tokens += result
            .get("total_output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
    }
    UsageRecord::Parsed {
        input_tokens,
        output_tokens,
        total_tokens: input_tokens + output_tokens,
        cost_usd: None,
    }
}
