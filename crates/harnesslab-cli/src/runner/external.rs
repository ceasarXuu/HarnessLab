use anyhow::{Result, bail};
use harnesslab_core::{
    AgentKind, AgentProfile, AttemptProvenance, EvaluationRecord, ExternalRunnerKind, FailureClass,
    FailureCode, InputMode, Outcome, ProcessRecord, RunSpec, TaskAttemptResult, TaskPlan,
    TaskState, UsageRecord, classify_agent_process,
};
use harnesslab_infra::{ExecSpec, HostProcessExecutor, append_event, atomic_write_json, event};
use serde_json::Value;
use std::fs;
use std::path::Path;

mod log_scan;
mod swe_bench_pro;

pub(super) fn is_external_task(task: &TaskPlan) -> bool {
    task.external_runner.is_some()
}

pub(super) fn validate_profile_for_plan(profile: &AgentProfile, tasks: &[TaskPlan]) -> Result<()> {
    for task in tasks {
        let Some(runner) = &task.external_runner else {
            continue;
        };
        match runner.kind {
            ExternalRunnerKind::TerminalBench => {
                let _ = terminal_bench_agent(profile)?;
            }
            ExternalRunnerKind::SweBenchPro => {}
        }
    }
    Ok(())
}

pub(super) struct ExternalTaskExecution<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) profile: &'a AgentProfile,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) task: &'a TaskPlan,
    pub(super) attempt: u32,
    pub(super) provenance: AttemptProvenance,
    pub(super) attempt_dir: &'a Path,
    pub(super) started: std::time::Instant,
}

pub(super) fn execute_external_task(ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
    let Some(runner) = &ctx.task.external_runner else {
        bail!("external task missing runner spec");
    };
    match runner.kind {
        ExternalRunnerKind::TerminalBench => {
            execute_terminal_bench(&ctx, Path::new(&runner.dataset_path))
        }
        ExternalRunnerKind::SweBenchPro => swe_bench_pro::execute(
            &ctx,
            Path::new(&runner.dataset_path),
            runner.source_path.as_deref().map(Path::new),
        ),
    }
}

fn execute_terminal_bench(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
) -> Result<TaskAttemptResult> {
    let attempt_root = fs::canonicalize(ctx.attempt_dir)?;
    let output_root = attempt_root.join("official/terminal-bench");
    let official_run_id = official_run_id(ctx.spec, ctx.task, ctx.attempt);
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_started",
            &format!(
                "terminal-bench dataset={} official_run_id={} output={}",
                dataset_path.display(),
                official_run_id,
                output_root.display()
            ),
        ),
        &[],
    )?;
    let agent = terminal_bench_agent(ctx.profile)?;
    let command = terminal_bench_command(
        dataset_path,
        &agent,
        &output_root,
        &official_run_id,
        ctx.profile,
        ctx,
    );
    let report_command = terminal_bench_command(
        dataset_path,
        &agent,
        &output_root,
        &official_run_id,
        ctx.report_profile,
        ctx,
    );
    write_external_command_snapshot(ctx.attempt_dir, ctx.report_profile, &report_command)?;
    let process = normalize_agent_paths(HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: ctx.attempt_dir.to_path_buf(),
        timeout_sec: terminal_bench_timeout_values(
            ctx.spec.execution.timeout_sec,
            ctx.profile.timeout_sec,
            ctx.task.verifier_spec.timeout_sec,
        )
        .2,
        stdout_path: ctx.attempt_dir.join("agent/stdout.log"),
        stderr_path: ctx.attempt_dir.join("agent/stderr.log"),
    })?);
    let result_path = output_root.join(&official_run_id).join("results.json");
    let parsed_result = parse_terminal_bench_result(ctx.attempt_dir, &result_path);
    let agent_failure = classify_agent_process(&process);
    let infra_failure = log_scan::terminal_bench_infra_failure(ctx.attempt_dir);
    let (evaluation, usage, mut failure_class, mut failure_code, mut score) = match parsed_result {
        Ok(parsed) => parsed,
        Err(error) => (
            {
                append_event(
                    &ctx.run_dir.join("events.jsonl"),
                    &event(
                        &ctx.spec.run_id,
                        Some(&ctx.task.task_id),
                        "external_result_parse_failed",
                        &format!("terminal-bench results parse failed: {error}"),
                    ),
                    &[],
                )?;
                missing_evaluation(ctx.attempt_dir, &result_path, &error.to_string())?
            },
            UsageRecord::Unknown,
            if agent_failure.class == FailureClass::Execution {
                agent_failure.class
            } else {
                FailureClass::Execution
            },
            agent_failure.code.or(Some(FailureCode::EvaluatorError)),
            0.0,
        ),
    };
    if let Some(code) = infra_failure {
        failure_class = FailureClass::Execution;
        failure_code = Some(code);
        score = 0.0;
    }
    let mut warnings = Vec::new();
    if matches!(usage, UsageRecord::Unknown) {
        warnings.push(FailureCode::UsageUnknown);
    }
    if agent_failure.class == FailureClass::Execution
        && failure_class != FailureClass::Execution
        && let Some(code) = agent_failure.code
    {
        warnings.push(code);
    }
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: if failure_class == FailureClass::None {
            TaskState::Success
        } else {
            TaskState::Failure
        },
        outcome: if failure_class == FailureClass::None {
            Outcome::Success
        } else {
            Outcome::Failure
        },
        failure_class,
        failure_code,
        benchmark_score: score,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: Some(process),
        evaluation: Some(evaluation),
        patch: None,
        usage,
        warnings,
    };
    let task_dir = harnesslab_core::task_dir_name(&ctx.task.task_id)?;
    atomic_write_json(
        &ctx.run_dir
            .join("tasks")
            .join(task_dir)
            .join("attempts")
            .join(ctx.attempt.to_string())
            .join("result.json"),
        &result,
    )?;
    Ok(result)
}

pub(super) fn write_external_command_snapshot(
    attempt_dir: &Path,
    profile: &AgentProfile,
    command: &str,
) -> Result<()> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::write(
        agent_dir.join("command.txt"),
        format!(
            "template={}\nrendered={}\ninput_mode=external\n",
            profile.command, command
        ),
    )?;
    Ok(())
}

fn terminal_bench_command(
    dataset_path: &Path,
    agent: &TerminalBenchAgent,
    output_root: &Path,
    run_id: &str,
    profile: &AgentProfile,
    ctx: &ExternalTaskExecution<'_>,
) -> String {
    let (agent_timeout, test_timeout, _) = terminal_bench_timeout_values(
        ctx.spec.execution.timeout_sec,
        profile.timeout_sec,
        ctx.task.verifier_spec.timeout_sec,
    );
    let mut command = vec![
        terminal_bench_agent_env(profile, agent_timeout),
        "if [ -z \"${DOCKER_HOST:-}\" ] && [ -S \"$HOME/.colima/default/docker.sock\" ]; then export DOCKER_HOST=\"unix://$HOME/.colima/default/docker.sock\"; fi;".to_string(),
        "uvx --from terminal-bench tb run".to_string(),
        format!("--dataset-path {}", shell_quote(&dataset_path.display().to_string())),
        format!("--task-id {}", shell_quote(&ctx.task.task_id)),
        "--n-attempts 1".to_string(),
        "--n-concurrent 1".to_string(),
        format!("--global-agent-timeout-sec {agent_timeout}"),
        format!("--global-test-timeout-sec {test_timeout}"),
        format!("--output-path {}", shell_quote(&output_root.display().to_string())),
        format!("--run-id {}", shell_quote(run_id)),
        "--no-upload-results".to_string(),
    ];
    match agent {
        TerminalBenchAgent::BuiltIn { name, model } => {
            command.push(format!("--agent {}", shell_quote(name)));
            if requires_terminal_bench_model(name)
                && let Some(model) = model
            {
                command.push(format!("--model {}", shell_quote(model)));
            }
        }
        TerminalBenchAgent::ImportPath(path) => {
            command.push(format!("--agent-import-path {}", shell_quote(path)));
        }
    }
    command.join(" ")
}

fn terminal_bench_agent_env(profile: &AgentProfile, agent_timeout: u64) -> String {
    let input_mode = terminal_bench_input_mode(profile);
    let working_dir = format!("{:?}", profile.working_dir).to_ascii_lowercase();
    let timeout = agent_timeout.to_string();
    let mut exports = [
        ("HARNESSLAB_AGENT_NAME", profile.name.as_str()),
        ("HARNESSLAB_AGENT_COMMAND", profile.command.as_str()),
        ("HARNESSLAB_AGENT_INPUT_MODE", input_mode),
        ("HARNESSLAB_AGENT_WORKING_DIR", working_dir.as_str()),
        ("HARNESSLAB_AGENT_TIMEOUT_SEC", timeout.as_str()),
    ]
    .into_iter()
    .map(|(name, value)| format!("export {name}={}", shell_quote(value)))
    .collect::<Vec<_>>();
    if let Some(path) = profile.labels.get("terminal_bench_agent_pythonpath")
        && !path.trim().is_empty()
    {
        exports.push(format!(
            "export PYTHONPATH={}${{PYTHONPATH:+:$PYTHONPATH}}",
            shell_quote(path)
        ));
    }
    format!("{};", exports.join("; "))
}

fn terminal_bench_input_mode(profile: &AgentProfile) -> &'static str {
    match profile.input_mode {
        InputMode::Stdin | InputMode::Tty => "stdin",
        InputMode::Argument => "argument",
        InputMode::File => "file",
    }
}

fn terminal_bench_timeout_values(
    run_timeout: Option<u64>,
    profile_timeout: u64,
    verifier_timeout: u64,
) -> (u64, u64, u64) {
    let agent_timeout = run_timeout.unwrap_or(profile_timeout).max(1);
    let test_timeout = run_timeout.unwrap_or(verifier_timeout).max(1);
    let process_timeout = agent_timeout.max(test_timeout).saturating_add(600);
    (agent_timeout, test_timeout, process_timeout)
}

enum TerminalBenchAgent {
    BuiltIn { name: String, model: Option<String> },
    ImportPath(String),
}

fn terminal_bench_agent(profile: &AgentProfile) -> Result<TerminalBenchAgent> {
    if let Some(path) = profile.labels.get("terminal_bench_agent_import_path") {
        return Ok(TerminalBenchAgent::ImportPath(path.clone()));
    }
    let model = terminal_bench_model(profile);
    if let Some(name) = profile.labels.get("terminal_bench_agent") {
        if requires_terminal_bench_model(name) && model.is_none() {
            bail!(
                "agent profile {} must set label terminal_bench_model or model for terminal-bench {} agent",
                profile.name,
                name
            );
        }
        return Ok(TerminalBenchAgent::BuiltIn {
            name: name.clone(),
            model,
        });
    }
    match profile.kind {
        AgentKind::Codex | AgentKind::Opencode if model.is_none() => bail!(
            "agent profile {} must set label terminal_bench_model or model for terminal-bench {} agent",
            profile.name,
            match profile.kind {
                AgentKind::Codex => "codex",
                AgentKind::Opencode => "opencode",
                _ => unreachable!(),
            }
        ),
        AgentKind::Codex => Ok(TerminalBenchAgent::BuiltIn {
            name: "codex".to_string(),
            model,
        }),
        AgentKind::ClaudeCode => Ok(TerminalBenchAgent::BuiltIn {
            name: "claude-code".to_string(),
            model,
        }),
        AgentKind::Opencode => Ok(TerminalBenchAgent::BuiltIn {
            name: "opencode".to_string(),
            model,
        }),
        AgentKind::PiCodingAgent | AgentKind::Custom | AgentKind::Fake => bail!(
            "agent profile {} must set label terminal_bench_agent or terminal_bench_agent_import_path",
            profile.name
        ),
    }
}

fn terminal_bench_model(profile: &AgentProfile) -> Option<String> {
    profile
        .labels
        .get("terminal_bench_model")
        .or_else(|| profile.labels.get("model"))
        .filter(|value| !value.trim().is_empty() && value.as_str() != "user-configured")
        .cloned()
}

fn requires_terminal_bench_model(name: &str) -> bool {
    matches!(name, "codex" | "opencode")
}

fn parse_terminal_bench_result(
    attempt_dir: &Path,
    result_path: &Path,
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

fn missing_evaluation(
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

fn read_result_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
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

fn official_run_id(spec: &RunSpec, task: &TaskPlan, attempt: u32) -> String {
    format!("{}-{}-{}", spec.run_id, task.task_id, attempt)
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn normalize_agent_paths(mut process: ProcessRecord) -> ProcessRecord {
    process.stdout_path = "agent/stdout.log".to_string();
    process.stderr_path = "agent/stderr.log".to_string();
    process
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
#[cfg(test)]
#[path = "external_tests.rs"]
mod tests;
