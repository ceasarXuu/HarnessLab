use anyhow::Result;
use harnesslab_core::{FailureClass, FailureCode, PatchRecord, PatchStatus, TaskPlan};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn capture_patch(
    workspace: &Path,
    attempt_dir: &Path,
    task: &TaskPlan,
) -> Result<Option<PatchRecord>> {
    if task.patch_spec.is_none() {
        return Ok(None);
    }
    let output = Command::new("git")
        .arg("diff")
        .current_dir(workspace)
        .output()?;
    fs::write(attempt_dir.join("patch.diff"), &output.stdout)?;
    fs::write(
        attempt_dir.join("prediction.jsonl"),
        serde_json::json!({
            "instance_id": task.task_id,
            "patch": String::from_utf8_lossy(&output.stdout)
        })
        .to_string(),
    )?;
    Ok(Some(PatchRecord {
        diff_path: "patch.diff".to_string(),
        prediction_path: Some("prediction.jsonl".to_string()),
        status: if output.stdout.is_empty() {
            PatchStatus::Empty
        } else {
            PatchStatus::Captured
        },
    }))
}

pub(super) fn patch_failure(patch: &Option<PatchRecord>) -> Option<harnesslab_core::Failure> {
    match patch.as_ref().map(|patch| patch.status) {
        Some(PatchStatus::Empty) => Some(harnesslab_core::Failure {
            class: FailureClass::Benchmark,
            code: Some(FailureCode::NoValidDiff),
            message: "no diff captured".to_string(),
        }),
        _ => None,
    }
}
