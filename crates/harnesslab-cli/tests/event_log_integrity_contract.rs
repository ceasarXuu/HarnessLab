use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn int_032_resume_rejects_malformed_event_log_before_reuse() {
    let home = tempfile::tempdir().unwrap();
    let run_dir = successful_run(home.path());
    corrupt_event_log(&run_dir);

    harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "resume",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("event log integrity check failed"))
        .stderr(predicate::str::contains("events.jsonl:2"));

    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("run_resumed"));
}

#[test]
fn int_033_replay_rejects_malformed_source_event_log() {
    let home = tempfile::tempdir().unwrap();
    let run_dir = successful_run(home.path());
    corrupt_event_log(&run_dir);

    harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("event log integrity check failed"))
        .stderr(predicate::str::contains("events.jsonl:2"));
}

#[test]
fn int_034_report_open_rejects_malformed_event_log() {
    let home = tempfile::tempdir().unwrap();
    let run_dir = successful_run(home.path());
    corrupt_event_log(&run_dir);

    harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "report",
            "open",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("event log integrity check failed"))
        .stderr(predicate::str::contains("events.jsonl:2"));
}

fn successful_run(home: &Path) -> std::path::PathBuf {
    init_home(home);
    write_agent(home, "printf ok > result.txt");
    let output = harnesslab()
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
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    std::path::PathBuf::from(json["run_dir"].as_str().unwrap())
}

fn corrupt_event_log(run_dir: &Path) {
    let path = run_dir.join("events.jsonl");
    let original = fs::read_to_string(&path).unwrap();
    let first_event = original.lines().next().unwrap();
    fs::write(&path, format!("{first_event}\n{{not-json}}\n")).unwrap();
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

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
