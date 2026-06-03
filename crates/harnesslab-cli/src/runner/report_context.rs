use crate::agent_registry::{AgentVersionSnapshot, MaterializedAgentProfile, VersionProbeStatus};
use anyhow::Result;
use harnesslab_core::{AgentProfile, ResolvedCapabilityPolicy, RunSpec};
use std::fs;
use std::path::Path;

pub(super) fn build_report_context(
    spec: &RunSpec,
    profile: &AgentProfile,
    materialized: &MaterializedAgentProfile,
    version_snapshot: Option<&AgentVersionSnapshot>,
    report_path: String,
    run_health: super::monitor::ReportHealth,
    run_dir: &Path,
    resumed: bool,
) -> harnesslab_report::ReportContext {
    harnesslab_report::ReportContext {
        run_id: spec.run_id.clone(),
        agent: spec.agent_profile_ref.clone(),
        agent_config_summary: super::store::agent_config_summary(spec, profile, materialized),
        setup_summary: materialized.setup_summary.clone(),
        skills_summary: materialized.skills_summary.clone(),
        tools_summary: materialized.tools_summary.clone(),
        hooks_summary: materialized.hooks_summary.clone(),
        skills_effective_summary: capability_effective_summary(&materialized.capabilities.skills),
        tools_effective_summary: capability_effective_summary(&materialized.capabilities.tools),
        hooks_effective_summary: capability_effective_summary(&materialized.capabilities.hooks),
        version_probe_summary: version_probe_summary(version_snapshot),
        has_version_probe_snapshot: version_snapshot.is_some(),
        benchmark: spec.benchmark.name.clone(),
        split: spec.benchmark.split.clone(),
        report_path,
        replay_command: super::store::replay_command(spec),
        original_command: super::store::original_command_from_snapshot(run_dir),
        resumed,
        run_health_status: run_health.status,
        run_health_reason: run_health.reason,
    }
}

pub(super) struct ReportWriteRequest<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) report_materialized: &'a MaterializedAgentProfile,
    pub(super) version_snapshot: Option<&'a AgentVersionSnapshot>,
    pub(super) report_path: String,
    pub(super) run_health: super::monitor::ReportHealth,
    pub(super) resumed: bool,
    pub(super) results: &'a harnesslab_core::RunResults,
}

pub(super) fn write_report(request: ReportWriteRequest<'_>) -> Result<()> {
    let model = harnesslab_report::build_report_model(
        build_report_context(
            request.spec,
            request.report_profile,
            request.report_materialized,
            request.version_snapshot,
            request.report_path,
            request.run_health,
            request.run_dir,
            request.resumed,
        ),
        request.results.clone(),
    );
    fs::write(
        request.run_dir.join("report.html"),
        harnesslab_report::render_html(&model)?,
    )?;
    Ok(())
}

fn capability_effective_summary(policy: &ResolvedCapabilityPolicy) -> String {
    let unsupported = policy
        .unsupported_reason()
        .map(|reason| format!("; unsupported_reason={reason}"))
        .unwrap_or_default();
    format!(
        "effective={}; candidate_effective={}; enforcement={}{}",
        list_text(&policy.effective),
        list_text(&policy.candidate_effective),
        if policy.unsupported_reason().is_some() {
            "unsupported"
        } else {
            "enforced"
        },
        unsupported
    )
}

fn version_probe_summary(snapshot: Option<&AgentVersionSnapshot>) -> String {
    let Some(snapshot) = snapshot else {
        return "not configured".to_string();
    };
    format!(
        "status={}; exit_code={:?}; termination_reason={:?}; stdout_tail={:?}; stderr_tail={:?}; message={}",
        version_status_text(snapshot.status),
        snapshot.exit_code,
        snapshot.termination_reason,
        snapshot.stdout_tail,
        snapshot.stderr_tail,
        snapshot.message
    )
}

fn version_status_text(status: VersionProbeStatus) -> &'static str {
    match status {
        VersionProbeStatus::Ok => "ok",
        VersionProbeStatus::Warning => "warning",
        VersionProbeStatus::Error => "error",
    }
}

fn list_text(values: &[String]) -> String {
    if values.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", values.join(", "))
    }
}
