use crate::output::{PathOutput, RunOutput};
use crate::print_json;
use anyhow::{Context, Result, bail};
use harnesslab_adapters::adapter_for;
use harnesslab_core::{
    AgentProfile, BenchmarkRef, EvaluationRecord, FailureClass, FailureCode, GlobalConfig,
    InputMode, Outcome, PatchRecord, PatchStatus, RunPaths, RunSpec, TaskAttemptResult, TaskPlan,
    TaskState, UsageRecord, classify_agent_process, classify_evaluation_process, derive_exit_code,
    is_valid_profile_name, summarize_results, validate_global_config, validate_run_spec,
};
use harnesslab_infra::{
    ExecSpec, HostProcessExecutor, append_event, atomic_write_json, collect_artifacts,
    command_exists, event, first_command_word, read_json,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use time::OffsetDateTime;

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
    let adapter = adapter_for(benchmark_name)
        .with_context(|| format!("unknown benchmark {benchmark_name}"))?;
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
    let spec: RunSpec = read_json(&source.join("run.json"))?;
    let profile: AgentProfile = read_json(&source.join("agent-profile.snapshot.json"))?;
    profile.validate()?;
    if let Some(command) = first_command_word(&profile.command)
        && !command_exists(command)
    {
        bail!("replay blocker: required agent command missing: {command}");
    }
    fs::write(
        home.join("agents").join(format!("{}.toml", profile.name)),
        toml::to_string_pretty(&profile)?,
    )?;
    execute_new_run(
        home,
        &profile.name,
        &spec.benchmark.name,
        &spec.benchmark.split,
        json,
        Some(spec.run_id),
    )
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
    let mut attempts = Vec::new();
    for task in &plan.tasks {
        attempts.push(execute_task(run_dir, profile, task, 1)?);
    }
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

fn execute_task(
    run_dir: &Path,
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
    let agent = HostProcessExecutor::exec(&ExecSpec {
        command: render_command(profile, task, &attempt_dir)?,
        stdin: matches!(profile.input_mode, InputMode::Stdin | InputMode::Tty)
            .then(|| task.instruction.clone()),
        working_dir: workspace.clone(),
        timeout_sec: agent_timeout(profile, task),
        stdout_path: attempt_dir.join("agent/stdout.log"),
        stderr_path: attempt_dir.join("agent/stderr.log"),
    })?;
    let agent_failure = classify_agent_process(&agent);
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
        agent: Some(agent),
        evaluation,
        patch,
        usage: UsageRecord::Unknown,
        warnings: vec![FailureCode::UsageUnknown],
    };
    atomic_write_json(&attempt_dir.join("result.json"), &result)?;
    Ok(result)
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

fn render_command(profile: &AgentProfile, task: &TaskPlan, attempt_dir: &Path) -> Result<String> {
    match profile.input_mode {
        InputMode::Argument => Ok(profile
            .command
            .replace("{{instruction}}", &shell_quote(&task.instruction))),
        InputMode::File => {
            let path = attempt_dir.join("instruction.txt");
            fs::write(&path, &task.instruction)?;
            Ok(profile
                .command
                .replace("{{instruction}}", &shell_quote(&path.display().to_string())))
        }
        InputMode::Stdin | InputMode::Tty => Ok(profile.command.clone()),
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

fn agent_timeout(profile: &AgentProfile, task: &TaskPlan) -> u64 {
    if task.task_id.contains("agent-timeout") {
        1
    } else {
        profile.timeout_sec
    }
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn run_shell(cwd: &Path, command: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .status()?;
    if !status.success() {
        bail!("command failed: {command}");
    }
    Ok(())
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
