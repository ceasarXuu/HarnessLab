use crate::agent_registry::{AgentVersionSnapshot, MaterializedAgentProfile, VersionProbeStatus};
use anyhow::Result;
use harnesslab_core::{AgentProfile, ResolvedCapabilityPolicy, RunSpec};
use std::fs;
use std::path::Path;

pub(super) fn build_report_context(
    request: &ReportWriteRequest<'_>,
) -> harnesslab_report::ReportContext {
    harnesslab_report::ReportContext {
        run_id: "[PRIVATE_RUN_ID]".to_string(),
        agent: request.spec.agent_profile_ref.clone(),
        agent_config_summary: super::store::agent_config_summary(
            request.spec,
            request.report_profile,
            request.report_materialized,
        ),
        setup_summary: request.report_materialized.setup_summary.clone(),
        skills_summary: request.report_materialized.skills_summary.clone(),
        tools_summary: request.report_materialized.tools_summary.clone(),
        hooks_summary: request.report_materialized.hooks_summary.clone(),
        skills_effective_summary: capability_effective_summary(
            &request.report_materialized.capabilities.skills,
        ),
        tools_effective_summary: capability_effective_summary(
            &request.report_materialized.capabilities.tools,
        ),
        hooks_effective_summary: capability_effective_summary(
            &request.report_materialized.capabilities.hooks,
        ),
        version_probe_summary: version_probe_summary(request.version_snapshot),
        has_version_probe_snapshot: request.version_snapshot.is_some(),
        benchmark: request.spec.benchmark.name.clone(),
        split: request.spec.benchmark.split.clone(),
        report_path: "report.html".to_string(),
        replay_command: "harnesslab run replay [RUN_DIR]".to_string(),
        original_command: "[PRIVATE_COMMAND]".to_string(),
        resumed: request.resumed,
        run_health_status: request.run_health.status.clone(),
        run_health_reason: request.run_health.reason.clone(),
    }
}

pub(super) struct ReportWriteRequest<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) report_materialized: &'a MaterializedAgentProfile,
    pub(super) version_snapshot: Option<&'a AgentVersionSnapshot>,
    pub(super) run_health: super::monitor::ReportHealth,
    pub(super) resumed: bool,
    pub(super) results: &'a harnesslab_core::RunResults,
}

pub(super) fn write_report(request: ReportWriteRequest<'_>) -> Result<()> {
    let model = harnesslab_report::build_report_model(
        build_report_context(&request),
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
