use super::ExternalTaskExecution;
use super::swe_bench_pro_adapter::SweBenchProRuntimeAttempt;
use anyhow::{Result, bail};
use harnesslab_core::{
    EvaluationRecord, FailureClass, FailureCode, Outcome, PatchRecord, PatchStatus,
    TaskAttemptResult, TaskPlan, TaskState, UsageRecord, classify_agent_process,
    health_impact_for_failure,
};
use harnesslab_infra::{ExecSpec, HostProcessExecutor, append_event, atomic_write_json, event};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

mod agent;
mod metadata;
pub(super) mod runtime_snapshot;
use metadata::{SweInstance, load_instance};

pub(super) fn execute_prepared(
    ctx: &ExternalTaskExecution<'_>,
    attempt: SweBenchProRuntimeAttempt,
) -> Result<TaskAttemptResult> {
    let dataset_path = attempt.dataset_path;
    let source_path = attempt.source_path;
    let compatibility = attempt.compatibility;
    let attempt_root = fs::canonicalize(ctx.attempt_dir)?;
    let workspace = attempt_root.join("workspace");
    fs::create_dir_all(&workspace)?;
    let swe_dir = attempt_root.join("swe-bench-pro");
    fs::create_dir_all(&swe_dir)?;
    append_swe_event(
        ctx,
        "external_runner_started",
        "swe-bench-pro metadata extraction",
    )?;
    append_swe_event(
        ctx,
        "swe_bench_pro_metadata_extraction_started",
        "phase=metadata_extraction status=started",
    )?;
    let instance = match load_instance(&dataset_path, &ctx.task.task_id, &swe_dir) {
        Ok(instance) => instance,
        Err(error) => {
            let message = format!("metadata extraction failed: {error}");
            missing_evaluation(ctx.attempt_dir, &message)?;
            runtime_snapshot::write_swe_setup_failure_snapshots(
                ctx,
                &dataset_path,
                Some(&source_path),
                &swe_dir,
                &workspace,
                runtime_snapshot::SweSetupFailurePhase::MetadataExtraction,
            )?;
            return setup_failure_result(
                ctx,
                "metadata_extraction",
                FailureCode::MetadataExtractionFailed,
                &message,
            );
        }
    };
    append_swe_event(
        ctx,
        "external_runner_workspace_started",
        "swe-bench-pro workspace prep",
    )?;
    append_swe_event(
        ctx,
        "swe_bench_pro_workspace_prep_started",
        "phase=workspace_preparation status=started",
    )?;
    if let Err(error) = prepare_workspace(&workspace, &swe_dir, &instance) {
        let message = format!("workspace preparation failed: {error}");
        missing_evaluation(ctx.attempt_dir, &message)?;
        runtime_snapshot::write_swe_setup_failure_snapshots(
            ctx,
            &dataset_path,
            Some(&source_path),
            &swe_dir,
            &workspace,
            runtime_snapshot::SweSetupFailurePhase::WorkspacePreparation,
        )?;
        return setup_failure_result(
            ctx,
            "workspace_preparation",
            FailureCode::WorkspacePrepFailed,
            &message,
        );
    }
    let agent_task = task_with_real_instruction(ctx.task, &instance);
    append_swe_event(ctx, "external_runner_agent_started", "swe-bench-pro agent")?;
    append_swe_event(
        ctx,
        "swe_bench_pro_agent_started",
        "phase=agent_execution status=started",
    )?;
    let agent_run = agent::run_agent(ctx, &agent_task, &workspace, &instance, &compatibility)?;
    let agent = agent_run.process;
    let agent_failure = agent_run
        .sandbox_failure
        .map(|code| (FailureClass::Execution, Some(code)))
        .unwrap_or_else(|| {
            let failure = classify_agent_process(&agent);
            (failure.class, failure.code)
        });
    let (evaluation, patch, failure_class, failure_code, score) = if agent_failure.0
        == FailureClass::Execution
    {
        (
            missing_evaluation(ctx.attempt_dir, "agent failed before evaluator")?,
            None,
            agent_failure.0,
            agent_failure.1,
            0.0,
        )
    } else {
        append_swe_event(
            ctx,
            "external_runner_patch_started",
            "swe-bench-pro patch capture",
        )?;
        append_swe_event(
            ctx,
            "swe_bench_pro_patch_capture_started",
            "phase=patch_capture status=started",
        )?;
        let patch = capture_prediction(&workspace, ctx.attempt_dir, ctx.task, &instance)?;
        let patch_status = patch_status_label(patch.status);
        append_swe_event(
            ctx,
            "external_runner_patch_captured",
            &format!("phase=patch_capture status={patch_status}"),
        )?;
        append_swe_event(
            ctx,
            "swe_bench_pro_patch_captured",
            &format!("phase=patch_capture status={patch_status}"),
        )?;
        let patch_failure = patch_failure(&patch);
        if let Some(failure) = patch_failure {
            (
                missing_evaluation(ctx.attempt_dir, "no valid diff captured")?,
                Some(patch),
                failure.0,
                failure.1,
                0.0,
            )
        } else {
            append_swe_event(
                ctx,
                "external_runner_evaluator_started",
                "swe-bench-pro evaluator",
            )?;
            append_swe_event(
                ctx,
                "swe_bench_pro_evaluator_started",
                "phase=evaluator status=started",
            )?;
            let evaluation = run_evaluator(&source_path, &swe_dir, ctx.attempt_dir, ctx.task)?;
            if let Some(parse_error) = &evaluation.parse_error {
                append_swe_event(ctx, "external_result_parse_failed", parse_error)?;
                append_swe_event(
                    ctx,
                    "swe_bench_pro_result_parse_failed",
                    &format!(
                        "phase=evaluator parser=eval_results final_failure_class=execution final_failure_code=evaluator_error message={parse_error}"
                    ),
                )?;
            }
            let score = evaluation.record.raw_score;
            let failure = if evaluation.parse_error.is_some() {
                (FailureClass::Execution, Some(FailureCode::EvaluatorError))
            } else if evaluation.record.exit_code == Some(0) && score >= 1.0 {
                (FailureClass::None, None)
            } else if evaluation.record.exit_code == Some(0) {
                (FailureClass::Benchmark, Some(FailureCode::TestFailed))
            } else {
                (FailureClass::Benchmark, Some(FailureCode::EvaluatorError))
            };
            (evaluation.record, Some(patch), failure.0, failure.1, score)
        }
    };
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: if failure_class == FailureClass::None {
            TaskState::Success
        } else {
            TaskState::Failure
        },
        outcome: if failure_class == FailureClass::None {
            Outcome::Success
        } else {
            Outcome::Failure
        },
        failure_class,
        failure_code,
        health_impact: health_impact_for_failure(failure_class, failure_code),
        benchmark_score: score,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: Some(agent),
        evaluation: Some(evaluation),
        patch,
        usage: UsageRecord::Unknown,
        warnings: vec![FailureCode::UsageUnknown],
    };
    runtime_snapshot::write_swe_runtime_snapshots(
        ctx,
        &dataset_path,
        &source_path,
        &swe_dir,
        &workspace,
        &instance,
    )
    .map_err(|error| {
        super::adapter_internal_error(
            "post_execution_snapshot",
            FailureCode::ArtifactCollectionFailed,
            error,
        )
    })?;
    atomic_write_json(&ctx.attempt_dir.join("result.json"), &result).map_err(|error| {
        super::adapter_internal_error(
            "post_execution_result",
            FailureCode::ArtifactCollectionFailed,
            error,
        )
    })?;
    Ok(result)
}

fn append_swe_event(ctx: &ExternalTaskExecution<'_>, name: &str, message: &str) -> Result<()> {
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(&ctx.spec.run_id, Some(&ctx.task.task_id), name, message),
        &[],
    )
}

pub(super) fn setup_failure_result(
    ctx: &ExternalTaskExecution<'_>,
    phase: &str,
    failure_code: FailureCode,
    message: &str,
) -> Result<TaskAttemptResult> {
    append_swe_event(ctx, "external_runner_setup_failed", message)?;
    append_swe_event(
        ctx,
        "swe_bench_pro_setup_failed",
        &format!(
            "phase={phase} failure_class=execution failure_code={} pending_tasks_should_abort=false message={message}",
            failure_code_label(failure_code)
        ),
    )?;
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: TaskState::Failure,
        outcome: Outcome::Failure,
        failure_class: FailureClass::Execution,
        failure_code: Some(failure_code),
        health_impact: health_impact_for_failure(FailureClass::Execution, Some(failure_code)),
        benchmark_score: 0.0,
        duration_ms: ctx.started.elapsed().as_millis() as u64,
        agent: None,
        evaluation: Some(missing_evaluation(ctx.attempt_dir, message)?),
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: vec![FailureCode::UsageUnknown],
    };
    atomic_write_json(&ctx.attempt_dir.join("result.json"), &result)?;
    Ok(result)
}

fn prepare_workspace(workspace: &Path, swe_dir: &Path, instance: &SweInstance) -> Result<()> {
    let manifest = serde_json::json!({
        "instance_id": instance.instance_id,
        "repo": instance.repo,
        "base_commit": instance.base_commit,
        "docker_image": docker_image(instance),
        "workspace": workspace.display().to_string(),
    });
    atomic_write_json(&swe_dir.join("workspace-manifest.json"), &manifest)?;
    let command = runtime_snapshot::workspace_prepare_command(workspace, instance);
    let process = HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 1800,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: swe_dir.join("workspace.stdout.log"),
        stderr_path: swe_dir.join("workspace.stderr.log"),
    })?;
    if process.exit_code == Some(0) {
        Ok(())
    } else {
        bail!("failed to prepare SWE-bench Pro workspace")
    }
}

fn docker_image(instance: &SweInstance) -> String {
    format!("jefzda/sweap-images:{}", instance.dockerhub_tag)
}

fn task_with_real_instruction(task: &TaskPlan, instance: &SweInstance) -> TaskPlan {
    let mut task = task.clone();
    task.instruction = format!(
        "{}\n\nRequirements:\n{}\n\nRepository: {}\nBase commit: {}",
        instance.problem_statement, instance.requirements, instance.repo, instance.base_commit
    );
    task
}

fn capture_prediction(
    workspace: &Path,
    attempt_dir: &Path,
    task: &TaskPlan,
    instance: &SweInstance,
) -> Result<PatchRecord> {
    let output = Command::new("git")
        .arg("diff")
        .current_dir(workspace)
        .output()?;
    atomic_write_json(
        &attempt_dir.join("git-diff.status.json"),
        &serde_json::json!({
            "exit_code": output.status.code(),
            "success": output.status.success()
        }),
    )?;
    fs::write(attempt_dir.join("git-diff.stderr.log"), &output.stderr)?;
    let patch = String::from_utf8_lossy(&output.stdout).to_string();
    fs::write(attempt_dir.join("patch.diff"), &patch)?;
    let prediction = serde_json::json!({
            "instance_id": task.task_id,
            "model_name_or_path": "harnesslab",
            "model_patch": patch,
            "patch": patch,
            "prefix": "harnesslab"
    });
    fs::write(
        attempt_dir.join("prediction.jsonl"),
        format!("{}\n", serde_json::to_string(&prediction)?),
    )?;
    fs::write(
        attempt_dir.join("prediction.eval.json"),
        serde_json::to_string_pretty(&vec![prediction])?,
    )?;
    let _ = instance;
    Ok(PatchRecord {
        diff_path: "patch.diff".to_string(),
        prediction_path: Some("prediction.jsonl".to_string()),
        status: if !output.status.success() {
            PatchStatus::ApplyFailed
        } else if output.stdout.is_empty() {
            PatchStatus::Empty
        } else {
            PatchStatus::Captured
        },
    })
}

fn run_evaluator(
    source_path: &Path,
    swe_dir: &Path,
    attempt_dir: &Path,
    task: &TaskPlan,
) -> Result<SweEvaluation> {
    let attempt_root = fs::canonicalize(attempt_dir)?;
    let eval_dir = swe_dir.join("eval");
    fs::create_dir_all(&eval_dir)?;
    let command = runtime_snapshot::evaluator_command(source_path, swe_dir, &attempt_root);
    let process = HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: source_path.to_path_buf(),
        timeout_sec: task.verifier_spec.timeout_sec,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: attempt_root.join("verifier/stdout.log"),
        stderr_path: attempt_root.join("verifier/stderr.log"),
    })?;
    let score_path = eval_dir.join("eval_results.json");
    let (raw_score, parse_error) = match parse_score(&score_path, &task.task_id) {
        Ok(score) => (score, None),
        Err(error) => {
            let message = format!(
                "swe-bench-pro official eval_results unavailable at {}: {error}",
                score_path.display()
            );
            append_log(&attempt_root.join("verifier/stderr.log"), &message)?;
            (0.0, Some(message))
        }
    };
    Ok(SweEvaluation {
        record: EvaluationRecord {
            exit_code: process.exit_code,
            raw_score,
            stdout_path: "verifier/stdout.log".to_string(),
            stderr_path: "verifier/stderr.log".to_string(),
        },
        parse_error,
    })
}

struct SweEvaluation {
    record: EvaluationRecord,
    parse_error: Option<String>,
}

fn parse_score(path: &Path, task_id: &str) -> Result<f64> {
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    Ok(
        if value.get(task_id).and_then(Value::as_bool).unwrap_or(false) {
            1.0
        } else {
            0.0
        },
    )
}

pub(super) fn missing_evaluation(attempt_dir: &Path, message: &str) -> Result<EvaluationRecord> {
    fs::create_dir_all(attempt_dir.join("verifier"))?;
    fs::write(attempt_dir.join("verifier/stdout.log"), "")?;
    fs::write(attempt_dir.join("verifier/stderr.log"), message)?;
    Ok(EvaluationRecord {
        exit_code: None,
        raw_score: 0.0,
        stdout_path: "verifier/stdout.log".to_string(),
        stderr_path: "verifier/stderr.log".to_string(),
    })
}

fn append_log(path: &Path, message: &str) -> Result<()> {
    let mut content = fs::read_to_string(path).unwrap_or_default();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(message);
    content.push('\n');
    fs::write(path, content)?;
    Ok(())
}

fn patch_failure(patch: &PatchRecord) -> Option<(FailureClass, Option<FailureCode>)> {
    match patch.status {
        PatchStatus::Empty => Some((FailureClass::Benchmark, Some(FailureCode::NoValidDiff))),
        PatchStatus::ApplyFailed => {
            Some((FailureClass::Execution, Some(FailureCode::PatchApplyFailed)))
        }
        _ => None,
    }
}

fn patch_status_label(status: PatchStatus) -> &'static str {
    match status {
        PatchStatus::NotApplicable => "not_applicable",
        PatchStatus::Captured => "captured",
        PatchStatus::Empty => "empty",
        PatchStatus::ApplyFailed => "apply_failed",
    }
}

fn failure_code_label(code: FailureCode) -> &'static str {
    match code {
        FailureCode::MetadataExtractionFailed => "metadata_extraction_failed",
        FailureCode::WorkspacePrepFailed => "workspace_prep_failed",
        FailureCode::ExternalRunnerSetupFailed => "external_runner_setup_failed",
        _ => "other",
    }
}
