use crate::agent_registry::{MaterializedAgentProfile, run_as_requires_sandbox};
use crate::runner::external::runtime_adapter::{preflight_external_task, runtime_adapter_for};
use crate::runner::store;
use anyhow::{Result, bail};
use harnesslab_core::{
    AgentProfile, AttemptProvenance, RunSpec, RuntimePreflightReport, TaskAttemptResult, TaskPlan,
    redact_public_value,
};
use std::fs;
use std::path::Path;

mod log_scan;
mod runtime_adapter;
mod runtime_anchor;
mod runtime_snapshot;
mod swe_bench_pro;
mod terminal_bench;
pub(super) mod terminal_bench_cleanup;
mod terminal_bench_env;
mod terminal_bench_result;
mod terminal_bench_runtime;
mod terminal_bench_timeout;

pub(super) fn is_external_task(task: &TaskPlan) -> bool {
    task.external_runner.is_some()
}

pub(super) fn validate_profile_for_plan(profile: &AgentProfile, tasks: &[TaskPlan]) -> Result<()> {
    let mut external_reports = Vec::new();
    for task in tasks {
        if task.external_runner.is_some() {
            external_reports.push((task, preflight_external_task(profile, task)?));
        }
    }
    validate_run_as_for_plan(profile, tasks, &external_reports)?;
    Ok(())
}

fn validate_run_as_for_plan(
    profile: &AgentProfile,
    tasks: &[TaskPlan],
    external_reports: &[(&TaskPlan, RuntimePreflightReport)],
) -> Result<()> {
    if !run_as_requires_sandbox(profile.setup.run_as) {
        return Ok(());
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
) -> Result<()> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    let secrets = external_snapshot_secret_refs(runtime_profile);
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
        push_unique(&mut refs, secret.clone());
        let escaped = secret.replace('\'', "'\\''");
        push_unique(&mut refs, escaped.clone());
        push_unique(&mut refs, format!("'{escaped}'"));
    }
    refs
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}
#[cfg(test)]
#[path = "external_tests.rs"]
mod tests;
