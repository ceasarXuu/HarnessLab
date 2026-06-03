use anyhow::Result;
use harnesslab_core::{AgentProfile, TerminationReason, redact_known_secret};
use harnesslab_infra::{ExecSpec, HostProcessExecutor};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const DEFAULT_TIMEOUT_SEC: u64 = 5;
const TAIL_LIMIT_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AgentVersionSnapshot {
    pub schema_version: u32,
    pub field: String,
    pub command: String,
    pub status: VersionProbeStatus,
    pub exit_code: Option<i32>,
    pub termination_reason: Option<TerminationReason>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub message: String,
    pub probed_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VersionProbeStatus {
    Ok,
    Warning,
    Error,
}

impl AgentVersionSnapshot {
    pub(crate) fn is_mismatch(&self, other: &Self) -> bool {
        self.command != other.command
            || self.status != other.status
            || self.exit_code != other.exit_code
            || self.termination_reason != other.termination_reason
            || self.stdout_tail != other.stdout_tail
            || self.stderr_tail != other.stderr_tail
    }
}

pub(crate) fn probe_agent_version(
    profile: &AgentProfile,
    work_dir: &Path,
    probe_dir: &Path,
    secrets: &[&str],
) -> Result<Option<AgentVersionSnapshot>> {
    let Some(command) = profile.version_command.as_deref() else {
        return Ok(None);
    };
    Ok(Some(run_version_probe(
        command, work_dir, probe_dir, secrets,
    )?))
}

fn run_version_probe(
    command: &str,
    work_dir: &Path,
    probe_dir: &Path,
    secrets: &[&str],
) -> Result<AgentVersionSnapshot> {
    fs::create_dir_all(probe_dir)?;
    let redacted_command = sanitize_probe_text(command, secrets);
    let probed_at = now_rfc3339();
    if command.trim().is_empty() {
        return Ok(AgentVersionSnapshot {
            schema_version: 1,
            field: "version_command".to_string(),
            command: redacted_command,
            status: VersionProbeStatus::Error,
            exit_code: None,
            termination_reason: None,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            message: "version_command is empty".to_string(),
            probed_at,
        });
    }
    let stdout_path = probe_dir.join("stdout.log");
    let stderr_path = probe_dir.join("stderr.log");
    let process = HostProcessExecutor::exec(&ExecSpec {
        command: command.to_string(),
        stdin: None,
        working_dir: work_dir.to_path_buf(),
        timeout_sec: DEFAULT_TIMEOUT_SEC,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: stdout_path.clone(),
        stderr_path: stderr_path.clone(),
    })?;
    let stdout_tail = sanitize_probe_text(&read_tail(&stdout_path)?, secrets);
    let stderr_tail = sanitize_probe_text(&read_tail(&stderr_path)?, secrets);
    fs::write(&stdout_path, &stdout_tail)?;
    fs::write(&stderr_path, &stderr_tail)?;
    let success =
        process.termination_reason == TerminationReason::Completed && process.exit_code == Some(0);
    let status = if success {
        VersionProbeStatus::Ok
    } else {
        VersionProbeStatus::Warning
    };
    let message = if success {
        "version_command completed successfully".to_string()
    } else {
        format!(
            "version_command probe did not complete successfully; termination_reason={:?}; exit_code={:?}",
            process.termination_reason, process.exit_code
        )
    };
    Ok(AgentVersionSnapshot {
        schema_version: 1,
        field: "version_command".to_string(),
        command: redacted_command,
        status,
        exit_code: process.exit_code,
        termination_reason: Some(process.termination_reason),
        stdout_tail,
        stderr_tail,
        message,
        probed_at,
    })
}

fn read_tail(path: &Path) -> Result<String> {
    let bytes = fs::read(path).unwrap_or_default();
    let start = bytes.len().saturating_sub(TAIL_LIMIT_BYTES);
    Ok(String::from_utf8_lossy(&bytes[start..]).to_string())
}

pub(crate) fn sanitize_probe_text(value: &str, secrets: &[&str]) -> String {
    let redacted = redact_known_secret(value, secrets);
    redacted
        .split_inclusive(char::is_whitespace)
        .map(redact_probe_token)
        .collect()
}

fn redact_probe_token(token: &str) -> String {
    let trimmed = token.trim_end_matches(char::is_whitespace);
    let suffix = &token[trimmed.len()..];
    if is_sensitive_probe_token(trimmed) {
        format!("[REDACTED]{suffix}")
    } else {
        token.to_string()
    }
}

fn is_sensitive_probe_token(token: &str) -> bool {
    let normalized = token
        .trim_matches(|c: char| c == '\'' || c == '"' || c == '`' || c == ';' || c == ',')
        .to_ascii_lowercase();
    normalized.contains("sk-")
        || normalized.contains("github_pat_")
        || normalized.contains("ghp_")
        || normalized.contains("gho_")
        || normalized.contains("ghu_")
        || normalized.contains("ghs_")
        || normalized.contains("xoxb-")
        || normalized.contains("xoxp-")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("access_token")
        || normalized.contains("auth_token")
        || normalized.contains("password")
        || normalized.contains("passwd")
        || normalized.contains("secret")
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agt_reg_010_empty_version_command_is_malformed() {
        let tmp = tempfile::tempdir().unwrap();
        let snapshot = run_version_probe("", tmp.path(), &tmp.path().join("probe"), &[]).unwrap();

        assert_eq!(snapshot.status, VersionProbeStatus::Error);
        assert!(snapshot.message.contains("version_command"));
    }

    #[test]
    fn agt_reg_010_version_probe_redacts_output_tail() {
        let tmp = tempfile::tempdir().unwrap();
        let snapshot = run_version_probe(
            "printf 'version sk-secret'",
            tmp.path(),
            &tmp.path().join("probe"),
            &["sk-secret"],
        )
        .unwrap();

        assert_eq!(snapshot.status, VersionProbeStatus::Ok);
        assert!(snapshot.stdout_tail.contains("[REDACTED]"));
        assert!(!snapshot.stdout_tail.contains("sk-secret"));
    }

    #[test]
    fn agt_reg_010_version_probe_times_out() {
        let tmp = tempfile::tempdir().unwrap();
        let snapshot =
            run_version_probe("sleep 10", tmp.path(), &tmp.path().join("probe"), &[]).unwrap();

        assert_eq!(snapshot.status, VersionProbeStatus::Warning);
        assert_eq!(
            snapshot.termination_reason,
            Some(TerminationReason::Timeout)
        );
        assert!(snapshot.message.contains("termination_reason=Timeout"));
    }

    #[test]
    fn agt_reg_010_version_probe_redacts_persisted_logs_without_known_secret() {
        let tmp = tempfile::tempdir().unwrap();
        let probe_dir = tmp.path().join("probe");
        let snapshot = run_version_probe("printf sk-secret", tmp.path(), &probe_dir, &[]).unwrap();

        assert_eq!(snapshot.status, VersionProbeStatus::Ok);
        assert!(snapshot.command.contains("[REDACTED]"));
        assert!(snapshot.stdout_tail.contains("[REDACTED]"));
        assert!(!snapshot.command.contains("sk-secret"));
        assert!(!snapshot.stdout_tail.contains("sk-secret"));
        assert!(
            !fs::read_to_string(probe_dir.join("stdout.log"))
                .unwrap()
                .contains("sk-secret")
        );
    }
}
