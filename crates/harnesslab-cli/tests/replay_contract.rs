mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use support::assert_public_artifacts_do_not_contain;

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
fn int_013_replay_blocks_when_benchmark_snapshot_is_missing() {
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
        .failure()
        .stderr(predicate::str::contains("benchmark.snapshot.json missing"));
    assert_eq!(fs::read_dir(home.path().join("runs")).unwrap().count(), 1);
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

#[test]
fn agt_reg_010_run_stores_agent_version_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_version_command(home.path(), "printf ok > result.txt", "printf v1");

    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let snapshot: serde_json::Value = serde_json::from_reader(
        fs::File::open(run_dir.join("agent-version.snapshot.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(snapshot["field"], "version_command");
    assert_eq!(snapshot["status"], "ok");
    assert_eq!(snapshot["stdout_tail"], "v1");
}

#[test]
fn agt_reg_010_run_redacts_version_probe_public_artifacts() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_version_command(home.path(), "printf ok > result.txt", "printf sk-secret");

    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let snapshot_text = fs::read_to_string(run_dir.join("agent-version.snapshot.json")).unwrap();
    let stdout_log = fs::read_to_string(run_dir.join("agent-version-probe/stdout.log")).unwrap();

    assert!(!snapshot_text.contains("sk-secret"));
    assert!(!stdout_log.contains("sk-secret"));
    assert!(snapshot_text.contains("[REDACTED]"));
    assert!(stdout_log.contains("[REDACTED]"));
    assert_public_artifacts_do_not_contain(run_dir, "sk-secret");
}

#[test]
fn agt_reg_010_replay_emits_version_mismatch_event() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_version_command(
        home.path(),
        "printf ok > result.txt",
        "printf $HARNESSLAB_VERSION_PROBE_TEST",
    );
    let output = run_success_with_env(home.path(), Some(("HARNESSLAB_VERSION_PROBE_TEST", "v1")));
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    let replay_output = harnesslab()
        .env("HARNESSLAB_VERSION_PROBE_TEST", "v2")
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
        .get_output()
        .stdout
        .clone();
    let replay_json: serde_json::Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_dir = Path::new(replay_json["run_dir"].as_str().unwrap());
    let events = fs::read_to_string(replay_dir.join("events.jsonl")).unwrap();

    assert!(events.contains("agent_version_mismatch"));
    assert!(events.contains("current version_command probe differs"));
    assert!(events.contains("source=status=Ok"));
    assert!(events.contains("current=status=Ok"));
    assert!(
        events.find("agent_version_mismatch").unwrap() < events.find("run_started").unwrap(),
        "version mismatch should be emitted before replay task execution"
    );
}

#[test]
fn agt_reg_010_replay_emits_version_compare_skip_when_source_snapshot_missing() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_version_command(home.path(), "printf ok > result.txt", "printf v1");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    fs::remove_file(run_dir.join("agent-version.snapshot.json")).unwrap();

    let replay_output = harnesslab()
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
        .get_output()
        .stdout
        .clone();
    let replay_json: serde_json::Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_dir = Path::new(replay_json["run_dir"].as_str().unwrap());
    let events = fs::read_to_string(replay_dir.join("events.jsonl")).unwrap();

    assert!(events.contains("agent_version_compare_skipped"));
    assert!(events.contains("source run has no agent-version.snapshot.json"));
}

#[test]
fn agt_reg_010_replay_emits_version_compare_skip_when_current_probe_missing() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_version_command(home.path(), "printf ok > result.txt", "printf v1");
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let runtime_profile_path = run_dir.join("agent-profile.runtime.json");
    let mut runtime_profile: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&runtime_profile_path).unwrap()).unwrap();
    runtime_profile
        .as_object_mut()
        .unwrap()
        .remove("version_command");
    fs::write(
        &runtime_profile_path,
        serde_json::to_vec_pretty(&runtime_profile).unwrap(),
    )
    .unwrap();

    let replay_output = harnesslab()
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
        .get_output()
        .stdout
        .clone();
    let replay_json: serde_json::Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_dir = Path::new(replay_json["run_dir"].as_str().unwrap());
    let events = fs::read_to_string(replay_dir.join("events.jsonl")).unwrap();

    assert!(events.contains("agent_version_compare_skipped"));
    assert!(events.contains("current profile has no version_command probe"));
    assert!(
        events.find("agent_version_compare_skipped").unwrap() < events.find("run_started").unwrap(),
        "version compare skip should be emitted before replay task execution"
    );
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

fn write_agent_with_version_command(home: &Path, command: &str, version_command: &str) {
    write_agent_content(
        home,
        command,
        &[],
        &format!("version_command = \"{version_command}\"\n"),
    );
}

fn write_agent_with_inherit_env(home: &Path, command: &str, inherit_env: &[&str]) {
    write_agent_content(home, command, inherit_env, "");
}

fn write_agent_content(home: &Path, command: &str, inherit_env: &[&str], extra_fields: &str) {
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
{extra_fields}

[auth]
inherit = false
inherit_env = [{inherit_env}]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
run_as = "current"

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

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
