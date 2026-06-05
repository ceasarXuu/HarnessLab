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

fn stable_task_plan_hash(task_plan: &TaskPlan) -> String {
    let bytes = serde_json::to_vec(task_plan).unwrap();
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
}
