use anyhow::Result;
use harnesslab_core::{EvaluationRecord, FailureClass, FailureCode, UsageRecord};
use serde_json::Value;
use std::fs;
use std::path::Path;

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
    let evaluation = EvaluationRecord {
        exit_code: Some(0),
        raw_score: score,
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    };
    let usage = terminal_bench_usage(&value);
    if score >= 1.0 {
        Ok((evaluation, usage, FailureClass::None, None, score))
    } else if let Some((failure_class, failure_code)) = terminal_bench_failure(&value, task_id) {
        Ok((evaluation, usage, failure_class, Some(failure_code), score))
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

fn warning_code_for_success(result: &Value) -> Option<FailureCode> {
    failure_mode_code(result.get("failure_mode").and_then(Value::as_str)?).map(|(_, code)| code)
}

fn failure_mode_code(mode: &str) -> Option<(FailureClass, FailureCode)> {
    match mode {
        "agent_timeout" => Some((FailureClass::Benchmark, FailureCode::AgentTimeout)),
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
