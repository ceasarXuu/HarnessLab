mod attempts;
mod cleanup;
mod external;
mod monitor;
mod patch;
mod replay;
mod sandbox;
mod schedule;
mod shell;
mod store;
mod usage;
use crate::benchmark_data::{ensure_split_runnable, resolve_benchmarks_dir};
use crate::output::{PathOutput, RunOutput};
use crate::print_json;
use anyhow::{Context, Result, bail};
use attempts::execute_attempts;
use cleanup::RunSandboxCleanup;
use harnesslab_adapters::adapter_for_with_root;
use harnesslab_core::{
    AgentProfile, AttemptProvenance, BenchmarkRef, EvaluationRecord, FailureClass, FailureCode,
    Outcome, RunPaths, RunSpec, TaskAttemptResult, TaskPlan, TaskState, classify_agent_process,
    classify_evaluation_process, derive_exit_code, summarize_results, task_dir_name,
    validate_benchmark_plan, validate_global_config, validate_run_spec,
};
use harnesslab_infra::{
    ExecSpec, HostProcessExecutor, append_event, atomic_write_json, collect_artifacts,
    command_exists, event, first_command_word, read_json,
};
use patch::{capture_patch, patch_failure};
use replay::{replay_plan_from_source, replay_spec_from_source};
use sandbox::run_agent;
use schedule::partition_attempts;
use shell::run_shell;
use std::fs;
use std::path::Path;
use store::{load_config, load_profile, write_run_inputs};
use usage::collect_usage;
#[derive(Debug, Clone, Copy)]
enum ExecutionMode {
    New,
    Resume,
    Replay,
}
#[cfg(test)]
use {
    attempts::panic_message,
    sandbox::{docker_create_request, render_command, task_requires_docker},
    schedule::{attempt_result_path, planned_attempts},
};
pub(crate) fn execute_new_run(
    home: &Path,
    agent_name: &str,
    benchmark_name: &str,
    split: &str,
    json: bool,
    overrides: RunOverrides,
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
    validate_benchmark_plan(&plan)?;
    external::validate_profile_for_plan(&profile, &plan.tasks)?;
    let run_id = format!(
        "{agent_name}-{benchmark_name}-{split}-{}",
        store::timestamp_id()
    );
    let run_dir = store::runs_dir(home, &config).join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let spec = RunSpec {
        schema_version: 1,
        run_id: run_id.clone(),
        created_at: store::now_rfc3339(),
        agent_profile_ref: agent_name.to_string(),
        benchmark: BenchmarkRef {
            name: benchmark_name.to_string(),
            version: plan.benchmark.version.clone(),
            split: split.to_string(),
        },
        execution: harnesslab_core::ExecutionConfig {
            concurrency: overrides.concurrency.unwrap_or(config.default_concurrency),
            attempts: overrides.attempts.unwrap_or(config.default_attempts),
            network: plan
                .run_config_overrides
                .network
                .unwrap_or(config.network_default),
            timeout_sec: overrides
                .timeout_sec
                .or(plan.run_config_overrides.timeout_sec),
        },
        paths: RunPaths {
            run_dir: run_dir.display().to_string(),
        },
        replay_source_run_id: replay_source,
    };
    validate_run_spec(&spec)?;
    let original_command =
        store::original_run_command(home, agent_name, benchmark_name, split, &spec, &config);
    let report_profile = store::public_profile_snapshot(&profile);
    write_run_inputs(
        &run_dir,
        &spec,
        &profile,
        &report_profile,
        &plan,
        &original_command,
    )?;
    let code = execute_plan(
        &run_dir,
        &spec,
        &profile,
        &report_profile,
        &plan,
        ExecutionMode::New,
    )?;
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
        println!("report: {}", run_dir.join("report.html").display());
    }
    Ok(code)
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RunOverrides {
    pub(crate) concurrency: Option<usize>,
    pub(crate) attempts: Option<u32>,
    pub(crate) timeout_sec: Option<u64>,
}

pub(crate) fn runs_dir(home: &Path, config: &harnesslab_core::GlobalConfig) -> std::path::PathBuf {
    store::runs_dir(home, config)
}

pub(crate) fn resume_run(_home: &Path, run_dir: &Path, json: bool) -> Result<i32> {
    let spec: RunSpec = read_json(&run_dir.join("run.json"))?;
    let (profile, profile_source) = store::load_run_profile(run_dir)?;
    let report_profile = store::load_report_profile(run_dir)?;
    let plan: harnesslab_core::BenchmarkPlan = read_json(&run_dir.join("benchmark.snapshot.json"))?;
    validate_run_spec(&spec)?;
    validate_benchmark_plan(&plan)?;
    profile.validate()?;
    external::validate_profile_for_plan(&profile, &plan.tasks)?;
    store::log_profile_snapshot_loaded(run_dir, &spec.run_id, profile_source.as_str(), "resume")?;
    let code = execute_plan(
        run_dir,
        &spec,
        &profile,
        &report_profile,
        &plan,
        ExecutionMode::Resume,
    )?;
    if json {
        print_json(&PathOutput {
            schema_version: 1,
            command: "run resume",
            status: "accepted",
            run_dir: run_dir.display().to_string(),
        })?;
    } else {
        println!("run resume: {}", run_dir.display());
        println!("report: {}", run_dir.join("report.html").display());
    }
    Ok(code)
}

pub(crate) fn replay_run(home: &Path, source: &Path, json: bool) -> Result<i32> {
    let config = load_config(home)?;
    let source_spec: RunSpec = read_json(&source.join("run.json"))?;
    let (profile, profile_source) = store::load_run_profile(source)?;
    let report_profile = store::load_report_profile(source)?;
    let plan = replay_plan_from_source(source, &source_spec)?;
    validate_benchmark_plan(&plan)?;
    profile.validate()?;
    external::validate_profile_for_plan(&profile, &plan.tasks)?;
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
        store::timestamp_id()
    );
    let run_dir = store::runs_dir(home, &config).join(&run_id);
    fs::create_dir_all(&run_dir)?;
    let spec =
        replay_spec_from_source(&source_spec, run_id.clone(), store::now_rfc3339(), &run_dir);
    validate_run_spec(&spec)?;
    let original_command = store::original_replay_command(home, source);
    write_run_inputs(
        &run_dir,
        &spec,
        &profile,
        &report_profile,
        &plan,
        &original_command,
    )?;
    store::log_profile_snapshot_loaded(&run_dir, &spec.run_id, profile_source.as_str(), "replay")?;
    let code = execute_plan(
        &run_dir,
        &spec,
        &profile,
        &report_profile,
        &plan,
        ExecutionMode::Replay,
    )?;
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
        println!("report: {}", run_dir.join("report.html").display());
    }
    Ok(code)
}

fn execute_plan(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    report_profile: &AgentProfile,
    plan: &harnesslab_core::BenchmarkPlan,
    mode: ExecutionMode,
) -> Result<i32> {
    let events = run_dir.join("events.jsonl");
    if matches!(mode, ExecutionMode::Resume) {
        append_event(
            &events,
            &event(
                &spec.run_id,
                None,
                "run_resumed",
                "run resumed from existing run directory",
            ),
            &[],
        )?;
    }
    append_event(
        &events,
        &event(&spec.run_id, None, "run_started", "run started"),
        &[],
    )?;
    let _sandbox_cleanup = RunSandboxCleanup::start(run_dir, spec, plan);
    let (mut attempts, pending) = partition_attempts(run_dir, plan, spec.execution.attempts)?;
    let pending = pending
        .into_iter()
        .map(|mut work| {
            work.provenance = match mode {
                ExecutionMode::Resume if work.attempt > spec.execution.attempts.max(1) => {
                    AttemptProvenance::Recovery
                }
                ExecutionMode::Resume => AttemptProvenance::Resumed,
                _ => AttemptProvenance::Original,
            };
            work
        })
        .collect::<Vec<_>>();
    for work in &pending {
        if work.attempt > spec.execution.attempts.max(1) {
            append_event(
                &events,
                &event(
                    &spec.run_id,
                    Some(&work.task.task_id),
                    "recovery_attempt_scheduled",
                    &format!("scheduled recovery attempt {}", work.attempt),
                ),
                &[],
            )?;
        }
    }
    attempts.extend(execute_attempts(
        run_dir,
        spec,
        profile,
        report_profile,
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
    let run_health = monitor::report_health(run_dir);
    let model = harnesslab_report::build_report_model(
        harnesslab_report::ReportContext {
            run_id: spec.run_id.clone(),
            agent: spec.agent_profile_ref.clone(),
            agent_config_summary: store::agent_config_summary(spec, report_profile),
            benchmark: spec.benchmark.name.clone(),
            split: spec.benchmark.split.clone(),
            report_path: run_dir.join("report.html").display().to_string(),
            replay_command: store::replay_command(spec),
            original_command: store::original_command_from_snapshot(run_dir),
            resumed: matches!(mode, ExecutionMode::Resume),
            run_health_status: run_health.status,
            run_health_reason: run_health.reason,
        },
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

fn execute_task(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    report_profile: &AgentProfile,
    task: &TaskPlan,
    attempt: u32,
    provenance: AttemptProvenance,
) -> Result<TaskAttemptResult> {
    let started = std::time::Instant::now();
    let task_dir = task_dir_name(&task.task_id)?;
    let attempt_dir = run_dir
        .join("tasks")
        .join(&task_dir)
        .join("attempts")
        .join(attempt.to_string());
    let workspace = attempt_dir.join("workspace");
    fs::create_dir_all(&workspace)?;
    prepare_workspace(&workspace, task)?;
    atomic_write_json(
        &run_dir
            .join("tasks")
            .join(&task_dir)
            .join("task.snapshot.json"),
        task,
    )?;
    if external::is_external_task(task) {
        return external::execute_external_task(external::ExternalTaskExecution {
            run_dir,
            spec,
            profile,
            report_profile,
            task,
            attempt,
            provenance,
            attempt_dir: &attempt_dir,
            started,
        });
    }
    let agent_run = run_agent(
        spec,
        profile,
        report_profile,
        task,
        attempt,
        &attempt_dir,
        &workspace,
    )?;
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
    let artifact_result = collect_artifacts(
        &workspace,
        &attempt_dir.join("artifacts"),
        &task.artifact_spec.required_paths,
    );
    let (usage, mut warnings) = collect_usage(profile, &attempt_dir);
    if artifact_result.is_err() {
        warnings.push(FailureCode::ArtifactCollectionFailed);
    }
    for warning in &warnings {
        append_event(
            &run_dir.join("events.jsonl"),
            &event(
                &spec.run_id,
                Some(&task.task_id),
                "task_warning",
                &format!("attempt {attempt} warning {warning:?}"),
            ),
            &[],
        )?;
    }
    let result = TaskAttemptResult {
        schema_version: 1,
        task_id: task.task_id.clone(),
        attempt,
        provenance,
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
        usage,
        warnings,
    };
    atomic_write_json(&attempt_dir.join("result.json"), &result)?;
    Ok(result)
}
fn prepare_workspace(workspace: &Path, task: &TaskPlan) -> Result<()> {
    if task.external_runner.is_some() {
        return Ok(());
    }
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
        no_output_timeout_sec: None,
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
#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
