use super::{
    ExternalTaskExecution, log_scan, terminal_bench_cleanup,
    terminal_bench_result::{
        missing_evaluation, terminal_bench_result_warnings, write_task_result,
    },
    terminal_bench_runtime::TerminalBenchRuntimeAttempt,
    terminal_bench_runtime_snapshot::write_terminal_bench_runtime_snapshots,
};
use crate::runner::store;
use anyhow::Result;
use harnesslab_core::{
    Failure, FailureClass, FailureCode, Outcome, ProcessRecord, TaskAttemptResult, TaskState,
    TerminationReason, UsageRecord, classify_agent_process, health_impact_for_failure,
};
use harnesslab_infra::{ExecSpec, HostProcessExecutor, NoOutputActivityEvent, append_event, event};

pub(super) use super::terminal_bench_result::parse_terminal_bench_result;

pub(super) fn execute_prepared(
    ctx: &ExternalTaskExecution<'_>,
    prepared: TerminalBenchRuntimeAttempt,
) -> Result<TaskAttemptResult> {
    let pre_cleanup = terminal_bench_cleanup::cleanup_task_resources(
        ctx.run_dir,
        ctx.spec,
        &ctx.task.task_id,
        "pre_task",
        &prepared.official_run_id,
        true,
        &cleanup_redaction_refs(ctx, &prepared),
    )?;
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_started",
            &format!(
                "terminal-bench dataset=[PRIVATE_PATH] runtime_dataset={} official_run_id=<run-id> output=official/terminal-bench",
                public_runtime_dataset_path(ctx, &prepared),
            ),
        ),
        &[],
    )?;
    let process = normalize_agent_paths(HostProcessExecutor::exec(&ExecSpec {
        command: prepared.command.clone(),
        stdin: None,
        working_dir: ctx.attempt_dir.to_path_buf(),
        timeout_sec: prepared.process_timeout_sec,
        no_output_timeout_sec: prepared.no_output_timeout_sec,
        no_output_progress_paths: prepared.no_output_progress_paths.clone(),
        no_output_activity_patterns: prepared.no_output_activity_patterns.clone(),
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
    append_process_termination_event(ctx, &process, prepared.process_timeout_sec)?;
    let post_cleanup = terminal_bench_cleanup::cleanup_task_resources(
        ctx.run_dir,
        ctx.spec,
        &ctx.task.task_id,
        "post_task",
        &prepared.official_run_id,
        false,
        &cleanup_redaction_refs(ctx, &prepared),
    )?;
    let post_cleanup_error = post_cleanup.error.clone();
    let parsed_result =
        parse_terminal_bench_result(ctx.attempt_dir, &prepared.result_path, &ctx.task.task_id);
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
                missing_evaluation(ctx.attempt_dir, &prepared.result_path, &error.to_string())?
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
        terminal_bench_result_warnings(
            &prepared.result_path,
            &ctx.task.task_id,
            official_failure_class,
        )
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
    terminal_bench_cleanup::write_task_cleanup_report(
        terminal_bench_cleanup::TaskCleanupReportRequest {
            attempt_dir: ctx.attempt_dir,
            task_id: &ctx.task.task_id,
            attempt: ctx.attempt,
            pre_task: &pre_cleanup,
            post_task: &post_cleanup,
            official_failure_class,
            official_failure_code,
            final_failure_class: failure_class,
            final_failure_code: failure_code,
            cleanup_overrides_result,
        },
    )
    .map_err(|error| {
        super::adapter_internal_error(
            "post_execution_cleanup",
            FailureCode::AgentCleanupFailed,
            error,
        )
    })?;
    write_terminal_bench_runtime_snapshots(
        ctx,
        &prepared,
        super::terminal_bench_runtime_snapshot::TerminalBenchSnapshotDiagnostics::post_execution(
            &pre_cleanup,
            &post_cleanup,
            official_failure_class,
            official_failure_code,
            failure_class,
            failure_code,
            cleanup_overrides_result,
        ),
    )
    .map_err(|error| {
        super::adapter_internal_error(
            "post_execution_snapshot",
            FailureCode::ArtifactCollectionFailed,
            error,
        )
    })?;
    write_task_result(ctx, &result).map_err(|error| {
        super::adapter_internal_error(
            "post_execution_result",
            FailureCode::ArtifactCollectionFailed,
            error,
        )
    })?;
    Ok(result)
}

fn public_runtime_dataset_path(
    ctx: &ExternalTaskExecution<'_>,
    prepared: &TerminalBenchRuntimeAttempt,
) -> String {
    prepared
        .runtime_dataset_path
        .strip_prefix(ctx.attempt_dir)
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "[PRIVATE_PATH]".to_string())
}

fn cleanup_redaction_refs(
    ctx: &ExternalTaskExecution<'_>,
    prepared: &TerminalBenchRuntimeAttempt,
) -> Vec<String> {
    let mut refs = store::secret_values(ctx.profile);
    refs.extend([
        ctx.run_dir.display().to_string(),
        ctx.attempt_dir.display().to_string(),
        prepared.source_dataset_path.display().to_string(),
        prepared.runtime_dataset_path.display().to_string(),
        prepared.output_root.display().to_string(),
    ]);
    refs
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

fn normalize_agent_paths(mut process: ProcessRecord) -> ProcessRecord {
    process.stdout_path = "agent/stdout.log".to_string();
    process.stderr_path = "agent/stderr.log".to_string();
    process
}
