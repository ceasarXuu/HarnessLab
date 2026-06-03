use crate::agent_registry::MaterializedAgentProfile;
use anyhow::Result;
use harnesslab_core::{FailureCode, ProcessRecord};
use std::fs;
use std::path::Path;

const SETUP_FAILED_MARKER: &str = "harnesslab agent setup failed:";

pub(super) fn write_snapshot(
    attempt_dir: &Path,
    materialized: &MaterializedAgentProfile,
) -> Result<()> {
    let Some(setup) = &materialized.setup_script else {
        return Ok(());
    };
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::write(agent_dir.join("setup.sh"), setup)?;
    Ok(())
}

pub(super) fn prefix_command(materialized: &MaterializedAgentProfile, command: &str) -> String {
    let Some(setup) = &materialized.setup_script else {
        return command.to_string();
    };
    format!(
        "printf '%s\\n' 'harnesslab agent setup starting' >&2; sh -c {}; setup_status=$?; if [ \"$setup_status\" -ne 0 ]; then printf '%s\\n' \"{SETUP_FAILED_MARKER} exit_code=$setup_status\" >&2; exit \"$setup_status\"; fi; printf '%s\\n' 'harnesslab agent setup completed' >&2; {command}",
        shell_quote(setup)
    )
}

pub(super) fn failure_code(
    attempt_dir: &Path,
    materialized: &MaterializedAgentProfile,
    process: &ProcessRecord,
) -> Option<FailureCode> {
    materialized.setup_script.as_ref()?;
    if process.exit_code == Some(0) {
        return None;
    }
    let stderr = fs::read_to_string(attempt_dir.join("agent/stderr.log")).ok()?;
    stderr
        .contains(SETUP_FAILED_MARKER)
        .then_some(FailureCode::ExternalRunnerSetupFailed)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{RunAs, TerminationReason};

    #[test]
    fn agt_reg_004_setup_prefix_marks_failures_and_writes_snapshot() {
        let tmp = tempfile::tempdir().unwrap();
        let materialized = materialized(Some("echo setup && exit 2"));

        write_snapshot(tmp.path(), &materialized).unwrap();
        let command = prefix_command(&materialized, "agent");

        assert_eq!(
            fs::read_to_string(tmp.path().join("agent/setup.sh")).unwrap(),
            "echo setup && exit 2"
        );
        assert!(command.contains("harnesslab agent setup starting"));
        assert!(command.contains("harnesslab agent setup failed:"));
        assert!(command.contains("agent"));
    }

    #[test]
    fn agt_reg_004_setup_failure_marker_maps_to_setup_failure_code() {
        let tmp = tempfile::tempdir().unwrap();
        let materialized = materialized(Some("exit 2"));
        fs::create_dir_all(tmp.path().join("agent")).unwrap();
        fs::write(
            tmp.path().join("agent/stderr.log"),
            "harnesslab agent setup failed: exit_code=2",
        )
        .unwrap();
        let process = ProcessRecord {
            exit_code: Some(2),
            termination_reason: TerminationReason::Completed,
            stdout_path: "agent/stdout.log".to_string(),
            stderr_path: "agent/stderr.log".to_string(),
        };

        assert_eq!(
            failure_code(tmp.path(), &materialized, &process),
            Some(FailureCode::ExternalRunnerSetupFailed)
        );
    }

    fn materialized(setup_script: Option<&str>) -> MaterializedAgentProfile {
        MaterializedAgentProfile {
            setup_script: setup_script.map(str::to_string),
            setup_summary: "preset=Custom".to_string(),
            skills_summary: "inherit=true".to_string(),
            tools_summary: "inherit=true".to_string(),
            hooks_summary: "inherit=true".to_string(),
            run_as: RunAs::Current,
            warnings: Vec::new(),
        }
    }
}
