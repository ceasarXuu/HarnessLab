use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

const MISSING_DOCKER_HOST: &str = "unix:///tmp/harnesslab-test-missing-docker.sock";

#[test]
fn doc_001_doctor_json_has_stable_shape() {
    let home = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
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
fn doc_002_doctor_text_reports_missing_home_config() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("doctor: error"));
}

#[test]
fn doc_003_doctor_reports_semantically_invalid_agent_profiles() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    fs::write(
        home.path().join("agents/bad.toml"),
        r#"schema_version = 2
name = "bad"
kind = "custom"
display_name = "Bad"
command = "missing-agent"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 1

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[usage]
parser = "none"
"#,
    )
    .unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("agent.bad.validation"))
        .stdout(predicate::str::contains("unsupported schema_version 2"));
}

#[test]
fn doc_004_doctor_reports_builtin_benchmark_readiness() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("benchmark.terminal-bench.smoke"))
        .stdout(predicate::str::contains("benchmark.swe-bench-pro.full"));
}

#[test]
fn doc_005_doctor_reports_agent_profile_warnings() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    fs::write(
        home.path().join("agents/socket.toml"),
        r#"schema_version = 1
name = "socket"
kind = "custom"
display_name = "Socket"
command = "sh"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 1

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = true

[usage]
parser = "none"
"#,
    )
    .unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("agent.socket.validation"))
        .stdout(predicate::str::contains("docker_socket_requested"));
}

#[test]
fn doc_006_doctor_reports_agent_profile_load_errors() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    fs::write(home.path().join("agents/broken.toml"), "not = [valid").unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .stdout(predicate::str::contains("agents.load"))
        .stdout(predicate::str::contains("Agent profiles failed to load"));
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}
