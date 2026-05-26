use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn cli_001_help_lists_m0_commands() {
    let mut cmd = Command::cargo_bin("harnesslab").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("agent"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("benchmark"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("report"));
}

#[test]
fn cli_002_resume_and_replay_are_nested_under_run() {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["resume", "/tmp/run"])
        .assert()
        .failure();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["run", "resume", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: harnesslab run resume"));

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["run", "replay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: harnesslab run replay"));
}

#[test]
fn doc_001_doctor_json_has_stable_shape() {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["doctor", "--json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert!(matches!(
        json["status"].as_str(),
        Some("ok" | "warning" | "error")
    ));
    let checks = json["checks"].as_array().unwrap();
    assert!(!checks.is_empty());
    let check = &checks[0];
    assert!(check["id"].as_str().is_some());
    assert!(check["status"].as_str().is_some());
    assert!(check["severity"].as_str().is_some());
    assert!(check["message"].as_str().is_some());
    assert!(check["details"].is_object());
}

#[test]
fn cli_003_m0_json_commands_have_stable_shape() {
    let cases = [
        (vec!["init", "--json"], "init"),
        (vec!["agent", "list", "--json"], "agent list"),
        (vec!["benchmark", "list", "--json"], "benchmark list"),
        (
            vec!["benchmark", "info", "terminal-bench", "--json"],
            "benchmark info",
        ),
    ];

    for (args, command_name) in cases {
        let output = Command::cargo_bin("harnesslab")
            .unwrap()
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["command"], command_name);
    }
}

#[test]
fn cli_004_m0_text_commands_succeed() {
    let cases = [
        vec!["init"],
        vec!["agent", "list"],
        vec!["benchmark", "list"],
        vec!["benchmark", "info", "terminal-bench"],
    ];

    for args in cases {
        Command::cargo_bin("harnesslab")
            .unwrap()
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}

#[test]
fn int_001_init_empty_home_creates_config_and_profiles() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.path().to_str().unwrap(), "init", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"init\""));

    assert!(home.path().join("config.toml").exists());
    assert!(home.path().join("agents/codex-default.toml").exists());
    assert!(home.path().join("runs").exists());
}

#[test]
fn int_003_fake_terminal_success_creates_report_and_results() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
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
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    assert!(run_dir.join("results.json").exists());
    assert!(run_dir.join("report.html").exists());

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "report",
            "open",
            "latest",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"report open\""));
}

#[test]
fn int_004_fake_terminal_test_fail_exits_2() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-terminal",
            "--split",
            "test-fail",
        ])
        .assert()
        .code(2);
}

#[test]
fn int_005_fake_terminal_timeout_exits_1() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "sleep 2", 5);

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-terminal",
            "--split",
            "agent-timeout",
        ])
        .assert()
        .code(1);
}

#[test]
fn int_005_fake_terminal_agent_crash_exits_1() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "exit 7", 5);

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-terminal",
            "--split",
            "agent-crash",
        ])
        .assert()
        .code(1);
}

#[test]
fn int_006_fake_patch_success_saves_diff() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf new > app.txt", 5);

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-patch",
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
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    assert!(
        run_dir
            .join("tasks/fake-patch-success/attempts/1/patch.diff")
            .exists()
    );
}

#[test]
fn int_007_fake_patch_no_diff_exits_2() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "true", 5);

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "fake-patch",
            "--split",
            "no-diff",
        ])
        .assert()
        .code(2);
}

#[test]
fn int_008_resume_completed_run_succeeds() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let source = json["run_dir"].as_str().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "resume",
            source,
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"run resume\""));
}

#[test]
fn int_008_resume_text_output_succeeds() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let source = json["run_dir"].as_str().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "resume",
            source,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("run resume:"));
}

#[test]
fn int_009_replay_success_creates_new_run() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let source = json["run_dir"].as_str().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            source,
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"replay_source_run_id\""));
}

#[test]
fn int_010_replay_missing_agent_blocks_before_execution() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let source = Path::new(json["run_dir"].as_str().unwrap());
    let profile = source.join("agent-profile.snapshot.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&profile).unwrap()).unwrap();
    value["command"] = serde_json::json!("missing-harnesslab-agent");
    fs::write(&profile, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            source.to_str().unwrap(),
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("replay blocker"));
}

#[test]
fn doc_002_doctor_text_reports_missing_home_config() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.path().to_str().unwrap(), "doctor"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("doctor: error"));
}

#[test]
fn cli_005_agent_list_json_allows_uninitialized_home() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "agent",
            "list",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"items\":[]"));
}

#[test]
fn cli_006_report_open_explicit_path_text_succeeds() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", 5);
    let output = run_success(home.path());
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let source = json["run_dir"].as_str().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "report",
            "open",
            source,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("report.html"));
}

#[test]
fn cli_007_run_and_benchmark_preflight_errors_are_clear() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.path().to_str().unwrap(), "run"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("--agent is required"));

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["benchmark", "info", "missing-benchmark"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("unknown benchmark"));
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str, timeout_sec: u64) {
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{command}"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = {timeout_sec}

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
