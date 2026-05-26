use anyhow::{Context, Result};
use harnesslab_adapters::adapter_for;
use harnesslab_core::RunSpec;
use harnesslab_infra::read_json;
use std::path::Path;

pub(super) fn replay_spec_from_source(
    source: &RunSpec,
    run_id: String,
    created_at: String,
    run_dir: &Path,
) -> RunSpec {
    let mut spec = source.clone();
    spec.run_id = run_id;
    spec.created_at = created_at;
    spec.paths.run_dir = run_dir.display().to_string();
    spec.replay_source_run_id = Some(source.run_id.clone());
    spec
}

pub(super) fn replay_plan_from_source(
    source: &Path,
    spec: &RunSpec,
) -> Result<harnesslab_core::BenchmarkPlan> {
    let snapshot = source.join("benchmark.snapshot.json");
    if snapshot.exists() {
        return read_json(&snapshot);
    }
    let adapter = adapter_for(&spec.benchmark.name)
        .with_context(|| format!("unknown benchmark {}", spec.benchmark.name))?;
    adapter
        .plan(&spec.benchmark.split)
        .map_err(anyhow::Error::msg)
}
