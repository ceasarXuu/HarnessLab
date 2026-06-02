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

    harnesslab()
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
fn int_012_replay_uses_unredacted_runtime_profile_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_inherit_env(
        home.path(),
        "printf ok > result.txt",
        &["HARNESSLAB_REDACT_REPLAY_TEST"],
    );
    let output = run_success_with_env(home.path(), Some(("HARNESSLAB_REDACT_REPLAY_TEST", "ok")));
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    assert!(
        fs::read_to_string(run_dir.join("agent-profile.snapshot.json"))
            .unwrap()
            .contains("printf [REDACTED] > result.txt")
    );
    assert!(
        fs::read_to_string(run_dir.join("agent-profile.runtime.json"))
            .unwrap()
            .contains("printf ok > result.txt")
    );
    assert!(
        !fs::read_to_string(run_dir.join("report.html"))
            .unwrap()
            .contains("agent-profile.runtime.json")
    );

    let replay_output = harnesslab()
        .env("HARNESSLAB_REDACT_REPLAY_TEST", "ok")
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
        .stdout(predicate::str::contains("\"status\":\"success\""))
        .get_output()
        .stdout
        .clone();
    let replay_json: serde_json::Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_dir = Path::new(replay_json["run_dir"].as_str().unwrap());
    assert_eq!(replay_json["verdict"], "success");
    assert_eq!(
        replay_json["report_path"],
        replay_dir.join("report.html").display().to_string()
    );
    assert_eq!(
        replay_json["results_path"],
        replay_dir.join("results.json").display().to_string()
    );
    let replay_results: serde_json::Value =
        serde_json::from_reader(fs::File::open(replay_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(replay_results["report_path"], replay_json["report_path"]);
}

#[test]
fn int_017_replay_redacts_public_artifacts_without_current_env() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_inherit_env(
        home.path(),
        "printf ok > result.txt # sk-replay-secret",
        &["HARNESSLAB_SECRET_REPLAY_TEST"],
    );
    let output = run_success_with_env(
        home.path(),
        Some(("HARNESSLAB_SECRET_REPLAY_TEST", "sk-replay-secret")),
    );
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    let replay_output = harnesslab()
        .env_remove("HARNESSLAB_SECRET_REPLAY_TEST")
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
        .stdout(predicate::str::contains("\"status\":\"success\""))
        .get_output()
        .stdout
        .clone();
    let replay_json: serde_json::Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_dir = Path::new(replay_json["run_dir"].as_str().unwrap());
    assert_public_artifacts_do_not_contain(replay_dir, "sk-replay-secret");
    assert!(
        fs::read_to_string(replay_dir.join("events.jsonl"))
            .unwrap()
            .contains("profile_snapshot_loaded")
    );
}

#[test]
fn int_018_replay_rejects_redacted_legacy_profile_without_runtime_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_inherit_env(
        home.path(),
        "printf ok > result.txt",
        &["HARNESSLAB_REDACT_LEGACY_REPLAY_TEST"],
    );
    let output = run_success_with_env(
        home.path(),
        Some(("HARNESSLAB_REDACT_LEGACY_REPLAY_TEST", "ok")),
    );
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    fs::remove_file(run_dir.join("agent-profile.runtime.json")).unwrap();

    harnesslab()
        .env("HARNESSLAB_REDACT_LEGACY_REPLAY_TEST", "ok")
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("runtime profile snapshot missing"));
}

#[test]
fn int_019_resume_report_marks_missing_original_command() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    fs::remove_file(run_dir.join("command.txt")).unwrap();

    harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "resume",
            run_dir.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert!(
        fs::read_to_string(run_dir.join("report.html"))
            .unwrap()
            .contains("[ORIGINAL_COMMAND_UNAVAILABLE]")
    );
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
    let profile_path = run_dir.join("agent-profile.runtime.json");
    let mut profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&profile_path).unwrap()).unwrap();
    profile["schema_version"] = serde_json::json!(2);
    fs::write(&profile_path, serde_json::to_vec_pretty(&profile).unwrap()).unwrap();

    harnesslab()
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
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str) {
    write_agent_with_inherit_env(home, command, &[]);
}

fn write_agent_with_inherit_env(home: &Path, command: &str, inherit_env: &[&str]) {
    let inherit_env = inherit_env
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
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
inherit_env = [{inherit_env}]
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
    run_success_with_env(home, None)
}

fn run_success_with_env(home: &Path, env: Option<(&str, &str)>) -> Vec<u8> {
    let mut command = harnesslab();
    if let Some((key, value)) = env {
        command.env(key, value);
    }
    command
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

fn assert_public_artifacts_do_not_contain(run_dir: &Path, secret: &str) {
    let mut stack = vec![run_dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("agent-profile.runtime.json")
            {
                continue;
            }
            let bytes = fs::read(&path).unwrap();
            let content = String::from_utf8_lossy(&bytes);
            assert!(
                !content.contains(secret),
                "public artifact {} leaked secret",
                path.display()
            );
        }
    }
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
