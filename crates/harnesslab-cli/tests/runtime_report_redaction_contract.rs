mod support;

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use support::assert_public_artifacts_do_not_contain;

#[test]
fn public_artifacts_redact_hardcoded_sensitive_tokens() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf ok > result.txt # sk-hardcoded",
        "printf sk-hardcoded",
        "printf sk-hardcoded >/tmp/harnesslab-hardcoded",
        &[],
    );

    let output = run_success(home.path(), &[]);
    let json: Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    assert_public_artifacts_do_not_contain(run_dir, "sk-hardcoded");
}

#[test]
fn replay_redacts_source_known_setup_and_version_secrets_without_current_env() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf ok > result.txt # do-not-leak",
        "printf do-not-leak",
        "printf do-not-leak >/tmp/harnesslab-replay-secret",
        &["HARNESSLAB_REPLAY_SECRET"],
    );
    let output = run_success(home.path(), &[("HARNESSLAB_REPLAY_SECRET", "do-not-leak")]);
    let json: Value = serde_json::from_slice(&output).unwrap();
    let source_run_dir = Path::new(json["run_dir"].as_str().unwrap());

    let replay_output = harnesslab()
        .env_remove("HARNESSLAB_REPLAY_SECRET")
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            source_run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let replay_json: Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_run_dir = Path::new(replay_json["run_dir"].as_str().unwrap());

    assert_public_artifacts_do_not_contain(replay_run_dir, "do-not-leak");
}

#[test]
fn replay_redacts_source_known_secret_embedded_in_version_assignment() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf ok > result.txt",
        r#"sh -c 'TOKEN=do-not-leak; printf %s "$TOKEN"'"#,
        "printf do-not-leak >/tmp/harnesslab-replay-secret",
        &["HARNESSLAB_REPLAY_SECRET"],
    );
    let output = run_success(home.path(), &[("HARNESSLAB_REPLAY_SECRET", "do-not-leak")]);
    let json: Value = serde_json::from_slice(&output).unwrap();
    let source_run_dir = Path::new(json["run_dir"].as_str().unwrap());

    let replay_output = harnesslab()
        .env_remove("HARNESSLAB_REPLAY_SECRET")
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            source_run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let replay_json: Value = serde_json::from_slice(&replay_output).unwrap();
    let replay_run_dir = Path::new(replay_json["run_dir"].as_str().unwrap());

    assert_public_artifacts_do_not_contain(replay_run_dir, "do-not-leak");
}

#[test]
fn resume_report_uses_persisted_materialized_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf ok > result.txt",
        "printf runtime-version",
        "printf setup >/tmp/harnesslab-setup",
        &[],
    );
    let output = run_success(home.path(), &[]);
    let json: Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let materialized_path = run_dir.join("agent-runtime.materialized.json");
    let mut materialized: Value =
        serde_json::from_slice(&fs::read(&materialized_path).unwrap()).unwrap();
    materialized["setup_summary"] = Value::String("DIFF_MARKER persisted setup".to_string());
    fs::write(
        &materialized_path,
        serde_json::to_string_pretty(&materialized).unwrap(),
    )
    .unwrap();

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
        .success();

    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("DIFF_MARKER persisted setup"));
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn run_success(home: &Path, env: &[(&str, &str)]) -> Vec<u8> {
    let mut command = harnesslab();
    for (key, value) in env {
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

fn write_agent(
    home: &Path,
    command: &str,
    version_command: &str,
    setup_command: &str,
    inherit_env: &[&str],
) {
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
version_command = "{}"
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

[setup]
preset = "custom"
required_commands = []
run_as = "current"
commands = ["{}"]

[usage]
parser = "none"
"#,
        toml_string(version_command),
        toml_string(setup_command)
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
