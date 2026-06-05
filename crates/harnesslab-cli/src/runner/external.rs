use crate::agent_registry::{MaterializedAgentProfile, run_as_requires_sandbox};
use crate::runner::external::runtime_adapter::{
    RuntimeCleanupContext, cleanup_runtime_target, preflight_external_task, runtime_adapter_for,
    runtime_cleanup_targets,
};
use crate::runner::store;
use anyhow::{Result, bail};
use harnesslab_core::{
    AgentProfile, AttemptProvenance, RunSpec, RuntimePreflightReport, TaskAttemptResult, TaskPlan,
    redact_public_value,
};
use harnesslab_infra::{append_event, event};
use std::fs;
use std::path::Path;

mod log_scan;
mod runtime_adapter;
mod runtime_anchor;
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

pub(super) fn is_external_task(task: &TaskPlan) -> bool {
    task.external_runner.is_some()
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
            "adapter_id={} adapter_phase=preflight runner_kind={:?} agent_bridge_mode={} readiness_status={} host_execution_reason={} blocking_reason={} compatibility_exception={} compatibility_label_keys={}",
            report.adapter_id,
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

fn collect_runtime_preflight_reports<'a>(
    profile: &AgentProfile,
    tasks: &'a [TaskPlan],
) -> Result<Vec<(&'a TaskPlan, RuntimePreflightReport)>> {
    let mut reports = Vec::new();
    for task in tasks {
        if task.external_runner.is_some() {
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
    let Some(runner) = &ctx.task.external_runner else {
        bail!("external task missing runner spec");
    };
    runtime_adapter_for(runner.kind).execute(ctx)
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
