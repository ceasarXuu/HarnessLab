use anyhow::{Result, bail};
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
    bail!(
        "replay blocker: benchmark.snapshot.json missing for {}/{}; cannot safely replay without authoritative benchmark snapshot",
        spec.benchmark.name,
        spec.benchmark.split
    )
}
