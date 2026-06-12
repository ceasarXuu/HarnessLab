use super::{
    ExternalTaskExecution, terminal_bench, terminal_bench_cleanup,
    terminal_bench_env::terminal_bench_agent_env,
    terminal_bench_result::setup_failed_result,
    terminal_bench_runtime::{
        TerminalBenchRuntimeAttempt, append_runner_config_event, terminal_bench_docker_platform,
        terminal_bench_no_output_activity_patterns, terminal_bench_runtime_dataset,
    },
    terminal_bench_runtime_snapshot::{
        TERMINAL_BENCH_RUNTIME_ADAPTER_VERSION, TerminalBenchSnapshotDiagnostics,
        write_terminal_bench_runtime_snapshots,
    },
    terminal_bench_timeout::{
        terminal_bench_no_output_timeout_sec, terminal_bench_process_timeout_sec,
        terminal_bench_timeout_values,
    },
    write_external_command_snapshot,
};
use crate::agent_registry::MaterializedAgentProfile;
use crate::runtime_compatibility::{
    AdapterCompatibilityProfile, BenchmarkRuntimeCompatibility,
    TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL, TERMINAL_BENCH_AGENT_LABEL,
    TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL, TERMINAL_BENCH_MODEL_LABEL, push_if_some,
};
use anyhow::{Result, bail};
use harnesslab_core::{
    AgentKind, AgentProfile, RunSpec, RuntimePreflightReport, TaskAttemptResult,
};
use std::fs;
use std::path::{Path, PathBuf};

use super::runtime_adapter::{
    BenchmarkRuntimeAdapter, RuntimeCleanupContext, RuntimeCleanupPhase, RuntimeCleanupReport,
    RuntimeCleanupTarget, RuntimePreflightContext, preflight_report,
};

const ADAPTER_ID: &str = "harnesslab.terminal-bench.runtime";
const IMPORT_AGENT_CLEANUP_GRACE_SEC: u64 = 30;

pub(super) struct TerminalBenchRuntimeAdapter;

pub(super) static TERMINAL_BENCH_RUNTIME_ADAPTER: TerminalBenchRuntimeAdapter =
    TerminalBenchRuntimeAdapter;

impl BenchmarkRuntimeAdapter for TerminalBenchRuntimeAdapter {
    fn adapter_id(&self) -> &'static str {
        ADAPTER_ID
    }

    fn adapter_version(&self) -> &'static str {
        TERMINAL_BENCH_RUNTIME_ADAPTER_VERSION
    }

    fn benchmark_name(&self) -> &'static str {
        "terminal-bench"
    }

    fn compatibility_profile(&self, profile: &AgentProfile) -> AdapterCompatibilityProfile {
        let compat = BenchmarkRuntimeCompatibility::from_profile(profile);
        let host_execution_reason = if compat.terminal_bench_agent_import_path.is_some() {
            Some("terminal-bench import agent host path")
        } else {
            None
        };
        let bridge_mode = if compat.terminal_bench_agent_import_path.is_some() {
            "terminal-bench-import-path"
        } else {
            "terminal-bench-official-agent"
        };
        let mut consumed_label_keys = Vec::new();
        push_if_some(
            &mut consumed_label_keys,
            TERMINAL_BENCH_AGENT_LABEL,
            &compat.terminal_bench_agent,
        );
        push_if_some(
            &mut consumed_label_keys,
            TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL,
            &compat.terminal_bench_agent_import_path,
        );
        push_if_some(
            &mut consumed_label_keys,
            TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL,
            &compat.terminal_bench_agent_pythonpath,
        );
        push_if_some(
            &mut consumed_label_keys,
            TERMINAL_BENCH_MODEL_LABEL,
            &compat.terminal_bench_model,
        );
        AdapterCompatibilityProfile {
            host_execution_reason,
            bridge_mode,
            consumed_label_keys,
        }
    }

    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> RuntimePreflightReport {
        match validate_profile(ctx.profile, &ctx.compatibility) {
            Ok(()) => preflight_report(self, ctx, None),
            Err(error) => preflight_report(self, ctx, Some(error.to_string())),
        }
    }

    fn execute(&self, ctx: &ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let compatibility = BenchmarkRuntimeCompatibility::from_profile(ctx.profile);
        let dataset_ref = super::runtime_dataset_ref(ctx.task)?;
        let dataset_path = Path::new(dataset_ref);
        let attempt_root = fs::canonicalize(ctx.attempt_dir)?;
        let output_root = attempt_root.join("official/terminal-bench");
        let official_run_id = official_run_id(ctx.spec, ctx.task, ctx.attempt);
        let result_path = output_root.join(&official_run_id).join("results.json");
        let docker_platform = docker_platform_for_task(&ctx.task.task_id);
        let runtime_dataset_path =
            match terminal_bench_runtime_dataset(&ctx, dataset_path, &docker_platform) {
                Ok(path) => path,
                Err(error) => {
                    let reason =
                        format!("terminal-bench runtime dataset preparation failed: {error}");
                    return setup_failed_result(&ctx, &result_path, &reason);
                }
            };
        let command = append_command_snapshot(
            &ctx,
            &runtime_dataset_path,
            &output_root,
            &official_run_id,
            &docker_platform,
            &compatibility,
        )?;
        let (agent_timeout_sec, test_timeout_sec, default_process_timeout_sec) =
            terminal_bench_timeout_values(
                ctx.spec.execution.timeout_sec,
                ctx.profile.timeout_sec,
                ctx.task.verifier_spec.timeout_sec,
                task_agent_timeout(&ctx),
            );
        let process_timeout_sec = terminal_bench_process_timeout_sec(
            default_process_timeout_sec,
            std::env::var("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC")
                .ok()
                .as_deref(),
        );
        let no_output_timeout_sec = terminal_bench_no_output_timeout_sec(
            agent_timeout_sec,
            test_timeout_sec,
            process_timeout_sec,
            std::env::var("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC")
                .ok()
                .as_deref(),
        );
        append_runner_config_event(
            &ctx,
            process_timeout_sec,
            no_output_timeout_sec,
            &docker_platform,
            &result_path,
            &ctx.attempt_dir.join("agent/command.txt"),
        )?;
        let prepared = TerminalBenchRuntimeAttempt {
            source_dataset_path: dataset_path.to_path_buf(),
            runtime_dataset_path,
            output_root: output_root.clone(),
            official_run_id: official_run_id.clone(),
            result_path,
            command,
            process_timeout_sec,
            no_output_timeout_sec,
            no_output_progress_paths: vec![output_root.join(&official_run_id).join("run.log")],
            no_output_activity_patterns: terminal_bench_no_output_activity_patterns(),
        };
        write_terminal_bench_runtime_snapshots(
            &ctx,
            &prepared,
            TerminalBenchSnapshotDiagnostics::PreExecution,
        )?;
        terminal_bench::execute_prepared(&ctx, prepared)
    }

    fn cleanup_targets(&self, ctx: RuntimeCleanupContext<'_>) -> Vec<RuntimeCleanupTarget> {
        terminal_bench_cleanup_targets(ctx)
    }

    fn cleanup_target_resources(
        &self,
        target: &RuntimeCleanupTarget,
    ) -> Result<RuntimeCleanupReport, String> {
        terminal_bench_cleanup::cleanup_run_resources(&target.run_dir, &target.scan_run_id)
            .map(|result| RuntimeCleanupReport {
                removed: result.removed,
                tokens: result.tokens,
                projects: result.projects,
                snapshot_projects: result.snapshot_projects,
                matched_projects: result.matched_projects,
            })
            .map_err(|error| error.to_string())
    }
}

pub(super) fn validate_profile(
    profile: &AgentProfile,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<()> {
    let _ = terminal_bench_agent(profile, compatibility)?;
    Ok(())
}

pub(super) fn append_command_snapshot(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    output_root: &Path,
    run_id: &str,
    docker_platform: &str,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<String> {
    let agent = terminal_bench_agent(ctx.profile, compatibility)?;
    let command = terminal_bench_command(TerminalBenchCommandRequest {
        dataset_path,
        agent: &agent,
        output_root,
        run_id,
        profile: ctx.profile,
        materialized_profile: ctx.materialized_profile,
        ctx,
        docker_platform,
        compatibility,
    });
    let report_command = terminal_bench_command(TerminalBenchCommandRequest {
        dataset_path,
        agent: &agent,
        output_root,
        run_id,
        profile: ctx.report_profile,
        materialized_profile: ctx.report_materialized_profile,
        ctx,
        docker_platform,
        compatibility,
    });
    write_external_command_snapshot(
        ctx.attempt_dir,
        ctx.profile,
        ctx.report_profile,
        &report_command,
        &terminal_bench_command_redaction_refs(
            ctx,
            dataset_path,
            output_root,
            run_id,
            docker_platform,
            compatibility,
        ),
    )?;
    Ok(command)
}

fn terminal_bench_command_redaction_refs(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    output_root: &Path,
    run_id: &str,
    docker_platform: &str,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Vec<String> {
    let mut refs = vec![
        ctx.attempt_dir.display().to_string(),
        dataset_path.display().to_string(),
        output_root.display().to_string(),
        output_root.join(run_id).display().to_string(),
        docker_platform.to_string(),
        ctx.profile.command.clone(),
        ctx.report_profile.command.clone(),
    ];
    if let Some(setup_script) = &ctx.materialized_profile.setup_script {
        refs.push(setup_script.clone());
    }
    if let Some(setup_script) = &ctx.report_materialized_profile.setup_script {
        refs.push(setup_script.clone());
    }
    if let Some(path) = &compatibility.terminal_bench_agent_import_path {
        refs.push(path.clone());
    }
    if let Some(path) = &compatibility.terminal_bench_agent_pythonpath {
        refs.push(path.clone());
    }
    refs
}

pub(super) fn docker_platform_for_task(task_id: &str) -> String {
    terminal_bench_docker_platform(
        task_id,
        std::env::var("HARNESSLAB_TERMINAL_BENCH_DOCKER_PLATFORM")
            .ok()
            .as_deref(),
    )
}

pub(super) fn terminal_bench_official_agent_timeout(
    agent_timeout: u64,
    uses_import_agent: bool,
) -> u64 {
    if uses_import_agent {
        agent_timeout.saturating_add(IMPORT_AGENT_CLEANUP_GRACE_SEC)
    } else {
        agent_timeout
    }
}

struct TerminalBenchCommandRequest<'a, 'ctx> {
    dataset_path: &'a Path,
    agent: &'a TerminalBenchAgent,
    output_root: &'a Path,
    run_id: &'a str,
    profile: &'a AgentProfile,
    materialized_profile: &'a MaterializedAgentProfile,
    ctx: &'a ExternalTaskExecution<'ctx>,
    docker_platform: &'a str,
    compatibility: &'a BenchmarkRuntimeCompatibility,
}

fn terminal_bench_command(request: TerminalBenchCommandRequest<'_, '_>) -> String {
    let TerminalBenchCommandRequest {
        dataset_path,
        agent,
        output_root,
        run_id,
        profile,
        materialized_profile,
        ctx,
        docker_platform,
        compatibility,
    } = request;
    let (agent_timeout, test_timeout, _) = terminal_bench_timeout_values(
        ctx.spec.execution.timeout_sec,
        profile.timeout_sec,
        ctx.task.verifier_spec.timeout_sec,
        task_agent_timeout(ctx),
    );
    let official_agent_timeout = terminal_bench_official_agent_timeout(
        agent_timeout,
        matches!(agent, TerminalBenchAgent::ImportPath(_)),
    );
    let mut command = vec![
        terminal_bench_agent_env(profile, materialized_profile, agent_timeout, compatibility),
        "if [ -z \"${DOCKER_HOST:-}\" ] && [ -S \"$HOME/.colima/default/docker.sock\" ]; then export DOCKER_HOST=\"unix://$HOME/.colima/default/docker.sock\"; fi;".to_string(),
        format!(
            "export DOCKER_DEFAULT_PLATFORM={}; export BUILDKIT_PROGRESS=plain;",
            shell_quote(docker_platform)
        ),
        "uvx --from terminal-bench tb run".to_string(),
        format!("--dataset-path {}", shell_quote(&dataset_path.display().to_string())),
        format!("--task-id {}", shell_quote(&ctx.task.task_id)),
        "--n-attempts 1".to_string(),
        "--n-concurrent 1".to_string(),
        format!("--global-agent-timeout-sec {official_agent_timeout}"),
        format!("--global-test-timeout-sec {test_timeout}"),
        format!("--output-path {}", shell_quote(&output_root.display().to_string())),
        format!("--run-id {}", shell_quote(run_id)),
        "--no-upload-results".to_string(),
    ];
    match agent {
        TerminalBenchAgent::BuiltIn { name, model } => {
            command.push(format!("--agent {}", shell_quote(name)));
            if requires_terminal_bench_model(name)
                && let Some(model) = model
            {
                command.push(format!("--model {}", shell_quote(model)));
            }
        }
        TerminalBenchAgent::ImportPath(path) => {
            command.push(format!("--agent-import-path {}", shell_quote(path)));
        }
    }
    command.join(" ")
}

fn task_agent_timeout(ctx: &ExternalTaskExecution<'_>) -> Option<u64> {
    ctx.task
        .external_runner
        .as_ref()
        .and_then(|runner| runner.agent_timeout_sec)
}

enum TerminalBenchAgent {
    BuiltIn { name: String, model: Option<String> },
    ImportPath(String),
}

fn terminal_bench_agent(
    profile: &AgentProfile,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> Result<TerminalBenchAgent> {
    if let Some(path) = &compatibility.terminal_bench_agent_import_path {
        return Ok(TerminalBenchAgent::ImportPath(path.clone()));
    }
    let model = compatibility.terminal_bench_model.clone();
    if let Some(name) = &compatibility.terminal_bench_agent {
        if requires_terminal_bench_model(name) && model.is_none() {
            bail!(
                "agent profile {} must set label terminal_bench_model or model for terminal-bench {} agent",
                profile.name,
                name
            );
        }
        return Ok(TerminalBenchAgent::BuiltIn {
            name: name.clone(),
            model,
        });
    }
    match profile.kind {
        AgentKind::Codex | AgentKind::Opencode if model.is_none() => bail!(
            "agent profile {} must set label terminal_bench_model or model for terminal-bench {} agent",
            profile.name,
            match profile.kind {
                AgentKind::Codex => "codex",
                AgentKind::Opencode => "opencode",
                _ => unreachable!(),
            }
        ),
        AgentKind::Codex => Ok(TerminalBenchAgent::BuiltIn {
            name: "codex".to_string(),
            model,
        }),
        AgentKind::ClaudeCode => Ok(TerminalBenchAgent::BuiltIn {
            name: "claude-code".to_string(),
            model,
        }),
        AgentKind::Opencode => Ok(TerminalBenchAgent::BuiltIn {
            name: "opencode".to_string(),
            model,
        }),
        AgentKind::PiCodingAgent | AgentKind::Custom | AgentKind::Fake => bail!(
            "agent profile {} must set label terminal_bench_agent or terminal_bench_agent_import_path",
            profile.name
        ),
    }
}

fn requires_terminal_bench_model(name: &str) -> bool {
    matches!(name, "codex" | "opencode")
}

fn terminal_bench_cleanup_targets(ctx: RuntimeCleanupContext<'_>) -> Vec<RuntimeCleanupTarget> {
    let mut targets = Vec::new();
    if ctx.phase == RuntimeCleanupPhase::PreRun
        && let Some(runs_dir) = ctx.run_dir.parent()
        && let Ok(entries) = fs::read_dir(runs_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path == ctx.run_dir {
                continue;
            }
            if let Some(target) = terminal_bench_cleanup_target(&path) {
                targets.push(target);
            }
        }
    }
    targets.push(terminal_bench_runtime_cleanup_target(
        ctx.run_dir.to_path_buf(),
        ctx.spec.run_id.clone(),
    ));
    targets
}

fn terminal_bench_cleanup_target(run_dir: &Path) -> Option<RuntimeCleanupTarget> {
    let file_name = run_dir.file_name()?.to_str()?.to_string();
    let snapshot_exists = run_dir
        .join("terminal-bench-compose-projects.json")
        .is_file();
    if let Some(run_id) = terminal_bench_run_id_from_spec(run_dir) {
        return Some(terminal_bench_runtime_cleanup_target(
            run_dir.to_path_buf(),
            run_id,
        ));
    }
    if snapshot_exists || looks_like_terminal_bench_run_dir(&file_name) {
        return Some(terminal_bench_runtime_cleanup_target(
            run_dir.to_path_buf(),
            file_name,
        ));
    }
    None
}

fn terminal_bench_runtime_cleanup_target(
    run_dir: PathBuf,
    scan_run_id: String,
) -> RuntimeCleanupTarget {
    RuntimeCleanupTarget {
        adapter_id: ADAPTER_ID,
        event_name: "terminal_bench_docker_cleanup",
        message_prefix: "terminal-bench docker cleanup",
        run_dir,
        scan_run_id,
    }
}

fn terminal_bench_run_id_from_spec(run_dir: &Path) -> Option<String> {
    let bytes = fs::read(run_dir.join("run.json")).ok()?;
    let spec = serde_json::from_slice::<RunSpec>(&bytes).ok()?;
    (spec.benchmark.name == "terminal-bench").then_some(spec.run_id)
}

fn looks_like_terminal_bench_run_dir(name: &str) -> bool {
    let Some(timestamp) = name.rsplit('-').next() else {
        return false;
    };
    let timestamp_bytes = timestamp.as_bytes();
    name.contains("-terminal-bench-")
        && timestamp.len() >= 10
        && timestamp.ends_with('Z')
        && timestamp_bytes
            .get(0..8)
            .is_some_and(|date| date.iter().all(u8::is_ascii_digit))
        && timestamp_bytes.get(8).is_some_and(|ch| *ch == b'T')
        && timestamp_bytes
            .get(9..timestamp.len().saturating_sub(1))
            .is_some_and(|tail| !tail.is_empty() && tail.iter().all(u8::is_ascii_digit))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn official_run_id(spec: &RunSpec, task: &harnesslab_core::TaskPlan, attempt: u32) -> String {
    format!("{}-{}-{}", spec.run_id, task.task_id, attempt)
        .chars()
        .map(|ch| {
            ch.is_ascii_alphanumeric()
                .then(|| ch.to_ascii_lowercase())
                .unwrap_or('-')
        })
        .collect()
}
