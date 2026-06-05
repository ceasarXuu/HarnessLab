use super::{
    ExternalTaskExecution, log_scan, terminal_bench_cleanup,
    terminal_bench_env::terminal_bench_agent_env,
    terminal_bench_result::{
        missing_evaluation, setup_failed_result, terminal_bench_result_warnings, write_task_result,
    },
    terminal_bench_runtime::{
        append_runner_config_event, terminal_bench_docker_platform,
        terminal_bench_no_output_activity_patterns, terminal_bench_runtime_dataset,
    },
    terminal_bench_timeout::{
        terminal_bench_no_output_timeout_sec, terminal_bench_process_timeout_sec,
        terminal_bench_timeout_values,
    },
    write_external_command_snapshot,
};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::{Result, bail};
use harnesslab_core::{
    AgentKind, AgentProfile, Failure, FailureClass, FailureCode, Outcome, ProcessRecord, RunSpec,
    TaskAttemptResult, TaskPlan, TaskState, TerminationReason, UsageRecord, classify_agent_process,
    health_impact_for_failure,
};
use harnesslab_infra::{ExecSpec, HostProcessExecutor, NoOutputActivityEvent, append_event, event};
use std::fs;
use std::path::Path;

pub(super) use super::terminal_bench_result::parse_terminal_bench_result;

const IMPORT_AGENT_CLEANUP_GRACE_SEC: u64 = 30;

pub(super) fn validate_profile(
    profile: &AgentProfile,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<()> {
    let _ = terminal_bench_agent(profile, compatibility)?;
    Ok(())
}

pub(super) fn execute(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<TaskAttemptResult> {
    let attempt_root = fs::canonicalize(ctx.attempt_dir)?;
    let output_root = attempt_root.join("official/terminal-bench");
    let official_run_id = official_run_id(ctx.spec, ctx.task, ctx.attempt);
    let agent = terminal_bench_agent(ctx.profile, compatibility)?;
    let docker_platform = terminal_bench_docker_platform(
        &ctx.task.task_id,
        std::env::var("HARNESSLAB_TERMINAL_BENCH_DOCKER_PLATFORM")
            .ok()
            .as_deref(),
    );
    let result_path = output_root.join(&official_run_id).join("results.json");
    let runtime_dataset_path =
        match terminal_bench_runtime_dataset(ctx, dataset_path, &docker_platform) {
            Ok(path) => path,
            Err(error) => {
                let reason = format!("terminal-bench runtime dataset preparation failed: {error}");
                return setup_failed_result(ctx, &result_path, &reason);
            }
        };
    terminal_bench_cleanup::cleanup_task_resources(
        ctx.run_dir,
        ctx.spec,
        &ctx.task.task_id,
        "pre_task",
        &official_run_id,
        true,
    )?;
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_started",
            &format!(
                "terminal-bench dataset={} runtime_dataset={} official_run_id={} output={}",
                dataset_path.display(),
                runtime_dataset_path.display(),
                official_run_id,
                output_root.display()
            ),
        ),
        &[],
    )?;
    let command = terminal_bench_command(
        &runtime_dataset_path,
        &agent,
        &output_root,
        &official_run_id,
        ctx.profile,
        ctx,
        &docker_platform,
        compatibility,
    );
    let report_command = terminal_bench_command(
        &runtime_dataset_path,
        &agent,
        &output_root,
        &official_run_id,
        ctx.report_profile,
        ctx,
        &docker_platform,
        compatibility,
    );
    write_external_command_snapshot(
        ctx.attempt_dir,
        ctx.profile,
        ctx.report_profile,
        &report_command,
    )?;
    let (agent_timeout_sec, test_timeout_sec, default_process_timeout_sec) =
        terminal_bench_timeout_values(
            ctx.spec.execution.timeout_sec,
            ctx.profile.timeout_sec,
            ctx.task.verifier_spec.timeout_sec,
            task_agent_timeout(ctx),
        );
    let process_timeout_sec = terminal_bench_process_timeout_sec(
        default_process_timeout_sec,
        std::env::var("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC")
            .ok()
            .as_deref(),
    );
    let no_output_timeout_sec = terminal_bench_no_output_timeout_sec(
        agent_timeout_sec,
        test_timeout_sec,
        process_timeout_sec,
        std::env::var("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC")
            .ok()
            .as_deref(),
    );
    append_runner_config_event(
        ctx,
        process_timeout_sec,
        no_output_timeout_sec,
        &docker_platform,
    )?;
    let process = normalize_agent_paths(HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: ctx.attempt_dir.to_path_buf(),
        timeout_sec: process_timeout_sec,
        no_output_timeout_sec,
        no_output_progress_paths: vec![output_root.join(&official_run_id).join("run.log")],
        no_output_activity_patterns: terminal_bench_no_output_activity_patterns(),
        no_output_activity_event: Some(NoOutputActivityEvent {
            path: ctx.run_dir.join("events.jsonl"),
            run_id: ctx.spec.run_id.clone(),
            task_id: Some(ctx.task.task_id.clone()),
            event_name: "external_runner_activity".to_string(),
            no_progress_event_name: Some("external_runner_no_progress".to_string()),
        }),
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: ctx.attempt_dir.join("agent/stdout.log"),
        stderr_path: ctx.attempt_dir.join("agent/stderr.log"),
    })?);
    append_process_termination_event(ctx, &process, process_timeout_sec)?;
    let post_cleanup_error = terminal_bench_cleanup::cleanup_task_resources(
        ctx.run_dir,
        ctx.spec,
        &ctx.task.task_id,
        "post_task",
        &official_run_id,
        false,
    )?;
    let parsed_result =
        parse_terminal_bench_result(ctx.attempt_dir, &result_path, &ctx.task.task_id);
    let agent_failure = terminal_bench_process_failure(&process);
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
    let official_failure_class = failure_class;
    let official_failure_code = failure_code;
    if let Some(code) = infra_failure {
        failure_class = FailureClass::Execution;
        failure_code = Some(code);
        score = 0.0;
    } else if agent_failure.class == FailureClass::Execution
        && process.termination_reason != TerminationReason::Completed
    {
        failure_class = FailureClass::Execution;
        failure_code = agent_failure.code;
        score = 0.0;
    }
    let cleanup_overrides_result =
        post_cleanup_error.is_some() && failure_class != FailureClass::Execution;
    if cleanup_overrides_result {
        failure_class = FailureClass::Execution;
        failure_code = Some(FailureCode::AgentCleanupFailed);
        score = 0.0;
    }
    let mut warnings = if infra_failure.is_some() {
        Vec::new()
    } else {
        terminal_bench_result_warnings(&result_path, &ctx.task.task_id, official_failure_class)
    };
    if failure_class == FailureClass::Execution
        && official_failure_class == FailureClass::Benchmark
        && infra_failure.is_none()
        && let Some(code) = official_failure_code
    {
        warnings.push(code);
    }
    if agent_failure.class == FailureClass::Execution
        && failure_class != FailureClass::Execution
        && let Some(code) = agent_failure.code
    {
        warnings.push(code);
    }
    if post_cleanup_error.is_some() && !cleanup_overrides_result {
        warnings.push(FailureCode::AgentCleanupFailed);
    }
    if matches!(usage, UsageRecord::Unknown) {
        warnings.push(FailureCode::UsageUnknown);
    }
    append_task_warnings(ctx, &warnings)?;
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
        health_impact: health_impact_for_failure(failure_class, failure_code),
        benchmark_score: score,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: Some(process),
        evaluation: Some(evaluation),
        patch: None,
        usage,
        warnings,
    };
    write_task_result(ctx, &result)?;
    Ok(result)
}

fn append_process_termination_event(
    ctx: &ExternalTaskExecution<'_>,
    process: &ProcessRecord,
    process_timeout_sec: u64,
) -> Result<()> {
    let (name, message) = match process.termination_reason {
        TerminationReason::NoProgress => return Ok(()),
        TerminationReason::Timeout => (
            "external_runner_timeout",
            format!(
                "terminal-bench official runner exceeded hard timeout {process_timeout_sec}s; killed process tree"
            ),
        ),
        _ => return Ok(()),
    };
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(&ctx.spec.run_id, Some(&ctx.task.task_id), name, &message),
        &[],
    )
}

fn append_task_warnings(ctx: &ExternalTaskExecution<'_>, warnings: &[FailureCode]) -> Result<()> {
    for warning in warnings {
        append_event(
            &ctx.run_dir.join("events.jsonl"),
            &event(
                &ctx.spec.run_id,
                Some(&ctx.task.task_id),
                "task_warning",
                &format!("attempt {} warning {warning:?}", ctx.attempt),
            ),
            &[],
        )?;
    }
    Ok(())
}

fn terminal_bench_command(
    dataset_path: &Path,
    agent: &TerminalBenchAgent,
    output_root: &Path,
    run_id: &str,
    profile: &AgentProfile,
    ctx: &ExternalTaskExecution<'_>,
    docker_platform: &str,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> String {
    let (agent_timeout, test_timeout, _) = terminal_bench_timeout_values(
        ctx.spec.execution.timeout_sec,
        profile.timeout_sec,
        ctx.task.verifier_spec.timeout_sec,
        task_agent_timeout(ctx),
    );
    let official_agent_timeout = terminal_bench_official_agent_timeout(
        agent_timeout,
        matches!(agent, TerminalBenchAgent::ImportPath(_)),
    );
    let mut command = vec![
        terminal_bench_agent_env(
            profile,
            ctx.materialized_profile,
            agent_timeout,
            compatibility,
        ),
        "if [ -z \"${DOCKER_HOST:-}\" ] && [ -S \"$HOME/.colima/default/docker.sock\" ]; then export DOCKER_HOST=\"unix://$HOME/.colima/default/docker.sock\"; fi;".to_string(),
        format!(
            "export DOCKER_DEFAULT_PLATFORM={}; export BUILDKIT_PROGRESS=plain;",
            shell_quote(docker_platform)
        ),
        "uvx --from terminal-bench tb run".to_string(),
        format!("--dataset-path {}", shell_quote(&dataset_path.display().to_string())),
        format!("--task-id {}", shell_quote(&ctx.task.task_id)),
        "--n-attempts 1".to_string(),
        "--n-concurrent 1".to_string(),
        format!("--global-agent-timeout-sec {official_agent_timeout}"),
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

fn task_agent_timeout(ctx: &ExternalTaskExecution<'_>) -> Option<u64> {
    ctx.task
        .external_runner
        .as_ref()
        .and_then(|runner| runner.agent_timeout_sec)
}

pub(super) fn terminal_bench_official_agent_timeout(
    agent_timeout: u64,
    uses_import_agent: bool,
) -> u64 {
    if uses_import_agent {
        agent_timeout.saturating_add(IMPORT_AGENT_CLEANUP_GRACE_SEC)
    } else {
        agent_timeout
    }
}

enum TerminalBenchAgent {
    BuiltIn { name: String, model: Option<String> },
    ImportPath(String),
}

fn terminal_bench_agent(
    profile: &AgentProfile,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<TerminalBenchAgent> {
    if let Some(path) = &compatibility.terminal_bench_agent_import_path {
        return Ok(TerminalBenchAgent::ImportPath(path.clone()));
    }
    let model = compatibility.terminal_bench_model.clone();
    if let Some(name) = &compatibility.terminal_bench_agent {
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

fn requires_terminal_bench_model(name: &str) -> bool {
    matches!(name, "codex" | "opencode")
}

pub(super) fn terminal_bench_process_failure(process: &ProcessRecord) -> Failure {
    match process.termination_reason {
        TerminationReason::NoProgress => Failure {
            class: FailureClass::Execution,
            code: Some(FailureCode::ExternalRunnerNoProgress),
            message: "terminal-bench official runner made no log progress before watchdog timeout"
                .to_string(),
        },
        TerminationReason::Timeout => Failure {
            class: FailureClass::Execution,
            code: Some(FailureCode::ExternalRunnerTimeout),
            message: "terminal-bench official runner exceeded its hard timeout".to_string(),
        },
        _ => classify_agent_process(process),
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
