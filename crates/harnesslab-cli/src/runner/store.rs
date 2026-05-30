use anyhow::{Result, bail};
use harnesslab_core::{AgentProfile, GlobalConfig, RunSpec, is_valid_profile_name};
use harnesslab_infra::atomic_write_json;
use std::fs;
use std::path::Path;

pub(super) fn load_config(home: &Path) -> Result<GlobalConfig> {
    Ok(toml::from_str(&fs::read_to_string(
        home.join("config.toml"),
    )?)?)
}

pub(super) fn load_profile(home: &Path, name: &str) -> Result<AgentProfile> {
    if !is_valid_profile_name(name) {
        bail!("invalid agent profile name: {name}");
    }
    Ok(toml::from_str(&fs::read_to_string(
        home.join("agents").join(format!("{name}.toml")),
    )?)?)
}

pub(super) fn write_run_inputs(
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
