use anyhow::{Result, bail};
use harnesslab_adapters::BenchmarkAdapter;
use harnesslab_core::{GlobalConfig, data_state_blocks_run, expand_path};
use std::path::{Path, PathBuf};

pub(crate) fn resolve_benchmarks_dir(
    home: &Path,
    config: Option<&GlobalConfig>,
) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("HARNESSLAB_BENCHMARKS_DIR").map(PathBuf::from)
        && path.exists()
    {
        return Some(path);
    }
    if let Some(config) = config {
        let configured = expand_path(&config.benchmarks_dir, home, home);
        if configured.exists() {
            return Some(configured);
        }
    }
    let local = PathBuf::from(".benchmarks");
    local.exists().then_some(local)
}

pub(crate) fn ensure_split_runnable(
    adapter: &dyn BenchmarkAdapter,
    benchmark_name: &str,
    split: &str,
) -> Result<()> {
    let descriptor = adapter.descriptor();
    let split_info = descriptor
        .splits
        .iter()
        .find(|candidate| candidate.name == split)
        .ok_or_else(|| anyhow::anyhow!("unknown split {split} for benchmark {benchmark_name}"))?;
    if data_state_blocks_run(split_info.data_state) {
        bail!(
            "benchmark split {benchmark_name}/{split} is not runnable: data_state={}",
            split_info.data_state
        );
    }
    Ok(())
}
