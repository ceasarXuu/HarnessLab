use super::{ExternalTaskExecution, swe_bench_pro_adapter, terminal_bench_adapter};
use crate::runtime_compatibility::{AdapterCompatibilityProfile, BenchmarkRuntimeCompatibility};
use anyhow::{Context, Result};
use harnesslab_adapters::built_in_protocol_registry;
use harnesslab_core::{
    AgentProfile, BenchmarkPlan, RunSpec, RuntimePreflightReport, TaskAttemptResult, TaskPlan,
};
use std::path::{Path, PathBuf};

pub(crate) trait BenchmarkRuntimeAdapter: Sync {
    fn adapter_id(&self) -> &'static str;
    fn adapter_version(&self) -> &'static str;
    fn benchmark_name(&self) -> &'static str;
    fn preflight(&self, ctx: RuntimePreflightContext<'_>) -> RuntimePreflightReport;
    fn execute(&self, ctx: &ExternalTaskExecution<'_>) -> Result<TaskAttemptResult>;
    fn cleanup_targets(&self, _ctx: RuntimeCleanupContext<'_>) -> Vec<RuntimeCleanupTarget> {
        Vec::new()
    }
    fn cleanup_target_resources(
        &self,
        target: &RuntimeCleanupTarget,
    ) -> Result<RuntimeCleanupReport, String>;
    /// Adapter-local compatibility profile. Generic layers must not branch on
    /// benchmark id; they consume this profile opaquely.
    fn compatibility_profile(&self, profile: &AgentProfile) -> AdapterCompatibilityProfile;
}

#[derive(Clone)]
pub(crate) struct RuntimePreflightContext<'a> {
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
pub(crate) struct RuntimeCleanupContext<'a> {
    pub(super) run_dir: &'a Path,
    pub(super) spec: &'a RunSpec,
    pub(super) plan: &'a BenchmarkPlan,
    pub(super) phase: RuntimeCleanupPhase,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct RuntimeCleanupTarget {
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

pub(super) fn runtime_adapter_for_adapter_id(
    adapter_id: &str,
) -> Result<&'static dyn BenchmarkRuntimeAdapter> {
    runtime_adapters()
        .iter()
        .copied()
        .find(|adapter| adapter.adapter_id() == adapter_id)
        .with_context(|| format!("unknown runtime adapter_id {adapter_id}"))
}

pub(in crate::runner) fn runtime_adapters() -> Vec<&'static dyn BenchmarkRuntimeAdapter> {
    // NOTE: deterministic-sample is intentionally excluded — it is a scaffold
    // golden-path adapter with no runtime execution capability. Only adapters
    // capable of actually running benchmarks are listed here.
    vec![
        &terminal_bench_adapter::TERMINAL_BENCH_RUNTIME_ADAPTER,
        &swe_bench_pro_adapter::SWE_BENCH_PRO_RUNTIME_ADAPTER,
    ]
}

pub(super) fn runtime_adapter_for_task(
    task: &TaskPlan,
) -> Result<&'static dyn BenchmarkRuntimeAdapter> {
    let binding = task
        .runtime_binding
        .as_ref()
        .context("external task missing runtime binding")?;
    built_in_protocol_registry()
        .validate_authority(&binding.authority)
        .map_err(|error| anyhow::anyhow!("invalid protocol runtime binding: {error}"))?;
    runtime_adapter_for_adapter_id(binding.authority.adapter_id.as_str())
}

pub(super) fn preflight_external_task(
    profile: &AgentProfile,
    task: &TaskPlan,
) -> Result<RuntimePreflightReport> {
    Ok(
        runtime_adapter_for_task(task)?.preflight(RuntimePreflightContext {
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
        match runtime_adapter_for_task(task) {
            Ok(adapter) => {
                if seen.contains(&adapter.adapter_id()) {
                    continue;
                }
                seen.push(adapter.adapter_id());
                targets.extend(adapter.cleanup_targets(ctx));
            }
            Err(error) => {
                eprintln!(
                    "WARNING: skipping cleanup targets for task {} due to adapter resolution failure: {error}",
                    task.task_id
                );
            }
        }
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
    runtime_adapter_for_adapter_id(target.adapter_id)
        .map_err(|error| error.to_string())?
        .cleanup_target_resources(target)
}

pub(super) fn preflight_report(
    adapter: &dyn BenchmarkRuntimeAdapter,
    ctx: RuntimePreflightContext<'_>,
    blocking_reason: Option<String>,
) -> RuntimePreflightReport {
    let compat = adapter.compatibility_profile(ctx.profile);
    let host_execution_reason = compat.host_execution_reason.map(str::to_string);
    RuntimePreflightReport {
        task_id: ctx.task.task_id.clone(),
        adapter_id: adapter.adapter_id().to_string(),
        protocol_adapter_id: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.adapter_id.to_string()),
        protocol_version: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.protocol_version.to_string()),
        protocol_benchmark_id: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.benchmark_id.to_string()),
        protocol_selected_mode: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.selected_mode.to_string()),
        protocol_stability: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| format!("{:?}", binding.authority.stability)),
        protocol_capabilities: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| {
                binding
                    .authority
                    .capabilities
                    .iter()
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        legacy_shim_used: false,
        agent_bridge_mode: compat.bridge_mode.to_string(),
        readiness_status: if blocking_reason.is_some() {
            "blocked".to_string()
        } else {
            "ready".to_string()
        },
        compatibility_exception: host_execution_reason
            .as_ref()
            .map(|_| "host-agent-run-as-current-only".to_string()),
        compatibility_label_keys: compat
            .consumed_label_keys
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
