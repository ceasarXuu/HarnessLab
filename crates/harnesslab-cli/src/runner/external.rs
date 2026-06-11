use crate::agent_registry::{MaterializedAgentProfile, run_as_requires_sandbox};
use crate::runner::external::runtime_adapter::{
    RuntimeCleanupContext, cleanup_runtime_target, preflight_external_task,
    runtime_adapter_for_task, runtime_cleanup_targets,
};
use crate::runner::external::runtime_snapshot::{
    ExternalRuntimeSnapshotRequest, RuntimePhaseCommand, write_external_runtime_snapshots,
};
use crate::runner::store;
use anyhow::{Result, bail};
use harnesslab_core::{
    AgentProfile, AttemptProvenance, ExternalRunnerKind, FailureClass, FailureCode, Outcome,
    RunSpec, RuntimePreflightReport, TaskAttemptResult, TaskPlan, TaskState, UsageRecord,
    health_impact_for_failure, redact_public_value,
};
use harnesslab_infra::{append_event, atomic_write_json, event};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

mod log_scan;
mod runtime_adapter;
mod runtime_anchor;
mod runtime_authority;
mod runtime_snapshot;
mod swe_bench_pro;
mod swe_bench_pro_adapter;
mod terminal_bench;
mod terminal_bench_adapter;
pub(super) mod terminal_bench_cleanup;
mod terminal_bench_env;
mod terminal_bench_result;
mod terminal_bench_runtime;
mod terminal_bench_runtime_snapshot;
mod terminal_bench_timeout;

pub(super) use runtime_adapter::{RuntimeCleanupPhase, RuntimeCleanupReport, RuntimeCleanupTarget};
pub(super) use runtime_authority::{
    runtime_dataset_ref, runtime_snapshot_source_ref, runtime_source_ref,
};

pub(super) fn is_external_task(task: &TaskPlan) -> bool {
    task.external_runner.is_some() || task.runtime_binding.is_some()
}

pub(super) fn validate_profile_for_plan(profile: &AgentProfile, tasks: &[TaskPlan]) -> Result<()> {
    let external_reports = collect_runtime_preflight_reports(profile, tasks)?;
    validate_run_as_for_plan(profile, tasks, &external_reports)?;
    Ok(())
}

pub(super) fn emit_runtime_preflight_reports(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    tasks: &[TaskPlan],
) -> Result<()> {
    for (task, report) in collect_runtime_preflight_reports(profile, tasks)? {
        let message = format!(
            "adapter_id={} protocol_adapter_id={} protocol_version={} protocol_benchmark_id={} protocol_selected_mode={} protocol_stability={} protocol_capabilities={} legacy_shim_used={} adapter_phase=preflight runner_kind={:?} agent_bridge_mode={} readiness_status={} host_execution_reason={} blocking_reason={} compatibility_exception={} compatibility_label_keys={}",
            report.adapter_id,
            report.protocol_adapter_id.as_deref().unwrap_or("none"),
            report.protocol_version.as_deref().unwrap_or("none"),
            report.protocol_benchmark_id.as_deref().unwrap_or("none"),
            report.protocol_selected_mode.as_deref().unwrap_or("none"),
            report.protocol_stability.as_deref().unwrap_or("none"),
            report.protocol_capabilities.join(","),
            report.legacy_shim_used,
            report.runner_kind,
            report.agent_bridge_mode,
            report.readiness_status,
            report.host_execution_reason.as_deref().unwrap_or("none"),
            report.blocking_reason.as_deref().unwrap_or("none"),
            report.compatibility_exception.as_deref().unwrap_or("none"),
            report.compatibility_label_keys.join(",")
        );
        append_event(
            &run_dir.join("events.jsonl"),
            &event(
                &spec.run_id,
                Some(&task.task_id),
                "external_runner_preflight",
                &message,
            ),
            &[],
        )?;
    }
    Ok(())
}

pub(super) fn runtime_cleanup_targets_for_phase(
    run_dir: &Path,
    spec: &RunSpec,
    plan: &harnesslab_core::BenchmarkPlan,
    phase: RuntimeCleanupPhase,
) -> Vec<RuntimeCleanupTarget> {
    runtime_cleanup_targets(RuntimeCleanupContext {
        run_dir,
        spec,
        plan,
        phase,
    })
}

pub(super) fn cleanup_runtime_resources(
    target: &RuntimeCleanupTarget,
) -> Result<RuntimeCleanupReport, String> {
    cleanup_runtime_target(target)
}

pub(super) fn runtime_adapter_version_for_task(task: &TaskPlan) -> Result<&'static str> {
    Ok(runtime_adapter_for_task(task)?.adapter_version())
}

pub(super) fn runtime_runner_kind_for_task(task: &TaskPlan) -> Result<ExternalRunnerKind> {
    Ok(runtime_adapter_for_task(task)?.kind())
}

#[derive(Debug)]
pub(super) struct AdapterInternalError {
    subphase: &'static str,
    failure_code: FailureCode,
    message: String,
}

impl AdapterInternalError {
    fn subphase(&self) -> &'static str {
        self.subphase
    }

    fn failure_code(&self) -> FailureCode {
        self.failure_code
    }
}

impl fmt::Display for AdapterInternalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for AdapterInternalError {}

pub(super) fn adapter_internal_error(
    subphase: &'static str,
    failure_code: FailureCode,
    error: anyhow::Error,
) -> anyhow::Error {
    AdapterInternalError {
        subphase,
        failure_code,
        message: error.to_string(),
    }
    .into()
}

fn collect_runtime_preflight_reports<'a>(
    profile: &AgentProfile,
    tasks: &'a [TaskPlan],
) -> Result<Vec<(&'a TaskPlan, RuntimePreflightReport)>> {
    let mut reports = Vec::new();
    for task in tasks {
        if is_external_task(task) {
            reports.push((task, preflight_external_task(profile, task)?));
        }
    }
    Ok(reports)
}

fn validate_run_as_for_plan(
    profile: &AgentProfile,
    tasks: &[TaskPlan],
    external_reports: &[(&TaskPlan, RuntimePreflightReport)],
) -> Result<()> {
    if !run_as_requires_sandbox(profile.setup.run_as) {
        if let Some((task, report)) = external_reports
            .iter()
            .find(|(_, report)| report.blocking_reason.is_some())
        {
            bail!("{}", runtime_preflight_blocked_message(task, report));
        }
        return Ok(());
    }
    if let Some((task, report)) = external_reports
        .iter()
        .find(|(_, report)| report.blocking_reason.is_some())
    {
        bail!("{}", runtime_preflight_blocked_message(task, report));
    }
    if let Some((task, reason)) = tasks.iter().find_map(host_task_execution_reason) {
        bail!(
            "setup.run_as={:?} is not enforceable for {}; task={}; host execution only supports setup.run_as=\"current\"; use a sandboxed agent path or set setup.run_as=\"current\"",
            profile.setup.run_as,
            reason,
            task.task_id
        );
    }
    if let Some((task, report)) = external_reports
        .iter()
        .find(|(_, report)| report.host_execution_reason.is_some())
    {
        let reason = report
            .host_execution_reason
            .as_deref()
            .unwrap_or("external host path");
        bail!(
            "setup.run_as={:?} is not enforceable for {}; task={}; host execution only supports setup.run_as=\"current\"; use a sandboxed agent path or set setup.run_as=\"current\"",
            profile.setup.run_as,
            reason,
            task.task_id
        );
    }
    Ok(())
}

fn runtime_preflight_blocked_message(task: &TaskPlan, report: &RuntimePreflightReport) -> String {
    let reason = report
        .blocking_reason
        .as_deref()
        .unwrap_or("runtime preflight blocked");
    format!(
        "runtime preflight blocked for {}; adapter_phase=preflight; task={}; readiness_status={}; blocking_reason={}; remediation=fix the adapter-specific profile labels or benchmark source material before running",
        report.adapter_id, task.task_id, report.readiness_status, reason
    )
}

fn host_task_execution_reason(task: &TaskPlan) -> Option<(&TaskPlan, &'static str)> {
    (task.external_runner.is_none() && !super::sandbox::task_requires_docker(task))
        .then_some((task, "host task"))
}

pub(super) struct ExternalTaskExecution<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) profile: &'a AgentProfile,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) materialized_profile: &'a MaterializedAgentProfile,
    pub(super) report_materialized_profile: &'a MaterializedAgentProfile,
    pub(super) task: &'a TaskPlan,
    pub(super) attempt: u32,
    pub(super) provenance: AttemptProvenance,
    pub(super) attempt_dir: &'a Path,
    pub(super) started: std::time::Instant,
}

pub(super) fn execute_external_task(ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
    let adapter = runtime_adapter_for_task(ctx.task)?;
    match adapter.execute(&ctx) {
        Ok(result) => Ok(result),
        Err(error) => internal_error_result(&ctx, adapter, error),
    }
}

fn internal_error_result(
    ctx: &ExternalTaskExecution<'_>,
    adapter: &dyn runtime_adapter::BenchmarkRuntimeAdapter,
    error: anyhow::Error,
) -> Result<TaskAttemptResult> {
    let typed_error = error.downcast_ref::<AdapterInternalError>();
    let adapter_subphase = typed_error
        .map(AdapterInternalError::subphase)
        .unwrap_or("execute");
    let failure_code = typed_error
        .map(AdapterInternalError::failure_code)
        .unwrap_or(FailureCode::ExternalRunnerSetupFailed);
    let public_message = format!(
        "adapter_id={} adapter_phase=execute adapter_subphase={} runner_kind={:?} failure_class=execution failure_code={} public_diagnostics=internal-error.public.json private_diagnostics=internal-error.private.json",
        adapter.adapter_id(),
        adapter_subphase,
        adapter.kind(),
        failure_code_event_label(failure_code)
    );
    let private_message = format!(
        "{} error={error}",
        public_message.replace("private_diagnostics=internal-error.private.json", "")
    );
    let redaction_refs = event_redaction_refs(ctx);
    let secret_refs = redaction_refs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_internal_error",
            &public_message,
        ),
        &secret_refs,
    )?;
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: TaskState::Failure,
        outcome: Outcome::Failure,
        failure_class: FailureClass::Execution,
        failure_code: Some(failure_code),
        health_impact: health_impact_for_failure(FailureClass::Execution, Some(failure_code)),
        benchmark_score: 0.0,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: vec![FailureCode::UsageUnknown],
    };
    write_internal_error_snapshot(
        ctx,
        adapter,
        adapter_subphase,
        failure_code,
        &private_message,
    )?;
    atomic_write_json(&ctx.attempt_dir.join("result.json"), &result)?;
    Ok(result)
}

fn write_internal_error_snapshot(
    ctx: &ExternalTaskExecution<'_>,
    adapter: &dyn runtime_adapter::BenchmarkRuntimeAdapter,
    adapter_subphase: &'static str,
    failure_code: FailureCode,
    message: &str,
) -> Result<()> {
    let public_diagnostics = serde_json::json!({
        "internal_error": {
            "adapter_id": adapter.adapter_id(),
            "phase": "execute",
            "subphase": adapter_subphase,
            "failure_code": failure_code_event_label(failure_code)
        }
    });
    let private_diagnostics = serde_json::json!({
        "internal_error": {
            "adapter_id": adapter.adapter_id(),
            "phase": "execute",
            "subphase": adapter_subphase,
            "failure_code": failure_code_event_label(failure_code),
            "message": message
        }
    });
    atomic_write_json(
        &ctx.attempt_dir.join("internal-error.public.json"),
        &public_diagnostics,
    )?;
    atomic_write_json(
        &ctx.attempt_dir.join("internal-error.private.json"),
        &private_diagnostics,
    )?;
    if ctx
        .attempt_dir
        .join("external-runtime.public.json")
        .is_file()
        && ctx
            .attempt_dir
            .join("external-runtime.private.json")
            .is_file()
    {
        return Ok(());
    }
    let dataset_ref = runtime_dataset_ref(ctx.task)?;
    let source_ref = runtime_snapshot_source_ref(ctx.task, adapter.kind())?;
    write_external_runtime_snapshots(ExternalRuntimeSnapshotRequest {
        run_id: &ctx.spec.run_id,
        attempt_dir: ctx.attempt_dir,
        benchmark: adapter.benchmark_name(),
        task_id: &ctx.task.task_id,
        attempt: ctx.attempt,
        runner_kind: adapter.kind(),
        protocol_authority: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.clone()),
        adapter_version: adapter.adapter_version(),
        network: ctx.spec.execution.network,
        timeout_sec: ctx
            .spec
            .execution
            .timeout_sec
            .or(Some(ctx.profile.timeout_sec)),
        profile: ctx.profile,
        dataset_path: Path::new(dataset_ref),
        source_path: source_ref.map(Path::new),
        commands: vec![RuntimePhaseCommand {
            phase: adapter_subphase,
            command: format!("{} internal error", adapter.adapter_id()),
            working_dir: ctx.attempt_dir.to_path_buf(),
            timeout_sec: ctx
                .spec
                .execution
                .timeout_sec
                .unwrap_or(ctx.profile.timeout_sec),
            stdout_path: ctx.attempt_dir.join("agent/stdout.log"),
            stderr_path: ctx.attempt_dir.join("agent/stderr.log"),
        }],
        materials: Vec::new(),
        public_artifacts: vec!["result.json".to_string(), "events.jsonl".to_string()],
        extra_redaction_refs: vec![ctx.attempt_dir.display().to_string()],
        private_diagnostics: Some(private_diagnostics),
        public_diagnostics: Some(public_diagnostics),
    })
}

fn event_redaction_refs(ctx: &ExternalTaskExecution<'_>) -> Vec<String> {
    let mut refs = store::secret_values(ctx.profile);
    refs.push(ctx.run_dir.display().to_string());
    refs.push(ctx.attempt_dir.display().to_string());
    if let Some(runner) = &ctx.task.external_runner {
        refs.push(runner.dataset_path.clone());
        if let Some(source_path) = &runner.source_path {
            refs.push(source_path.clone());
        }
    }
    if let Some(binding) = &ctx.task.runtime_binding {
        refs.push(binding.dataset_ref.clone());
        refs.push(binding.task_ref.clone());
    }
    refs
}

fn failure_code_event_label(code: FailureCode) -> &'static str {
    match code {
        FailureCode::AgentCleanupFailed => "agent_cleanup_failed",
        FailureCode::ArtifactCollectionFailed => "artifact_collection_failed",
        FailureCode::ExternalRunnerSetupFailed => "external_runner_setup_failed",
        FailureCode::EvaluatorError => "evaluator_error",
        FailureCode::PatchApplyFailed => "patch_apply_failed",
        _ => "other",
    }
}

pub(super) fn write_external_command_snapshot(
    attempt_dir: &Path,
    runtime_profile: &AgentProfile,
    report_profile: &AgentProfile,
    command: &str,
    extra_redaction_refs: &[String],
) -> Result<()> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    let mut secrets = external_snapshot_secret_refs(runtime_profile);
    for extra in extra_redaction_refs {
        push_redaction_ref(&mut secrets, extra);
    }
    let secret_refs = secrets.iter().map(String::as_str).collect::<Vec<_>>();
    fs::write(
        agent_dir.join("command.txt"),
        format!(
            "template={}\nrendered={}\ninput_mode=external\n",
            redact_public_value(&report_profile.command, &secret_refs),
            redact_public_value(command, &secret_refs)
        ),
    )?;
    Ok(())
}

fn external_snapshot_secret_refs(runtime_profile: &AgentProfile) -> Vec<String> {
    let mut refs = Vec::new();
    for secret in store::secret_values(runtime_profile) {
        if secret.is_empty() {
            continue;
        }
        push_redaction_ref(&mut refs, &secret);
    }
    refs
}

fn push_redaction_ref(refs: &mut Vec<String>, value: &str) {
    if value.is_empty() {
        return;
    }
    push_unique(refs, value.to_string());
    let escaped = value.replace('\'', "'\\''");
    push_unique(refs, escaped.clone());
    push_unique(refs, format!("'{escaped}'"));
    push_unique(refs, format!("\"{}\"", value.replace('"', "\\\"")));
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}
#[cfg(test)]
#[path = "external_tests.rs"]
mod tests;
