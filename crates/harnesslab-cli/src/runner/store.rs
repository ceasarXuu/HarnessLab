use anyhow::{Result, bail};
use harnesslab_core::{
    AgentProfile, GlobalConfig, RunSpec, is_valid_profile_name, redact_known_secret,
    redacted_profile_snapshot,
};
use harnesslab_infra::{append_event, atomic_write_json, event, read_json};
use std::fs;
use std::path::Path;
use time::OffsetDateTime;

pub(super) const RUNTIME_PROFILE_SNAPSHOT: &str = "agent-profile.runtime.json";
pub(super) const REPORT_PROFILE_SNAPSHOT: &str = "agent-profile.snapshot.json";
const ORIGINAL_COMMAND_UNAVAILABLE: &str = "[ORIGINAL_COMMAND_UNAVAILABLE]";

#[derive(Debug)]
pub(super) enum ProfileSnapshotSource {
    Runtime,
}

impl ProfileSnapshotSource {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::Runtime => RUNTIME_PROFILE_SNAPSHOT,
        }
    }
}

pub(super) fn load_config(home: &Path) -> Result<GlobalConfig> {
    Ok(toml::from_str(&fs::read_to_string(
        home.join("config.toml"),
    )?)?)
}

pub(super) fn load_profile(home: &Path, name: &str) -> Result<AgentProfile> {
    if !is_valid_profile_name(name) {
        bail!("invalid agent profile name: {name}");
    }
    Ok(toml::from_str(&fs::read_to_string(
        home.join("agents").join(format!("{name}.toml")),
    )?)?)
}

pub(super) fn load_run_profile(run_dir: &Path) -> Result<(AgentProfile, ProfileSnapshotSource)> {
    let runtime = run_dir.join(RUNTIME_PROFILE_SNAPSHOT);
    if runtime.exists() {
        return Ok((read_json(&runtime)?, ProfileSnapshotSource::Runtime));
    }
    bail!("runtime profile snapshot missing; cannot safely resume/replay")
}

pub(super) fn load_report_profile(run_dir: &Path) -> Result<AgentProfile> {
    read_json(&run_dir.join(REPORT_PROFILE_SNAPSHOT))
}

pub(super) fn public_profile_snapshot(profile: &AgentProfile) -> AgentProfile {
    let secrets = secret_values(profile);
    let secret_refs = secrets.iter().map(String::as_str).collect::<Vec<_>>();
    redacted_profile_snapshot(profile, &secret_refs)
}

pub(super) fn write_run_inputs(
    run_dir: &Path,
    spec: &RunSpec,
    runtime_profile: &AgentProfile,
    report_profile: &AgentProfile,
    plan: &harnesslab_core::BenchmarkPlan,
    original_command: &str,
) -> Result<()> {
    let secrets = secret_values(runtime_profile);
    let secret_refs = secrets.iter().map(String::as_str).collect::<Vec<_>>();
    atomic_write_json(&run_dir.join("run.json"), spec)?;
    atomic_write_json(&run_dir.join(RUNTIME_PROFILE_SNAPSHOT), runtime_profile)?;
    restrict_runtime_snapshot(&run_dir.join(RUNTIME_PROFILE_SNAPSHOT))?;
    atomic_write_json(&run_dir.join(REPORT_PROFILE_SNAPSHOT), report_profile)?;
    atomic_write_json(&run_dir.join("benchmark.snapshot.json"), plan)?;
    fs::write(
        run_dir.join("command.txt"),
        root_command_snapshot(spec, report_profile, original_command, &secret_refs),
    )?;
    Ok(())
}

pub(super) fn secret_values(profile: &AgentProfile) -> Vec<String> {
    profile
        .auth
        .inherit_env
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .filter(|value| !value.is_empty())
        .collect()
}

pub(super) fn root_command_snapshot(
    spec: &RunSpec,
    profile: &AgentProfile,
    original_command: &str,
    secrets: &[&str],
) -> String {
    format!(
        "original_command={}\nreplay_command={}\nagent_profile={}\nagent_kind={:?}\nagent_runtime_snapshot={}\nagent_report_snapshot={}\nagent_command_template={}\n",
        redact_known_secret(original_command, secrets),
        replay_command(spec),
        profile.name,
        profile.kind,
        RUNTIME_PROFILE_SNAPSHOT,
        REPORT_PROFILE_SNAPSHOT,
        redact_known_secret(&profile.command, secrets)
    )
}

pub(super) fn original_run_command(
    home: &Path,
    agent: &str,
    benchmark: &str,
    split: &str,
) -> String {
    format!(
        "harnesslab --home {} run --agent {} --benchmark {} --split {}",
        shell_quote(&home.display().to_string()),
        shell_quote(agent),
        shell_quote(benchmark),
        shell_quote(split)
    )
}

pub(super) fn original_replay_command(home: &Path, source: &Path) -> String {
    format!(
        "harnesslab --home {} run replay {}",
        shell_quote(&home.display().to_string()),
        shell_quote(&source.display().to_string())
    )
}

pub(super) fn replay_command(spec: &RunSpec) -> String {
    format!("harnesslab run replay {}", shell_quote(&spec.paths.run_dir))
}

pub(super) fn original_command_from_snapshot(run_dir: &Path) -> String {
    fs::read_to_string(run_dir.join("command.txt"))
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find_map(|line| line.strip_prefix("original_command=").map(str::to_string))
        })
        .unwrap_or_else(|| ORIGINAL_COMMAND_UNAVAILABLE.to_string())
}

pub(super) fn log_profile_snapshot_loaded(
    run_dir: &Path,
    run_id: &str,
    source: &str,
    mode: &str,
) -> Result<()> {
    append_event(
        &run_dir.join("events.jsonl"),
        &event(
            run_id,
            None,
            "profile_snapshot_loaded",
            &format!("loaded {source} for {mode}"),
        ),
        &[],
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(super) fn agent_config_summary(spec: &RunSpec, profile: &AgentProfile) -> String {
    format!(
        "kind={:?}; input_mode={:?}; timeout_sec={}; concurrency={}; attempts={}; network={:?}; command={}",
        profile.kind,
        profile.input_mode,
        profile.timeout_sec,
        spec.execution.concurrency,
        spec.execution.attempts,
        spec.execution.network,
        profile.command
    )
}

pub(super) fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

pub(super) fn timestamp_id() -> String {
    now_rfc3339()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

#[cfg(unix)]
fn restrict_runtime_snapshot(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_runtime_snapshot(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_original_command_is_explicit() {
        let tmp = tempfile::tempdir().unwrap();

        assert_eq!(
            original_command_from_snapshot(tmp.path()),
            ORIGINAL_COMMAND_UNAVAILABLE
        );
    }

    #[test]
    fn missing_runtime_snapshot_is_rejected_without_heuristics() {
        let tmp = tempfile::tempdir().unwrap();
        let mut profile = harnesslab_core::default_agent_profile(
            "fake",
            harnesslab_core::AgentKind::Fake,
            "agent",
        );
        profile.auth.inherit = false;
        atomic_write_json(&tmp.path().join(REPORT_PROFILE_SNAPSHOT), &profile).unwrap();

        let error = load_run_profile(tmp.path()).unwrap_err().to_string();

        assert!(error.contains("runtime profile snapshot missing"));
    }
}
