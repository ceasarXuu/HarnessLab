mod cleanup;
mod replay;
mod sandbox;
mod shell;

use crate::benchmark_data::{ensure_split_runnable, resolve_benchmarks_dir};
use crate::output::{PathOutput, RunOutput};
use crate::print_json;
use anyhow::{Context, Result, bail};
use cleanup::RunSandboxCleanup;
use harnesslab_adapters::adapter_for_with_root;
use harnesslab_core::{
    AgentProfile, BenchmarkRef, EvaluationRecord, FailureClass, FailureCode, GlobalConfig, Outcome,
    PatchRecord, PatchStatus, RunPaths, RunSpec, TaskAttemptResult, TaskPlan, TaskState,
    UsageRecord, classify_agent_process, classify_evaluation_process, derive_exit_code,
    is_valid_profile_name, summarize_results, validate_global_config, validate_run_spec,
};
use harnesslab_infra::{
    ExecSpec, HostProcessExecutor, append_event, atomic_write_json, collect_artifacts,
    command_exists, event, first_command_word, read_json,
};
use replay::{replay_plan_from_source, replay_spec_from_source};
use sandbox::run_agent;
use shell::run_shell;
use std::any::Any;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use time::OffsetDateTime;

#[cfg(test)]
use sandbox::{docker_create_request, render_command, task_requires_docker};

#[derive(Debug, Clone)]
struct AttemptWork {
    task: TaskPlan,
    attempt: u32,
}

pub(crate) fn execute_new_run(
    home: &Path,
    agent_name: &str,
    benchmark_name: &str,
    split: &str,
    json: bool,
    replay_source: Option<String>,
) -> Result<i32> {
    let config = load_config(home)?;
    validate_global_config(&config)?;
    let profile = load_profile(home, agent_name)?;
    profile.validate()?;
    let benchmark_root = resolve_benchmarks_dir(home, Some(&config));
    let adapter = adapter_for_with_root(benchmark_name, benchmark_root.as_deref())
        .with_context(|| format!("unknown benchmark {benchmark_name}"))?;
    ensure_split_runnable(adapter.as_ref(), benchmark_name, split)?;
    let plan = adapter.plan(split).map_err(anyhow::Error::msg)?;
    let run_id = format!(
        "{}-{}-{}-{}",
        agent_name,
        benchmark_name,
        split,
        timestamp_id()
    );
    let run_dir = home.join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let spec = RunSpec {
        schema_version: 1,
        run_id: run_id.clone(),
        created_at: now_rfc3339(),
        agent_profile_ref: agent_name.to_string(),
        benchmark: BenchmarkRef {
            name: benchmark_name.to_string(),
            version: plan.benchmark.version.clone(),
            split: split.to_string(),
        },
        execution: harnesslab_core::ExecutionConfig {
            concurrency: config.default_concurrency,
            attempts: config.default_attempts,
            network: config.network_default,
            timeout_sec: None,
        },
        paths: RunPaths {
            run_dir: run_dir.display().to_string(),
        },
        replay_source_run_id: replay_source,
    };
    validate_run_spec(&spec)?;
    write_run_inputs(&run_dir, &spec, &profile, &plan)?;
    let code = execute_plan(&run_dir, &spec, &profile, &plan)?;
    if json {
        print_json(&RunOutput {
            schema_version: 1,
            command: "run",
            status: if code == 0 { "success" } else { "failure" },
            run_id,
            run_dir: run_dir.display().to_string(),
            replay_source_run_id: spec.replay_source_run_id,
        })?;
    } else {
        println!("run: {}", run_dir.display());
    }
    Ok(code)
}

pub(crate) fn resume_run(_home: &Path, run_dir: &Path, json: bool) -> Result<i32> {
    let spec: RunSpec = read_json(&run_dir.join("run.json"))?;
    let profile: AgentProfile = read_json(&run_dir.join("agent-profile.snapshot.json"))?;
    let plan: harnesslab_core::BenchmarkPlan = read_json(&run_dir.join("benchmark.snapshot.json"))?;
    validate_run_spec(&spec)?;
    profile.validate()?;
    let code = execute_plan(run_dir, &spec, &profile, &plan)?;
    if json {
        print_json(&PathOutput {
            schema_version: 1,
            command: "run resume",
            status: "accepted",
            run_dir: run_dir.display().to_string(),
        })?;
    } else {
        println!("run resume: {}", run_dir.display());
    }
    Ok(code)
}

pub(crate) fn replay_run(home: &Path, source: &Path, json: bool) -> Result<i32> {
    let source_spec: RunSpec = read_json(&source.join("run.json"))?;
    let profile: AgentProfile = read_json(&source.join("agent-profile.snapshot.json"))?;
    let plan = replay_plan_from_source(source, &source_spec)?;
    profile.validate()?;
    if let Some(command) = first_command_word(&profile.command)
        && !command_exists(command)
    {
        bail!("replay blocker: required agent command missing: {command}");
    }
    let run_id = format!(
        "{}-{}-{}-replay-{}",
        profile.name,
        source_spec.benchmark.name,
        source_spec.benchmark.split,
        timestamp_id()
    );
    let run_dir = home.join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let spec = replay_spec_from_source(&source_spec, run_id.clone(), now_rfc3339(), &run_dir);
    validate_run_spec(&spec)?;
    write_run_inputs(&run_dir, &spec, &profile, &plan)?;
    let code = execute_plan(&run_dir, &spec, &profile, &plan)?;
    if json {
        print_json(&RunOutput {
            schema_version: 1,
            command: "run",
            status: if code == 0 { "success" } else { "failure" },
            run_id,
            run_dir: run_dir.display().to_string(),
            replay_source_run_id: spec.replay_source_run_id,
        })?;
    } else {
        println!("run: {}", run_dir.display());
    }
    Ok(code)
}

fn execute_plan(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    plan: &harnesslab_core::BenchmarkPlan,
) -> Result<i32> {
    let events = run_dir.join("events.jsonl");
    append_event(
        &events,
        &event(&spec.run_id, None, "run_started", "run started"),
        &[],
    )?;
    let _sandbox_cleanup = RunSandboxCleanup::start(run_dir, spec, plan);
    let (mut attempts, pending) = partition_attempts(run_dir, plan, spec.execution.attempts)?;
    attempts.extend(execute_attempts(
        run_dir,
        spec,
        profile,
        pending,
        spec.execution.concurrency,
    )?);
    attempts.sort_by(|left, right| {
        left.task_id
            .cmp(&right.task_id)
            .then(left.attempt.cmp(&right.attempt))
    });
    let results = summarize_results(&spec.run_id, attempts);
    atomic_write_json(&run_dir.join("results.json"), &results)?;
    let model = harnesslab_report::build_report_model(
        &spec.run_id,
        &spec.agent_profile_ref,
        &spec.benchmark.name,
        &spec.benchmark.split,
        results.clone(),
    );
    fs::write(
        run_dir.join("report.html"),
        harnesslab_report::render_html(&model)?,
    )?;
    append_event(
        &events,
        &event(&spec.run_id, None, "run_finished", "run finished"),
        &[],
    )?;
    Ok(derive_exit_code(&results.tasks, false))
}

fn execute_attempts(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    attempts: Vec<AttemptWork>,
    concurrency: usize,
) -> Result<Vec<TaskAttemptResult>> {
    let mut results = Vec::new();
    for chunk in attempts.chunks(concurrency.max(1)) {
        let mut handles = Vec::new();
        for work in chunk.iter().cloned() {
            let run_dir = run_dir.to_path_buf();
            let profile = profile.clone();
            let spec = spec.clone();
            handles.push(thread::spawn(move || {
                execute_task(&run_dir, &spec, &profile, &work.task, work.attempt)
            }));
        }
        let mut first_error = None;
        for handle in handles {
            match handle.join() {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(panic) => {
                    if first_error.is_none() {
                        first_error =
                            Some(anyhow::anyhow!("task panicked: {}", panic_message(panic)));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
    }
    Ok(results)
}

fn panic_message(panic: Box<dyn Any + Send + 'static>) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

fn partition_attempts(
    run_dir: &Path,
    plan: &harnesslab_core::BenchmarkPlan,
    attempts: u32,
) -> Result<(Vec<TaskAttemptResult>, Vec<AttemptWork>)> {
    let mut completed = Vec::new();
    let mut pending = Vec::new();
    for work in planned_attempts(plan, attempts) {
        let result_path = attempt_result_path(run_dir, &work.task.task_id, work.attempt);
        if result_path.exists() {
            completed.push(read_json(&result_path)?);
        } else {
            pending.push(work);
        }
    }
    Ok((completed, pending))
}

fn planned_attempts(plan: &harnesslab_core::BenchmarkPlan, attempts: u32) -> Vec<AttemptWork> {
    let mut work = Vec::new();
    for task in &plan.tasks {
        for attempt in 1..=attempts.max(1) {
            work.push(AttemptWork {
                task: task.clone(),
                attempt,
            });
        }
    }
    work
}

fn execute_task(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    task: &TaskPlan,
    attempt: u32,
) -> Result<TaskAttemptResult> {
    let started = std::time::Instant::now();
    let attempt_dir = run_dir
        .join("tasks")
        .join(&task.task_id)
        .join("attempts")
        .join(attempt.to_string());
    let workspace = attempt_dir.join("workspace");
    fs::create_dir_all(&workspace)?;
    prepare_workspace(&workspace, task)?;
    atomic_write_json(
        &run_dir
            .join("tasks")
            .join(&task.task_id)
            .join("task.snapshot.json"),
        task,
    )?;
    let agent_run = run_agent(spec, profile, task, attempt, &attempt_dir, &workspace)?;
    let agent_failure = agent_run.sandbox_failure.map_or_else(
        || classify_agent_process(&agent_run.process),
        |code| harnesslab_core::Failure {
            class: FailureClass::Execution,
            code: Some(code),
            message: "sandbox failed".to_string(),
        },
    );
    let (evaluation, patch, failure_class, failure_code, score) =
        if agent_failure.class == FailureClass::Execution {
            (None, None, agent_failure.class, agent_failure.code, 0.0)
        } else {
            let patch = capture_patch(&workspace, &attempt_dir, task)?;
            let evaluation = run_verifier(&workspace, &attempt_dir, task)?;
            let failure =
                patch_failure(&patch).unwrap_or_else(|| classify_evaluation_process(&evaluation));
            let score = if failure.class == FailureClass::None {
                1.0
            } else {
                0.0
            };
            (Some(evaluation), patch, failure.class, failure.code, score)
        };
    let _ = collect_artifacts(
        &workspace,
        &attempt_dir.join("artifacts"),
        &task.artifact_spec.required_paths,
    );
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: task.task_id.clone(),
        attempt,
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
        benchmark_score: score,
        duration_ms: started.elapsed().as_millis() as u64,
        agent: Some(agent_run.process),
        evaluation,
        patch,
        usage: UsageRecord::Unknown,
        warnings: vec![FailureCode::UsageUnknown],
    };
    atomic_write_json(&attempt_dir.join("result.json"), &result)?;
    Ok(result)
}

fn attempt_result_path(run_dir: &Path, task_id: &str, attempt: u32) -> std::path::PathBuf {
    run_dir
        .join("tasks")
        .join(task_id)
        .join("attempts")
        .join(attempt.to_string())
        .join("result.json")
}

fn prepare_workspace(workspace: &Path, task: &TaskPlan) -> Result<()> {
    if task.patch_spec.is_some() {
        fs::write(workspace.join("app.txt"), "old\n")?;
        run_shell(
            workspace,
            "git init -q && git config user.email harnesslab@example.invalid && git config user.name HarnessLab && git add app.txt && git commit -q -m init",
        )?;
    }
    Ok(())
}

fn run_verifier(workspace: &Path, attempt_dir: &Path, task: &TaskPlan) -> Result<EvaluationRecord> {
    let result = HostProcessExecutor::exec(&ExecSpec {
        command: task.verifier_spec.command.clone(),
        stdin: None,
        working_dir: workspace.to_path_buf(),
        timeout_sec: task.verifier_spec.timeout_sec,
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

fn capture_patch(
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
        serde_json::json!({"instance_id": task.task_id, "patch": String::from_utf8_lossy(&output.stdout)}).to_string(),
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

fn patch_failure(patch: &Option<PatchRecord>) -> Option<harnesslab_core::Failure> {
    match patch.as_ref().map(|patch| patch.status) {
        Some(PatchStatus::Empty) => Some(harnesslab_core::Failure {
            class: FailureClass::Benchmark,
            code: Some(FailureCode::NoValidDiff),
            message: "no diff captured".to_string(),
        }),
        _ => None,
    }
}

fn load_config(home: &Path) -> Result<GlobalConfig> {
    Ok(toml::from_str(&fs::read_to_string(
        home.join("config.toml"),
    )?)?)
}

fn load_profile(home: &Path, name: &str) -> Result<AgentProfile> {
    if !is_valid_profile_name(name) {
        bail!("invalid agent profile name: {name}");
    }
    Ok(toml::from_str(&fs::read_to_string(
        home.join("agents").join(format!("{name}.toml")),
    )?)?)
}

fn write_run_inputs(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    plan: &harnesslab_core::BenchmarkPlan,
) -> Result<()> {
    atomic_write_json(&run_dir.join("run.json"), spec)?;
    atomic_write_json(&run_dir.join("agent-profile.snapshot.json"), profile)?;
    atomic_write_json(&run_dir.join("benchmark.snapshot.json"), plan)?;
    Ok(())
}

fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn timestamp_id() -> String {
    now_rfc3339()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
