use assert_cmd::Command;
use harnesslab_core::{BenchmarkPlan, RuntimeTaskSnapshot, TaskPlan, task_dir_name};
use std::fs;
use std::path::Path;

#[test]
fn replay_007_run_writes_task_runtime_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    let plan: BenchmarkPlan =
        serde_json::from_reader(fs::File::open(run_dir.join("benchmark.snapshot.json")).unwrap())
            .unwrap();
    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(plan.task_runtime_snapshots.len(), 1);

    let runtime_snapshot = &plan.task_runtime_snapshots[0];
    assert_eq!(runtime_snapshot.benchmark.name, "fake-terminal");
    assert_eq!(runtime_snapshot.benchmark.version, "fixture");
    assert_eq!(runtime_snapshot.split, "success");
    assert_eq!(runtime_snapshot.task_id, plan.tasks[0].task_id);
    assert_eq!(runtime_snapshot.source_ref.benchmark, "fake-terminal");
    assert_eq!(
        runtime_snapshot.source_ref.upstream_id,
        plan.tasks[0].task_id
    );
    assert_eq!(
        runtime_snapshot.upstream_metadata_hash,
        runtime_snapshot.source_ref.checksum
    );
    assert!(!runtime_snapshot.instruction_hash.is_empty());
    assert!(!runtime_snapshot.task_plan_hash.is_empty());
    assert!(runtime_snapshot.external_runner.is_none());

    let task_dir = run_dir
        .join("tasks")
        .join(task_dir_name(&runtime_snapshot.task_id).unwrap());
    let task_snapshot: TaskPlan =
        serde_json::from_reader(fs::File::open(task_dir.join("task.snapshot.json")).unwrap())
            .unwrap();
    assert_eq!(
        stable_task_plan_hash(&task_snapshot),
        runtime_snapshot.task_plan_hash
    );
    let task_runtime_snapshot: RuntimeTaskSnapshot = serde_json::from_reader(
        fs::File::open(task_dir.join("task-runtime.snapshot.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(task_runtime_snapshot, *runtime_snapshot);
}

#[test]
fn replay_008_replay_blocks_external_task_runtime_snapshot_gaps() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    set_task_external_runner_only(run_dir);
    assert_replay_blocker(home.path(), run_dir, "task external_runner mismatch");

    let task_runtime_path = externalize_source_plan(run_dir, false);
    assert_replay_blocker(home.path(), run_dir, "task-runtime.snapshot.json mismatch");

    write_task_runtime_snapshot_from_plan(run_dir, &task_runtime_path);
    fs::remove_file(&task_runtime_path).unwrap();
    assert_replay_blocker(home.path(), run_dir, "task-runtime.snapshot.json missing");

    write_task_runtime_snapshot_from_plan(run_dir, &task_runtime_path);
    let mut plan: serde_json::Value =
        serde_json::from_reader(fs::File::open(run_dir.join("benchmark.snapshot.json")).unwrap())
            .unwrap();
    plan["task_runtime_snapshots"] = serde_json::json!([]);
    fs::write(
        run_dir.join("benchmark.snapshot.json"),
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    assert_replay_blocker(home.path(), run_dir, "task_runtime_snapshots missing");
    assert_eq!(fs::read_dir(home.path().join("runs")).unwrap().count(), 1);
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str) {
    let command = command.replace('\\', "\\\\").replace('"', "\\\"");
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{command}"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 5

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[usage]
parser = "none"
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn run_success(home: &Path) -> Vec<u8> {
    harnesslab()
        .args([
            "--home",
            home.to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-terminal",
            "--split",
            "success",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}

fn assert_replay_blocker(home: &Path, run_dir: &Path, message: &str) {
    let run_count = fs::read_dir(home.join("runs")).unwrap().count();
    harnesslab()
        .args([
            "--home",
            home.to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(message));
    assert_eq!(fs::read_dir(home.join("runs")).unwrap().count(), run_count);
}

fn set_task_external_runner_only(run_dir: &Path) {
    let benchmark_snapshot = run_dir.join("benchmark.snapshot.json");
    let mut plan: serde_json::Value =
        serde_json::from_reader(fs::File::open(&benchmark_snapshot).unwrap()).unwrap();
    plan["tasks"][0]["external_runner"] = replay_blocker_runner();
    fs::write(
        &benchmark_snapshot,
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
}

fn externalize_source_plan(run_dir: &Path, write_runtime_artifact: bool) -> std::path::PathBuf {
    let benchmark_snapshot = run_dir.join("benchmark.snapshot.json");
    let mut plan: serde_json::Value =
        serde_json::from_reader(fs::File::open(&benchmark_snapshot).unwrap()).unwrap();
    let runner = replay_blocker_runner();
    plan["tasks"][0]["external_runner"] = runner.clone();
    plan["task_runtime_snapshots"][0]["external_runner"] = runner;
    fs::write(
        &benchmark_snapshot,
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    let task_id = plan["tasks"][0]["task_id"].as_str().unwrap();
    let task_runtime_path = run_dir
        .join("tasks")
        .join(task_dir_name(task_id).unwrap())
        .join("task-runtime.snapshot.json");
    if write_runtime_artifact {
        write_task_runtime_snapshot_from_plan(run_dir, &task_runtime_path);
    }
    task_runtime_path
}

fn write_task_runtime_snapshot_from_plan(run_dir: &Path, task_runtime_path: &Path) {
    let plan: serde_json::Value =
        serde_json::from_reader(fs::File::open(run_dir.join("benchmark.snapshot.json")).unwrap())
            .unwrap();
    fs::write(
        task_runtime_path,
        serde_json::to_vec_pretty(&plan["task_runtime_snapshots"][0]).unwrap(),
    )
    .unwrap();
}

fn replay_blocker_runner() -> serde_json::Value {
    serde_json::json!({
        "kind": "terminal_bench",
        "dataset_path": "/tmp/harnesslab-replay-blocker"
    })
}

fn stable_task_plan_hash(task_plan: &TaskPlan) -> String {
    let bytes = serde_json::to_vec(task_plan).unwrap();
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}
