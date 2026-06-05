use super::{ExternalTaskExecution, swe_bench_pro, terminal_bench_adapter};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::{Context, Result};
use harnesslab_core::{
    AgentProfile, BenchmarkPlan, ExternalRunnerKind, RunSpec, RuntimePreflightReport,
    TaskAttemptResult, TaskPlan,
};
use std::path::{Path, PathBuf};

pub(super) trait BenchmarkRuntimeAdapter: Sync {
    fn adapter_id(&self) -> &'static str;
    fn kind(&self) -> ExternalRunnerKind;
    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> RuntimePreflightReport;
    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult>;
    fn cleanup_targets(&self, _ctx: RuntimeCleanupContext<'_>) -> Vec<RuntimeCleanupTarget> {
        Vec::new()
    }
    fn cleanup_target_resources(
        &self,
        target: &RuntimeCleanupTarget,
    ) -> Result<RuntimeCleanupReport, String>;
}

#[derive(Clone)]
pub(super) struct RuntimePreflightContext<'a> {
    pub(super) profile: &'a AgentProfile,
    pub(super) compatibility: BenchmarkRuntimeCompatibility,
    pub(super) task: &'a TaskPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum RuntimeCleanupPhase {
    PreRun,
    PostRun,
}

#[derive(Clone, Copy)]
pub(super) struct RuntimeCleanupContext<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) plan: &'a BenchmarkPlan,
    pub(super) phase: RuntimeCleanupPhase,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct RuntimeCleanupTarget {
    pub(in crate::runner) runner_kind: ExternalRunnerKind,
    pub(in crate::runner) adapter_id: &'static str,
    pub(in crate::runner) event_name: &'static str,
    pub(in crate::runner) message_prefix: &'static str,
    pub(in crate::runner) run_dir: PathBuf,
    pub(in crate::runner) scan_run_id: String,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct RuntimeCleanupReport {
    pub(in crate::runner) removed: Vec<String>,
    pub(in crate::runner) tokens: Vec<String>,
    pub(in crate::runner) projects: Vec<String>,
    pub(in crate::runner) snapshot_projects: usize,
    pub(in crate::runner) matched_projects: usize,
}

pub(super) fn runtime_adapter_for(
    kind: ExternalRunnerKind,
) -> &'static dyn BenchmarkRuntimeAdapter {
    match kind {
        ExternalRunnerKind::TerminalBench => {
            &terminal_bench_adapter::TERMINAL_BENCH_RUNTIME_ADAPTER
        }
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
    Ok(
        runtime_adapter_for(runner.kind).preflight(RuntimePreflightContext {
            profile,
            compatibility: BenchmarkRuntimeCompatibility::from_profile(profile),
            task,
        }),
    )
}

pub(super) fn runtime_cleanup_targets(ctx: RuntimeCleanupContext<'_>) -> Vec<RuntimeCleanupTarget> {
    let mut seen = Vec::new();
    let mut targets = Vec::new();
    for task in &ctx.plan.tasks {
        let Some(runner) = &task.external_runner else {
            continue;
        };
        if seen.contains(&runner.kind) {
            continue;
        }
        seen.push(runner.kind);
        targets.extend(runtime_adapter_for(runner.kind).cleanup_targets(ctx));
    }
    targets.sort_by(|left, right| {
        left.adapter_id
            .cmp(right.adapter_id)
            .then(left.run_dir.cmp(&right.run_dir))
    });
    targets
}

pub(super) fn cleanup_runtime_target(
    target: &RuntimeCleanupTarget,
) -> Result<RuntimeCleanupReport, String> {
    runtime_adapter_for(target.runner_kind).cleanup_target_resources(target)
}

struct SweBenchProRuntimeAdapter;

static SWE_BENCH_PRO_RUNTIME_ADAPTER: SweBenchProRuntimeAdapter = SweBenchProRuntimeAdapter;

impl BenchmarkRuntimeAdapter for SweBenchProRuntimeAdapter {
    fn adapter_id(&self) -> &'static str {
        "swe-bench-pro-runtime"
    }

    fn kind(&self) -> ExternalRunnerKind {
        ExternalRunnerKind::SweBenchPro
    }

    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> RuntimePreflightReport {
        preflight_report(self, ctx, None)
    }

    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let runner = ctx
            .task
            .external_runner
            .as_ref()
            .context("swe-bench-pro task missing runner spec")?;
        let compatibility = BenchmarkRuntimeCompatibility::from_profile(ctx.profile);
        swe_bench_pro::execute(
            &ctx,
            Path::new(&runner.dataset_path),
            runner.source_path.as_deref().map(Path::new),
            &compatibility,
        )
    }

    fn cleanup_target_resources(
        &self,
        _target: &RuntimeCleanupTarget,
    ) -> Result<RuntimeCleanupReport, String> {
        Err("swe-bench-pro has no run-level runtime cleanup target".to_string())
    }
}

pub(super) fn preflight_report(
    adapter: &dyn BenchmarkRuntimeAdapter,
    ctx: RuntimePreflightContext<'_>,
    blocking_reason: Option<String>,
) -> RuntimePreflightReport {
    let host_execution_reason = ctx
        .compatibility
        .host_execution_reason(adapter.kind())
        .map(str::to_string);
    RuntimePreflightReport {
        task_id: ctx.task.task_id.clone(),
        runner_kind: adapter.kind(),
        adapter_id: adapter.adapter_id().to_string(),
        agent_bridge_mode: ctx
            .compatibility
            .agent_bridge_mode(adapter.kind())
            .to_string(),
        readiness_status: if blocking_reason.is_some() {
            "blocked".to_string()
        } else {
            "ready".to_string()
        },
        compatibility_exception: host_execution_reason
            .as_ref()
            .map(|_| "host-agent-run-as-current-only".to_string()),
        compatibility_label_keys: ctx
            .compatibility
            .consumed_label_keys(adapter.kind())
            .into_iter()
            .map(str::to_string)
            .collect(),
        host_execution_reason,
        blocking_reason,
    }
}

#[cfg(test)]
#[path = "runtime_adapter_tests.rs"]
mod tests;
