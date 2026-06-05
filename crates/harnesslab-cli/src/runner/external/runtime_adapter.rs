use super::{ExternalTaskExecution, swe_bench_pro, terminal_bench};
use anyhow::{Context, Result};
use harnesslab_core::{
    AgentProfile, ExternalRunnerKind, RuntimePreflightReport, TaskAttemptResult, TaskPlan,
};
use std::path::Path;

pub(super) trait BenchmarkRuntimeAdapter: Sync {
    fn kind(&self) -> ExternalRunnerKind;
    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> Result<RuntimePreflightReport>;
    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult>;
}

#[derive(Clone, Copy)]
pub(super) struct RuntimePreflightContext<'a> {
    pub(super) profile: &'a AgentProfile,
    pub(super) task: &'a TaskPlan,
}

pub(super) fn runtime_adapter_for(
    kind: ExternalRunnerKind,
) -> &'static dyn BenchmarkRuntimeAdapter {
    match kind {
        ExternalRunnerKind::TerminalBench => &TERMINAL_BENCH_RUNTIME_ADAPTER,
        ExternalRunnerKind::SweBenchPro => &SWE_BENCH_PRO_RUNTIME_ADAPTER,
    }
}

pub(super) fn preflight_external_task(
    profile: &AgentProfile,
    task: &TaskPlan,
) -> Result<RuntimePreflightReport> {
    let runner = task
        .external_runner
        .as_ref()
        .context("external task missing runner spec")?;
    runtime_adapter_for(runner.kind).preflight(RuntimePreflightContext { profile, task })
}

struct TerminalBenchRuntimeAdapter;
struct SweBenchProRuntimeAdapter;

static TERMINAL_BENCH_RUNTIME_ADAPTER: TerminalBenchRuntimeAdapter = TerminalBenchRuntimeAdapter;
static SWE_BENCH_PRO_RUNTIME_ADAPTER: SweBenchProRuntimeAdapter = SweBenchProRuntimeAdapter;

impl BenchmarkRuntimeAdapter for TerminalBenchRuntimeAdapter {
    fn kind(&self) -> ExternalRunnerKind {
        ExternalRunnerKind::TerminalBench
    }

    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> Result<RuntimePreflightReport> {
        terminal_bench::validate_profile(ctx.profile)?;
        Ok(RuntimePreflightReport {
            task_id: ctx.task.task_id.clone(),
            runner_kind: self.kind(),
            host_execution_reason: ctx
                .profile
                .labels
                .contains_key("terminal_bench_agent_import_path")
                .then(|| "terminal-bench import agent host path".to_string()),
        })
    }

    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let runner = ctx
            .task
            .external_runner
            .as_ref()
            .context("terminal-bench task missing runner spec")?;
        terminal_bench::execute(&ctx, Path::new(&runner.dataset_path))
    }
}

impl BenchmarkRuntimeAdapter for SweBenchProRuntimeAdapter {
    fn kind(&self) -> ExternalRunnerKind {
        ExternalRunnerKind::SweBenchPro
    }

    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> Result<RuntimePreflightReport> {
        Ok(RuntimePreflightReport {
            task_id: ctx.task.task_id.clone(),
            runner_kind: self.kind(),
            host_execution_reason: (ctx
                .profile
                .labels
                .get("swe_bench_pro_agent")
                .map(String::as_str)
                == Some("gold"))
            .then(|| "swe-bench-pro gold host path".to_string()),
        })
    }

    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let runner = ctx
            .task
            .external_runner
            .as_ref()
            .context("swe-bench-pro task missing runner spec")?;
        swe_bench_pro::execute(
            &ctx,
            Path::new(&runner.dataset_path),
            runner.source_path.as_deref().map(Path::new),
        )
    }
}

#[cfg(test)]
#[path = "runtime_adapter_tests.rs"]
mod tests;
