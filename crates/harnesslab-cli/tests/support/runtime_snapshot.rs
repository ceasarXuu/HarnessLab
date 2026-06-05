use std::fs;
use std::path::{Path, PathBuf};

pub fn run_ids(home: &Path) -> Vec<String> {
    let mut ids = fs::read_dir(home.join("runs"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

pub fn assert_material_scope(
    array: &serde_json::Value,
    name: &str,
    scope: &str,
    public_path_is_null: bool,
) {
    let material = array
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == name)
        .unwrap_or_else(|| panic!("missing {name} in {array}"));
    assert_eq!(material["validation_scope"], scope);
    assert_eq!(material["public_path"].is_null(), public_path_is_null);
}

pub fn material_path(private_path: &Path, name: &str) -> PathBuf {
    let private: serde_json::Value =
        serde_json::from_slice(&fs::read(private_path).unwrap()).unwrap();
    private["replay_materials"]
        .as_array()
        .unwrap()
        .iter()
        .find(|material| material["name"] == name)
        .unwrap()["path"]
        .as_str()
        .unwrap()
        .into()
}

pub fn task_runtime_path(attempt_dir: &Path) -> PathBuf {
    attempt_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("task-runtime.snapshot.json")
}

pub fn rewrite_snapshot_pair_for_material(
    private_path: &Path,
    public_path: &Path,
    name: &str,
    material_path: &Path,
) {
    let checksum = stable_file_checksum(material_path);
    let size = fs::metadata(material_path).unwrap().len();
    let mut private: serde_json::Value =
        serde_json::from_slice(&fs::read(private_path).unwrap()).unwrap();
    let mut public: serde_json::Value =
        serde_json::from_slice(&fs::read(public_path).unwrap()).unwrap();
    update_material(&mut private["replay_materials"], name, &checksum, size);
    update_material(&mut public["runtime_materials"], name, &checksum, size);
    let runtime_fingerprint = runtime_fingerprint_from_private(&private);
    private["runtime_fingerprint"] = serde_json::json!(runtime_fingerprint);
    public["runtime_fingerprint"] = private["runtime_fingerprint"].clone();
    private["public_fingerprint"] = serde_json::json!(public_fingerprint_from_public(&public));
    fs::write(private_path, serde_json::to_vec_pretty(&private).unwrap()).unwrap();
    fs::write(public_path, serde_json::to_vec_pretty(&public).unwrap()).unwrap();
}

pub fn rewrite_task_runtime_anchor_for_attempt(
    task_runtime_path: &Path,
    private_path: &Path,
    public_path: &Path,
) {
    let private: serde_json::Value =
        serde_json::from_slice(&fs::read(private_path).unwrap()).unwrap();
    let mut task_runtime: serde_json::Value =
        serde_json::from_slice(&fs::read(task_runtime_path).unwrap()).unwrap();
    let attempt = private["attempt"].as_u64().unwrap();
    let anchor = task_runtime["external_runtime_attempts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["attempt"].as_u64() == Some(attempt))
        .unwrap();
    anchor["private_checksum"] = serde_json::json!(stable_file_checksum(private_path));
    anchor["public_checksum"] = serde_json::json!(stable_file_checksum(public_path));
    anchor["runtime_fingerprint"] = private["runtime_fingerprint"].clone();
    anchor["public_fingerprint"] = private["public_fingerprint"].clone();
    fs::write(
        task_runtime_path,
        serde_json::to_vec_pretty(&task_runtime).unwrap(),
    )
    .unwrap();
}

pub fn rewrite_snapshot_scope_with_authority(
    private_path: &Path,
    public_path: &Path,
    task_runtime_path: &Path,
    benchmark_path: &Path,
    name: &str,
    scope: Option<&str>,
) {
    let mut private: serde_json::Value =
        serde_json::from_slice(&fs::read(private_path).unwrap()).unwrap();
    let mut public: serde_json::Value =
        serde_json::from_slice(&fs::read(public_path).unwrap()).unwrap();
    update_material_scope(&mut private["replay_materials"], name, scope);
    update_material_scope(&mut public["runtime_materials"], name, scope);
    let runtime_fingerprint = runtime_fingerprint_from_private(&private);
    private["runtime_fingerprint"] = serde_json::json!(runtime_fingerprint);
    public["runtime_fingerprint"] = private["runtime_fingerprint"].clone();
    private["public_fingerprint"] = serde_json::json!(public_fingerprint_from_public(&public));
    fs::write(private_path, serde_json::to_vec_pretty(&private).unwrap()).unwrap();
    fs::write(public_path, serde_json::to_vec_pretty(&public).unwrap()).unwrap();
    rewrite_task_runtime_anchor_for_attempt(task_runtime_path, private_path, public_path);
    rewrite_benchmark_task_runtime_snapshot(benchmark_path, task_runtime_path);
}

pub fn stable_file_checksum(path: &Path) -> String {
    stable_checksum_bytes(&fs::read(path).unwrap())
}

pub fn assert_json_array_has_name(array: &serde_json::Value, name: &str) {
    assert!(
        array
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == name || entry["phase"] == name),
        "missing {name} in {array}"
    );
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn update_material(array: &mut serde_json::Value, name: &str, checksum: &str, size: u64) {
    let material = array
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|material| material["name"] == name)
        .unwrap();
    material["checksum"] = serde_json::json!(checksum);
    material["size_bytes"] = serde_json::json!(size);
}

fn update_material_scope(array: &mut serde_json::Value, name: &str, scope: Option<&str>) {
    let material = array
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|material| material["name"] == name)
        .unwrap()
        .as_object_mut()
        .unwrap();
    if let Some(scope) = scope {
        material.insert("validation_scope".to_string(), serde_json::json!(scope));
    } else {
        material.remove("validation_scope");
    }
}

fn rewrite_benchmark_task_runtime_snapshot(benchmark_path: &Path, task_runtime_path: &Path) {
    let task_runtime: serde_json::Value =
        serde_json::from_slice(&fs::read(task_runtime_path).unwrap()).unwrap();
    let task_id = task_runtime["task_id"].as_str().unwrap();
    let mut plan: serde_json::Value =
        serde_json::from_slice(&fs::read(benchmark_path).unwrap()).unwrap();
    let snapshot = plan["task_runtime_snapshots"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|snapshot| snapshot["task_id"] == task_id)
        .unwrap();
    *snapshot = task_runtime;
    fs::write(benchmark_path, serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
}

fn runtime_fingerprint_from_private(private: &serde_json::Value) -> String {
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
    }))
}

fn public_fingerprint_from_public(public: &serde_json::Value) -> String {
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
        "runtime_fingerprint": public["runtime_fingerprint"].clone(),
    }))
}

fn stable_json_checksum(value: &serde_json::Value) -> String {
    stable_checksum_bytes(&serde_json::to_vec(value).unwrap())
}

fn stable_checksum_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}
