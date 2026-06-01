use super::ExternalTaskExecution;
use anyhow::{Context, Result, bail};
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

pub(super) fn execute(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    source_path: Option<&Path>,
) -> Result<TaskAttemptResult> {
    let source_path = source_path.context("swe-bench-pro external runner missing source_path")?;
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
    let instance = match load_instance(dataset_path, &ctx.task.task_id, &swe_dir) {
        Ok(instance) => instance,
        Err(error) => {
            return setup_failure_result(ctx, &format!("metadata extraction failed: {error}"));
        }
    };
    append_swe_event(
        ctx,
        "external_runner_workspace_started",
        "swe-bench-pro workspace prep",
    )?;
    if let Err(error) = prepare_workspace(&workspace, &swe_dir, &instance) {
        return setup_failure_result(ctx, &format!("workspace preparation failed: {error}"));
    }
    let agent_task = task_with_real_instruction(ctx.task, &instance);
    let agent_run = agent::run_agent(ctx, &agent_task, &workspace, &instance)?;
    let agent = agent_run.process;
    let agent_failure = agent_run
        .sandbox_failure
        .map(|code| (FailureClass::Execution, Some(code)))
        .unwrap_or_else(|| {
            let failure = classify_agent_process(&agent);
            (failure.class, failure.code)
        });
    let (evaluation, patch, failure_class, failure_code, score) =
        if agent_failure.0 == FailureClass::Execution {
            (
                missing_evaluation(ctx.attempt_dir, "agent failed before evaluator")?,
                None,
                agent_failure.0,
                agent_failure.1,
                0.0,
            )
        } else {
            let patch = capture_prediction(&workspace, ctx.attempt_dir, ctx.task, &instance)?;
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
                let evaluation = run_evaluator(source_path, &swe_dir, ctx.attempt_dir, ctx.task)?;
                if let Some(parse_error) = &evaluation.parse_error {
                    append_swe_event(ctx, "external_result_parse_failed", parse_error)?;
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
    atomic_write_json(&ctx.attempt_dir.join("result.json"), &result)?;
    Ok(result)
}

fn append_swe_event(ctx: &ExternalTaskExecution<'_>, name: &str, message: &str) -> Result<()> {
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(&ctx.spec.run_id, Some(&ctx.task.task_id), name, message),
        &[],
    )
}

fn setup_failure_result(
    ctx: &ExternalTaskExecution<'_>,
    message: &str,
) -> Result<TaskAttemptResult> {
    append_swe_event(ctx, "external_runner_setup_failed", message)?;
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: ctx.task.task_id.clone(),
        attempt: ctx.attempt,
        provenance: ctx.provenance,
        state: TaskState::Failure,
        outcome: Outcome::Failure,
        failure_class: FailureClass::Execution,
        failure_code: Some(FailureCode::WorkspacePrepFailed),
        health_impact: health_impact_for_failure(
            FailureClass::Execution,
            Some(FailureCode::WorkspacePrepFailed),
        ),
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

#[derive(Debug, Clone)]
struct SweInstance {
    instance_id: String,
    repo: String,
    base_commit: String,
    dockerhub_tag: String,
    problem_statement: String,
    requirements: String,
    gold_patch: String,
}

fn load_instance(dataset_path: &Path, task_id: &str, swe_dir: &Path) -> Result<SweInstance> {
    let raw_sample = swe_dir.join("raw_sample.jsonl");
    let instance_json = swe_dir.join("instance.json");
    let script_path = swe_dir.join("extract_instance.py");
    fs::write(&script_path, extract_script())?;
    let parquet = first_parquet(dataset_path).context("swe-bench-pro parquet data is missing")?;
    let process = HostProcessExecutor::exec(&ExecSpec {
        command: format!(
            "unset PYTHONHOME PYTHONPATH PYTHONUSERBASE; export PYTHONNOUSERSITE=1; uv run --with pandas --with pyarrow python {} {} {} {} {}",
            shell_quote(&script_path.display().to_string()),
            shell_quote(&parquet.display().to_string()),
            shell_quote(task_id),
            shell_quote(&raw_sample.display().to_string()),
            shell_quote(&instance_json.display().to_string())
        ),
        stdin: None,
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 300,
        no_output_timeout_sec: None,
        stdout_path: swe_dir.join("metadata.stdout.log"),
        stderr_path: swe_dir.join("metadata.stderr.log"),
    })?;
    if process.exit_code != Some(0) {
        bail!("failed to extract SWE-bench Pro instance metadata");
    }
    let value: Value = serde_json::from_slice(&fs::read(instance_json)?)?;
    Ok(SweInstance {
        instance_id: json_string(&value, "instance_id")?,
        repo: json_string(&value, "repo")?,
        base_commit: json_string(&value, "base_commit")?,
        dockerhub_tag: json_string(&value, "dockerhub_tag")?,
        problem_statement: json_string(&value, "problem_statement")?,
        requirements: json_string(&value, "requirements")?,
        gold_patch: json_string(&value, "patch")?,
    })
}

fn extract_script() -> &'static str {
    r#"
import json
import pandas as pd
import sys

parquet, instance_id, raw_sample_path, instance_json_path = sys.argv[1:5]
df = pd.read_parquet(parquet)
matches = df[df["instance_id"] == instance_id]
if matches.empty:
    raise SystemExit(f"instance_id not found: {instance_id}")
row = matches.iloc[0].where(pd.notna(matches.iloc[0]), "")
record = row.to_dict()
with open(raw_sample_path, "w") as f:
    f.write(json.dumps(record) + "\n")
keys = ["instance_id", "repo", "base_commit", "dockerhub_tag", "problem_statement", "requirements", "patch"]
with open(instance_json_path, "w") as f:
    json.dump({key: str(record.get(key, "")) for key in keys}, f)
"#
}

fn first_parquet(dataset_path: &Path) -> Option<std::path::PathBuf> {
    let mut files = fs::read_dir(dataset_path.join("data"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "parquet"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    files.into_iter().next()
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
    let command = format!(
        "set -e; {}; image={}; docker pull --platform linux/amd64 \"$image\"; cid=$(docker create --platform linux/amd64 \"$image\"); trap 'docker rm -f \"$cid\" >/dev/null 2>&1 || true' EXIT; docker cp \"$cid:/app/.\" {}; cd {}; git config user.email harnesslab@example.invalid; git config user.name HarnessLab",
        docker_host_prefix(),
        shell_quote(&docker_image(instance)),
        shell_quote(&workspace.display().to_string()),
        shell_quote(&workspace.display().to_string())
    );
    let process = HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 1800,
        no_output_timeout_sec: None,
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
    let command = format!(
        "set -e; {}; unset PYTHONHOME PYTHONPATH PYTHONUSERBASE; export PYTHONNOUSERSITE=1; uv run --with pandas --with tqdm --with docker python {} --raw_sample_path {} --patch_path {} --output_dir {} --scripts_dir {} --dockerhub_username jefzda --use_local_docker --docker_platform linux/amd64 --num_workers 1 --redo",
        docker_host_prefix(),
        shell_quote(
            &source_path
                .join("swe_bench_pro_eval.py")
                .display()
                .to_string()
        ),
        shell_quote(&swe_dir.join("raw_sample.jsonl").display().to_string()),
        shell_quote(
            &attempt_root
                .join("prediction.eval.json")
                .display()
                .to_string()
        ),
        shell_quote(&eval_dir.display().to_string()),
        shell_quote(&source_path.join("run_scripts").display().to_string()),
    );
    let process = HostProcessExecutor::exec(&ExecSpec {
        command,
        stdin: None,
        working_dir: source_path.to_path_buf(),
        timeout_sec: task.verifier_spec.timeout_sec,
        no_output_timeout_sec: None,
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

fn missing_evaluation(attempt_dir: &Path, message: &str) -> Result<EvaluationRecord> {
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

fn json_string(value: &Value, key: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .with_context(|| format!("missing string field {key}"))
}

fn docker_host_prefix() -> &'static str {
    "if [ -z \"${DOCKER_HOST:-}\" ] && [ -S \"$HOME/.colima/default/docker.sock\" ]; then export DOCKER_HOST=\"unix://$HOME/.colima/default/docker.sock\"; fi"
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
