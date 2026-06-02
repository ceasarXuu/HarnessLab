use anyhow::{Result, bail};
use harnesslab_core::{
    AgentProfile, AttemptProvenance, ExternalRunnerKind, RunSpec, TaskAttemptResult, TaskPlan,
};
use std::fs;
use std::path::Path;

mod log_scan;
mod swe_bench_pro;
mod terminal_bench;
pub(super) mod terminal_bench_cleanup;
mod terminal_bench_env;
mod terminal_bench_result;
mod terminal_bench_runtime;
mod terminal_bench_timeout;

pub(super) fn is_external_task(task: &TaskPlan) -> bool {
    task.external_runner.is_some()
}

pub(super) fn validate_profile_for_plan(profile: &AgentProfile, tasks: &[TaskPlan]) -> Result<()> {
    for task in tasks {
        let Some(runner) = &task.external_runner else {
            continue;
        };
        match runner.kind {
            ExternalRunnerKind::TerminalBench => {
                terminal_bench::validate_profile(profile)?;
            }
            ExternalRunnerKind::SweBenchPro => {}
        }
    }
    Ok(())
}

pub(super) struct ExternalTaskExecution<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) profile: &'a AgentProfile,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) task: &'a TaskPlan,
    pub(super) attempt: u32,
    pub(super) provenance: AttemptProvenance,
    pub(super) attempt_dir: &'a Path,
    pub(super) started: std::time::Instant,
}

pub(super) fn execute_external_task(ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
    let Some(runner) = &ctx.task.external_runner else {
        bail!("external task missing runner spec");
    };
    match runner.kind {
        ExternalRunnerKind::TerminalBench => {
            terminal_bench::execute(&ctx, Path::new(&runner.dataset_path))
        }
        ExternalRunnerKind::SweBenchPro => swe_bench_pro::execute(
            &ctx,
            Path::new(&runner.dataset_path),
            runner.source_path.as_deref().map(Path::new),
        ),
    }
}

pub(super) fn write_external_command_snapshot(
    attempt_dir: &Path,
    profile: &AgentProfile,
    command: &str,
) -> Result<()> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::write(
        agent_dir.join("command.txt"),
        format!(
            "template={}\nrendered={}\ninput_mode=external\n",
            profile.command, command
        ),
    )?;
    Ok(())
}
#[cfg(test)]
#[path = "external_tests.rs"]
mod tests;
