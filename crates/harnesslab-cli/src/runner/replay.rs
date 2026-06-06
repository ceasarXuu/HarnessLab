use anyhow::{Result, bail};
use harnesslab_core::{
    BenchmarkPlan, ExternalRunnerKind, RunSpec, RuntimeTaskSnapshot, task_dir_name,
};
use harnesslab_infra::{read_json, stable_path_checksum};
use serde_json::Value;
use std::fs;
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

pub(super) fn replay_plan_from_source(source: &Path, spec: &RunSpec) -> Result<BenchmarkPlan> {
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

pub(super) fn validate_replay_task_runtime_snapshots(
    source: &Path,
    plan: &BenchmarkPlan,
) -> Result<()> {
    let requires_task_runtime_snapshot = plan.tasks.iter().any(|task| {
        task.external_runner.is_some()
            || plan.task_runtime_snapshots.iter().any(|snapshot| {
                snapshot.task_id == task.task_id && snapshot.external_runner.is_some()
            })
    });
    if !requires_task_runtime_snapshot {
        return Ok(());
    }
    if plan.task_runtime_snapshots.is_empty() {
        bail!(
            "replay blocker: task_runtime_snapshots missing for external benchmark {}/{}; cannot safely replay without task runtime authority",
            plan.benchmark.name,
            plan.split
        );
    }
    for task in &plan.tasks {
        let expected = task_runtime_snapshot_for(plan, &task.task_id)?;
        if task.external_runner != expected.external_runner {
            bail!(
                "replay blocker: task external_runner mismatch for task {}; cannot safely replay external benchmark task with divergent runtime authority",
                task.task_id
            );
        }
        if task.external_runner.is_none() && expected.external_runner.is_none() {
            continue;
        }
        let snapshot_path = source
            .join("tasks")
            .join(task_dir_name(&task.task_id)?)
            .join("task-runtime.snapshot.json");
        if !snapshot_path.exists() {
            bail!(
                "replay blocker: task-runtime.snapshot.json missing for task {}; cannot safely replay external benchmark task without task runtime authority",
                task.task_id
            );
        }
        let actual: RuntimeTaskSnapshot = read_json(&snapshot_path)?;
        if &actual != expected {
            bail!(
                "replay blocker: task-runtime.snapshot.json mismatch for task {}; cannot safely replay external benchmark task with divergent runtime authority",
                task.task_id
            );
        }
    }
    validate_external_runtime_snapshots(source, plan)?;
    Ok(())
}

fn validate_external_runtime_snapshots(source: &Path, plan: &BenchmarkPlan) -> Result<()> {
    for task in plan
        .tasks
        .iter()
        .filter(|task| task.external_runner.is_some())
    {
        let attempts_dir = source
            .join("tasks")
            .join(task_dir_name(&task.task_id)?)
            .join("attempts");
        let mut attempts = Vec::new();
        for entry in fs::read_dir(&attempts_dir).map_err(|error| {
            anyhow::anyhow!(
                "replay blocker: external-runtime snapshots missing for task {}; cannot safely replay external benchmark task without attempt runtime materials: {error}",
                task.task_id
            )
        })? {
            let entry = entry.map_err(|error| {
                anyhow::anyhow!(
                    "replay blocker: external-runtime attempt entry is unreadable for task {}; cannot safely replay external benchmark task without complete attempt runtime materials: {error}",
                    task.task_id
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                anyhow::anyhow!(
                    "replay blocker: external-runtime attempt entry type is unreadable for task {}; cannot safely replay external benchmark task without complete attempt runtime materials: {error}",
                    task.task_id
                )
            })?;
            if file_type.is_dir() {
                attempts.push(entry.path());
            }
        }
        if attempts.is_empty() {
            bail!(
                "replay blocker: external-runtime snapshots missing for task {}; cannot safely replay external benchmark task without attempt runtime materials",
                task.task_id
            );
        }
        for attempt_dir in attempts {
            let task_runtime = task_runtime_snapshot_artifact(source, &task.task_id)?;
            validate_external_runtime_snapshot_pair(&attempt_dir, plan, task, &task_runtime)?;
        }
    }
    Ok(())
}

fn validate_external_runtime_snapshot_pair(
    attempt_dir: &Path,
    plan: &BenchmarkPlan,
    task: &harnesslab_core::TaskPlan,
    task_runtime: &RuntimeTaskSnapshot,
) -> Result<()> {
    let private_path = attempt_dir.join("external-runtime.private.json");
    if !private_path.exists() {
        bail!(
            "replay blocker: external-runtime.private.json missing for task {}; cannot safely replay external benchmark task without private runtime materials",
            task.task_id
        );
    }
    let public_path = attempt_dir.join("external-runtime.public.json");
    if !public_path.exists() {
        bail!(
            "replay blocker: external-runtime.public.json missing for task {}; cannot safely replay external benchmark task without public runtime materials",
            task.task_id
        );
    }
    let private: Value = read_json(&private_path).map_err(|error| {
        anyhow::anyhow!(
            "replay blocker: external-runtime.private.json invalid for task {}; cannot safely replay external benchmark task without readable private runtime materials: {error}",
            task.task_id
        )
    })?;
    let public: Value = read_json(&public_path).map_err(|error| {
        anyhow::anyhow!(
            "replay blocker: external-runtime.public.json invalid for task {}; cannot safely replay external benchmark task without readable public runtime materials: {error}",
            task.task_id
        )
    })?;
    let Some(runner) = task.external_runner.as_ref() else {
        return Ok(());
    };
    let attempt = attempt_number(attempt_dir)?;
    let runner_kind = serde_json::to_value(runner.kind)?;
    if private["schema_version"] != 1
        || public["schema_version"] != 1
        || private["visibility"] != "private"
        || public["visibility"] != "public"
        || private["benchmark"] != plan.benchmark.name
        || public["benchmark"] != plan.benchmark.name
        || private["task_id"] != task.task_id
        || public["task_id"] != task.task_id
        || private["attempt"] != attempt
        || public["attempt"] != attempt
        || private["runner_kind"] != runner_kind
        || public["runner_kind"] != runner_kind
        || private["adapter_version"] != public["adapter_version"]
        || private["runtime_policy"] != public["runtime_policy"]
        || private["public_artifacts"] != public["public_artifacts"]
        || private["dataset_path"] != runner.dataset_path
        || private["source_path"] != serde_json::to_value(&runner.source_path)?
        || public.get("dataset_path").is_some()
        || public.get("source_path").is_some()
        || public.get("redaction_basis").is_some()
    {
        bail!(
            "replay blocker: external-runtime snapshot mismatch for task {}; cannot safely replay external benchmark task with divergent attempt materials",
            task.task_id
        );
    }
    let expected_adapter_version = current_adapter_version(runner.kind);
    if private["adapter_version"].as_str() != Some(expected_adapter_version) {
        bail!(
            "replay blocker: external-runtime adapter version drift for task {}; stored={} current={expected_adapter_version}; cannot safely replay external benchmark task with changed runtime adapter semantics",
            task.task_id,
            private["adapter_version"].as_str().unwrap_or("unknown")
        );
    }
    let runtime_fingerprint = runtime_fingerprint_from_private(&private)?;
    let public_fingerprint = public_fingerprint_from_public(&public)?;
    if private["runtime_fingerprint"].as_str() != Some(runtime_fingerprint.as_str())
        || public["runtime_fingerprint"].as_str() != Some(runtime_fingerprint.as_str())
        || private["public_fingerprint"].as_str() != Some(public_fingerprint.as_str())
    {
        bail!(
            "replay blocker: external-runtime snapshot mismatch for task {}; cannot safely replay external benchmark task with divergent attempt materials",
            task.task_id
        );
    }
    validate_attempt_anchor(
        attempt,
        &private_path,
        &public_path,
        &private,
        task,
        task_runtime,
    )?;
    validate_live_materials(&private, task)?;
    Ok(())
}

fn current_adapter_version(kind: ExternalRunnerKind) -> &'static str {
    match kind {
        ExternalRunnerKind::TerminalBench => "terminal-bench-runtime.v1",
        ExternalRunnerKind::SweBenchPro => "swe-bench-pro-runtime.v1",
    }
}

fn validate_live_materials(private: &Value, task: &harnesslab_core::TaskPlan) -> Result<()> {
    let Some(materials) = private["replay_materials"].as_array() else {
        bail!(
            "replay blocker: external-runtime snapshot mismatch for task {}; cannot safely replay external benchmark task with missing replay materials",
            task.task_id
        );
    };
    for material in materials {
        let name = material["name"].as_str().unwrap_or("unknown");
        match material["validation_scope"].as_str() {
            Some("live_external") => validate_live_material(material, name, task)?,
            Some("archived_attempt") => {}
            _ => {
                bail!(
                    "replay blocker: external-runtime material validation scope missing for task {}; material={name}",
                    task.task_id
                );
            }
        }
    }
    Ok(())
}

fn validate_live_material(
    material: &Value,
    name: &str,
    task: &harnesslab_core::TaskPlan,
) -> Result<()> {
    let Some(path) = material["path"].as_str() else {
        bail!(
            "replay blocker: external-runtime live material path missing for task {}; material={name}",
            task.task_id
        );
    };
    let path = Path::new(path);
    let kind = material["kind"].as_str().unwrap_or("file");
    let exists = match kind {
        "file" => path.is_file(),
        "directory" => path.is_dir(),
        "missing" => false,
        _ => {
            bail!(
                "replay blocker: external-runtime live material kind unknown for task {}; material={name}; kind={kind}",
                task.task_id
            );
        }
    };
    if !exists {
        bail!(
            "replay blocker: external-runtime live material missing for task {}; material={name}; path={}",
            task.task_id,
            path.display()
        );
    }
    let actual = stable_path_checksum(path);
    let expected = material["checksum"].as_str().unwrap_or_default();
    if actual != expected {
        bail!(
            "replay blocker: external-runtime live material drift for task {}; material={name}; expected={expected}; actual={actual}",
            task.task_id
        );
    }
    Ok(())
}

fn validate_attempt_anchor(
    attempt: u64,
    private_path: &Path,
    public_path: &Path,
    private: &Value,
    task: &harnesslab_core::TaskPlan,
    task_runtime: &RuntimeTaskSnapshot,
) -> Result<()> {
    let Some(anchor) = task_runtime
        .external_runtime_attempts
        .iter()
        .find(|anchor| u64::from(anchor.attempt) == attempt)
    else {
        bail!(
            "replay blocker: external-runtime attempt anchor missing for task {}; attempt={attempt}",
            task.task_id
        );
    };
    let Some(task_dir) = private_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    else {
        bail!(
            "replay blocker: external-runtime attempt anchor path invalid for task {}; attempt={attempt}",
            task.task_id
        );
    };
    if attempt_relative_path(private_path, task_dir) != anchor.private_path
        || attempt_relative_path(public_path, task_dir) != anchor.public_path
        || stable_file_checksum(private_path) != anchor.private_checksum
        || stable_file_checksum(public_path) != anchor.public_checksum
        || private["runtime_fingerprint"].as_str() != Some(anchor.runtime_fingerprint.as_str())
        || private["public_fingerprint"].as_str() != Some(anchor.public_fingerprint.as_str())
    {
        bail!(
            "replay blocker: external-runtime attempt anchor mismatch for task {}; attempt={attempt}",
            task.task_id
        );
    }
    Ok(())
}

fn attempt_number(attempt_dir: &Path) -> Result<u64> {
    attempt_dir
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.parse::<u64>().ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "replay blocker: external-runtime attempt directory is invalid: {}",
                attempt_dir.display()
            )
        })
}

fn runtime_fingerprint_from_private(private: &Value) -> Result<String> {
    stable_json_checksum(&serde_json::json!({
        "schema_version": private["schema_version"].clone(),
        "benchmark": private["benchmark"].clone(),
        "task_id": private["task_id"].clone(),
        "attempt": private["attempt"].clone(),
        "runner_kind": private["runner_kind"].clone(),
        "adapter_version": private["adapter_version"].clone(),
        "runtime_policy": private["runtime_policy"].clone(),
        "dataset_path": private["dataset_path"].clone(),
        "source_path": private["source_path"].clone(),
        "commands": private["commands"].clone(),
        "replay_materials": private["replay_materials"].clone(),
        "public_artifacts": private["public_artifacts"].clone(),
        "runtime_diagnostics": private["runtime_diagnostics"].clone(),
    }))
}

fn public_fingerprint_from_public(public: &Value) -> Result<String> {
    stable_json_checksum(&serde_json::json!({
        "schema_version": public["schema_version"].clone(),
        "visibility": public["visibility"].clone(),
        "benchmark": public["benchmark"].clone(),
        "task_id": public["task_id"].clone(),
        "attempt": public["attempt"].clone(),
        "runner_kind": public["runner_kind"].clone(),
        "adapter_version": public["adapter_version"].clone(),
        "runtime_policy": public["runtime_policy"].clone(),
        "commands": public["commands"].clone(),
        "runtime_materials": public["runtime_materials"].clone(),
        "public_artifacts": public["public_artifacts"].clone(),
        "runtime_diagnostics": public["runtime_diagnostics"].clone(),
        "runtime_fingerprint": public["runtime_fingerprint"].clone(),
    }))
}

fn stable_json_checksum(value: &Value) -> Result<String> {
    Ok(stable_checksum_bytes(&serde_json::to_vec(value)?))
}

fn stable_checksum_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}

fn stable_file_checksum(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) => stable_checksum_bytes(&bytes),
        Err(_) => stable_checksum_bytes(format!("missing:{}", path.display()).as_bytes()),
    }
}

fn task_runtime_snapshot_artifact(source: &Path, task_id: &str) -> Result<RuntimeTaskSnapshot> {
    read_json(
        &source
            .join("tasks")
            .join(task_dir_name(task_id)?)
            .join("task-runtime.snapshot.json"),
    )
}

fn attempt_relative_path(path: &Path, task_dir: &Path) -> String {
    path.strip_prefix(task_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn task_runtime_snapshot_for<'a>(
    plan: &'a BenchmarkPlan,
    task_id: &str,
) -> Result<&'a RuntimeTaskSnapshot> {
    plan.task_runtime_snapshots
        .iter()
        .find(|snapshot| snapshot.task_id == task_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "replay blocker: task runtime snapshot missing for task {task_id}; cannot safely replay external benchmark task without task runtime authority"
            )
        })
}
