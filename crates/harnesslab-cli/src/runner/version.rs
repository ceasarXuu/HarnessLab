use super::store;
use crate::agent_registry::{AgentVersionSnapshot, probe_agent_version};
use anyhow::Result;
use harnesslab_core::{AgentProfile, redact_public_value};
use harnesslab_infra::{append_event, event};
use std::path::Path;

pub(super) fn probe_profile_version(
    profile: &AgentProfile,
    run_dir: &Path,
) -> Result<Option<AgentVersionSnapshot>> {
    probe_profile_version_with_extra_secrets(profile, run_dir, &[])
}

pub(super) fn probe_profile_version_with_extra_secrets(
    profile: &AgentProfile,
    run_dir: &Path,
    extra_secret_refs: &[String],
) -> Result<Option<AgentVersionSnapshot>> {
    let secrets = store::secret_values(profile);
    let secret_refs = store::combined_secret_refs(&secrets, extra_secret_refs);
    probe_agent_version(
        profile,
        run_dir,
        &run_dir.join("agent-version-probe"),
        &secret_refs,
    )
}

pub(super) fn append_replay_version_warning(
    source_run_dir: &Path,
    replay_run_dir: &Path,
    run_id: &str,
    current_snapshot: Option<&AgentVersionSnapshot>,
    extra_secret_refs: &[String],
) -> Result<()> {
    let secret_refs = extra_secret_refs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let Some(source_snapshot) = store::load_agent_version_snapshot(source_run_dir)? else {
        append_event(
            &replay_run_dir.join("events.jsonl"),
            &event(
                run_id,
                None,
                "agent_version_compare_skipped",
                "replay warning: source run has no agent-version.snapshot.json; version_command comparison skipped",
            ),
            &[],
        )?;
        return Ok(());
    };
    let Some(current_snapshot) = current_snapshot else {
        append_event(
            &replay_run_dir.join("events.jsonl"),
            &event(
                run_id,
                None,
                "agent_version_compare_skipped",
                "replay warning: current profile has no version_command probe; version_command comparison skipped",
            ),
            &[],
        )?;
        return Ok(());
    };
    if source_snapshot.is_mismatch(current_snapshot) {
        append_event(
            &replay_run_dir.join("events.jsonl"),
            &event(
                run_id,
                None,
                "agent_version_mismatch",
                &format!(
                    "replay warning: current version_command probe differs from source run snapshot; source={}; current={}",
                    snapshot_summary(&source_snapshot, &secret_refs),
                    snapshot_summary(current_snapshot, &secret_refs)
                ),
            ),
            &[],
        )?;
    }
    Ok(())
}

fn snapshot_summary(snapshot: &AgentVersionSnapshot, secrets: &[&str]) -> String {
    redact_public_value(
        &format!(
            "status={:?},exit_code={:?},termination_reason={:?},stdout_tail={:?},stderr_tail={:?}",
            snapshot.status,
            snapshot.exit_code,
            snapshot.termination_reason,
            snapshot.stdout_tail,
            snapshot.stderr_tail
        ),
        secrets,
    )
}
