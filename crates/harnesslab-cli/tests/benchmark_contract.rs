use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn bench_001_terminal_bench_info_uses_local_data_root() {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .args(["benchmark", "info", "terminal-bench", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let full = json["benchmark"]["splits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|split| split["name"] == "full")
        .unwrap();
    assert_eq!(full["task_count"], 1);
    assert_eq!(full["data_state"], "unsupported");
}

#[test]
fn bench_002_swe_bench_pro_info_uses_local_data_root() {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();
    std::fs::write(
        root.path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 731\n",
    )
    .unwrap();

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .args(["benchmark", "info", "swe-bench-pro", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let full = json["benchmark"]["splits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|split| split["name"] == "full")
        .unwrap();
    assert_eq!(full["task_count"], 731);
    assert_eq!(full["data_state"], "unsupported");
}

#[test]
fn bench_003_run_blocks_unsupported_local_full_split_before_planning() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "terminal-bench",
            "--split",
            "full",
            "--json",
        ])
        .assert()
        .code(3)
        .stderr(
            predicate::str::contains("terminal-bench/full")
                .and(predicate::str::contains("data_state=unsupported")),
        );
}

#[test]
fn bench_004_run_blocks_swe_bench_pro_full_before_planning() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();
    std::fs::write(
        root.path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 731\n",
    )
    .unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "swe-bench-pro",
            "--split",
            "full",
            "--json",
        ])
        .assert()
        .code(3)
        .stderr(
            predicate::str::contains("swe-bench-pro/full")
                .and(predicate::str::contains("data_state=unsupported")),
        );
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str) {
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
