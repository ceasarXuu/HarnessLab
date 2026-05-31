use anyhow::{Result, bail};
use harnesslab_adapters::BenchmarkAdapter;
use harnesslab_core::{GlobalConfig, data_state_blocks_run};
use std::path::{Path, PathBuf};

pub(crate) fn resolve_benchmarks_dir(
    home: &Path,
    config: Option<&GlobalConfig>,
) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("HARNESSLAB_BENCHMARKS_DIR").map(PathBuf::from) {
        return Some(path);
    }
    if let Some(config) = config {
        return Some(benchmarks_dir(home, config));
    }
    let local = PathBuf::from(".benchmarks");
    local.exists().then_some(local)
}

fn benchmarks_dir(home: &Path, config: &GlobalConfig) -> PathBuf {
    if config.benchmarks_dir == "~/.harnesslab/benchmarks" {
        return home.join("benchmarks");
    }
    expand_config_path(&config.benchmarks_dir, home)
}

fn expand_config_path(value: &str, home: &Path) -> PathBuf {
    if value == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.to_path_buf());
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return std::env::var_os("HOME")
            .map(|host_home| PathBuf::from(host_home).join(rest))
            .unwrap_or_else(|| home.join(rest));
    }
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        home.join(path)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_benchmarks_dir_uses_harnesslab_home_not_nested_home() {
        let tmp = tempfile::tempdir().unwrap();
        let config = GlobalConfig::default();

        assert_eq!(
            benchmarks_dir(tmp.path(), &config),
            tmp.path().join("benchmarks")
        );
    }

    #[test]
    fn configured_missing_benchmarks_dir_is_authoritative() {
        let tmp = tempfile::tempdir().unwrap();
        let config = GlobalConfig {
            benchmarks_dir: "missing-cache".to_string(),
            ..GlobalConfig::default()
        };

        assert_eq!(
            resolve_benchmarks_dir(tmp.path(), Some(&config)).unwrap(),
            tmp.path().join("missing-cache")
        );
    }

    #[test]
    fn custom_benchmarks_dir_expands_home_and_absolute_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let host_home = std::env::var_os("HOME").map(PathBuf::from);
        let config_home = GlobalConfig {
            benchmarks_dir: "~".to_string(),
            ..GlobalConfig::default()
        };
        assert_eq!(
            benchmarks_dir(tmp.path(), &config_home),
            host_home
                .clone()
                .unwrap_or_else(|| tmp.path().to_path_buf())
        );

        let config_child = GlobalConfig {
            benchmarks_dir: "~/cache".to_string(),
            ..GlobalConfig::default()
        };
        assert_eq!(
            benchmarks_dir(tmp.path(), &config_child),
            host_home
                .clone()
                .map_or_else(|| tmp.path().join("cache"), |home| home.join("cache"))
        );

        let absolute = tmp.path().join("absolute-cache");
        let config_absolute = GlobalConfig {
            benchmarks_dir: absolute.display().to_string(),
            ..GlobalConfig::default()
        };
        assert_eq!(benchmarks_dir(tmp.path(), &config_absolute), absolute);
    }
}
