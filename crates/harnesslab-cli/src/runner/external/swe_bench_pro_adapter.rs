use super::runtime_adapter::{
    BenchmarkRuntimeAdapter, RuntimeCleanupReport, RuntimeCleanupTarget, RuntimePreflightContext,
    preflight_report,
};
use super::swe_bench_pro::runtime_snapshot::{
    SWE_BENCH_PRO_RUNTIME_ADAPTER_VERSION, SweSetupFailurePhase,
};
use super::{ExternalTaskExecution, swe_bench_pro};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::Result;
use harnesslab_core::{ExternalRunnerKind, FailureCode, TaskAttemptResult};
use std::fs;
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
        "harnesslab.swe-bench-pro.runtime"
    }

    fn adapter_version(&self) -> &'static str {
        SWE_BENCH_PRO_RUNTIME_ADAPTER_VERSION
    }

    fn benchmark_name(&self) -> &'static str {
        "swe-bench-pro"
    }

    fn kind(&self) -> ExternalRunnerKind {
        ExternalRunnerKind::SweBenchPro
    }

    fn preflight(
        &self,
        ctx: RuntimePreflightContext<'_>,
    ) -> harnesslab_core::RuntimePreflightReport {
        let blocking_reason = match super::runtime_source_ref(ctx.task) {
            Ok(Some(_)) => None,
            Ok(None) => Some("swe-bench-pro runtime binding missing source_path".to_string()),
            Err(error) => Some(error.to_string()),
        };
        preflight_report(self, ctx, blocking_reason)
    }

    fn execute(&self, ctx: &ExternalTaskExecution<'_>) -> Result<TaskAttemptResult> {
        let dataset_ref = super::runtime_dataset_ref(ctx.task)?;
        let Some(source_ref) = super::runtime_source_ref(ctx.task)? else {
            return source_path_failure_result(ctx, Path::new(dataset_ref));
        };
        swe_bench_pro::execute_prepared(
            ctx,
            SweBenchProRuntimeAttempt {
                dataset_path: Path::new(dataset_ref).to_path_buf(),
                source_path: Path::new(source_ref).to_path_buf(),
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

fn source_path_failure_result(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
) -> Result<TaskAttemptResult> {
    let attempt_root = fs::canonicalize(ctx.attempt_dir)?;
    let workspace = attempt_root.join("workspace");
    fs::create_dir_all(&workspace)?;
    let swe_dir = attempt_root.join("swe-bench-pro");
    fs::create_dir_all(&swe_dir)?;
    swe_bench_pro::missing_evaluation(
        ctx.attempt_dir,
        "swe-bench-pro external runner missing source_path",
    )?;
    swe_bench_pro::runtime_snapshot::write_swe_setup_failure_snapshots(
        ctx,
        dataset_path,
        None,
        &swe_dir,
        &workspace,
        SweSetupFailurePhase::SourcePathValidation,
    )?;
    swe_bench_pro::setup_failure_result(
        ctx,
        "source_path_validation",
        FailureCode::ExternalRunnerSetupFailed,
        "swe-bench-pro external runner missing source_path",
    )
}
