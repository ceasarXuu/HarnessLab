use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn int_012_replay_text_output_succeeds() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            json["run_dir"].as_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("run:"));
}

#[test]
fn int_013_replay_falls_back_when_benchmark_snapshot_is_missing() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    fs::remove_file(run_dir.join("benchmark.snapshot.json")).unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"replay_source_run_id\""));
}

#[test]
fn int_014_resume_rejects_invalid_profile_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let profile_path = run_dir.join("agent-profile.snapshot.json");
    let mut profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    profile["schema_version"] = serde_json::json!(2);
    fs::write(&profile_path, serde_json::to_vec_pretty(&profile).unwrap()).unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "resume",
            run_dir.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported schema_version 2"));
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

fn run_success(home: &Path) -> Vec<u8> {
    Command::cargo_bin("harnesslab")
        .unwrap()
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
