use anyhow::Result;
use harnesslab_core::{EvaluationRecord, TaskPlan};
use harnesslab_infra::{ExecSpec, HostProcessExecutor};
use std::path::Path;

pub(super) fn run_verifier(
    workspace: &Path,
    attempt_dir: &Path,
    task: &TaskPlan,
) -> Result<EvaluationRecord> {
    let result = HostProcessExecutor::exec(&ExecSpec {
        command: task.verifier_spec.command.clone(),
        stdin: None,
        working_dir: workspace.to_path_buf(),
        timeout_sec: task.verifier_spec.timeout_sec,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        stdout_path: attempt_dir.join("verifier/stdout.log"),
        stderr_path: attempt_dir.join("verifier/stderr.log"),
    })?;
    Ok(EvaluationRecord {
        exit_code: result.exit_code,
        raw_score: if task
            .verifier_spec
            .expected_exit_codes
            .contains(&result.exit_code.unwrap_or(-1))
        {
            1.0
        } else {
            0.0
        },
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    })
}
