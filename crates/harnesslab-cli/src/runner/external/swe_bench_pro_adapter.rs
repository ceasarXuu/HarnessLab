use super::runtime_adapter::{
    BenchmarkRuntimeAdapter, RuntimeCleanupReport, RuntimeCleanupTarget, RuntimePreflightContext,
    preflight_report,
};
use super::{ExternalTaskExecution, swe_bench_pro};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::{Context, Result};
use harnesslab_core::{ExternalRunnerKind, TaskAttemptResult};
use std::path::{Path, PathBuf};

pub(super) static SWE_BENCH_PRO_RUNTIME_ADAPTER: SweBenchProRuntimeAdapter =
    SweBenchProRuntimeAdapter;

pub(super) struct SweBenchProRuntimeAttempt {
    pub(super) dataset_path: PathBuf,
    pub(super) source_path: PathBuf,
    pub(super) compatibility: BenchmarkRuntimeCompatibility,
}

pub(super) struct SweBenchProRuntimeAdapter;

impl BenchmarkRuntimeAdapter for SweBenchProRuntimeAdapter {
    fn adapter_id(&self) -> &'static str {
        "swe-bench-pro-runtime"
    }

    fn kind(&self) -> ExternalRunnerKind {
        ExternalRunnerKind::SweBenchPro
    }

    fn preflight(
        &self,
        ctx: RuntimePreflightContext<'_>,
    ) -> harnesslab_core::RuntimePreflightReport {
        let blocking_reason = ctx.task.external_runner.as_ref().and_then(|runner| {
            runner
                .source_path
                .is_none()
                .then_some("swe-bench-pro external runner missing source_path".to_string())
        });
        preflight_report(self, ctx, blocking_reason)
    }

    fn execute(&self, ctx: ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let runner = ctx
            .task
            .external_runner
            .as_ref()
            .context("swe-bench-pro task missing runner spec")?;
        let Some(source_path) = runner.source_path.as_ref() else {
            return swe_bench_pro::source_path_failure_result(
                &ctx,
                Path::new(&runner.dataset_path),
            );
        };
        swe_bench_pro::execute_prepared(
            &ctx,
            SweBenchProRuntimeAttempt {
                dataset_path: Path::new(&runner.dataset_path).to_path_buf(),
                source_path: Path::new(source_path).to_path_buf(),
                compatibility: BenchmarkRuntimeCompatibility::from_profile(ctx.profile),
            },
        )
    }

    fn cleanup_target_resources(
        &self,
        _target: &RuntimeCleanupTarget,
    ) -> Result<RuntimeCleanupReport, String> {
        Err("swe-bench-pro has no run-level runtime cleanup target".to_string())
    }
}
