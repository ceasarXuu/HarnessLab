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
        .stdout(predicate::str::contains("benchmark.swe-bench-pro.full"))
        .stdout(predicate::str::contains("data_state=requires_auth"));
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

#[test]
fn doc_007_doctor_reports_auth_and_usage_configuration_problems() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    let existing_auth_dir = home.path().join("existing-auth-dir");
    let excluded_auth_dir = home.path().join("excluded-auth-dir");
    fs::create_dir_all(&existing_auth_dir).unwrap();
    fs::create_dir_all(&excluded_auth_dir).unwrap();
    let existing_mount = format!("{}:/root/existing-auth-dir:ro", existing_auth_dir.display());
    let excluded_mount = format!("{}:/root/excluded-auth-dir:ro", excluded_auth_dir.display());
    fs::write(
        home.path().join("agents/auth-usage.toml"),
        format!(
            r#"schema_version = 1
name = "auth-usage"
kind = "custom"
display_name = "Auth Usage"
command = "sh"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 1

[auth]
inherit = true
inherit_env = ["HARNESSLAB_DOCTOR_MISSING_TOKEN"]
include_paths = [
  "missing-auth-dir:/root/missing-auth-dir:ro",
  "{existing_mount}",
  "{excluded_mount}",
]
exclude_paths = ["{}"]
mount_ssh_socket = true
mount_docker_socket = false

[usage]
parser = "mystery"
source = "unsafe/../../usage.json"
"#,
            excluded_auth_dir.display(),
        ),
    )
    .unwrap();

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .env_remove("HARNESSLAB_DOCTOR_MISSING_TOKEN")
        .env_remove("SSH_AUTH_SOCK")
        .args(["--home", home.path().to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let include_paths = find_check(&json, "agent.auth-usage.auth.include_paths");
    assert_eq!(include_paths["status"], "warning");
    assert_eq!(include_paths["severity"], "warning");
    assert!(
        include_paths["details"]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path["host_path"] == "missing-auth-dir" && path["exists"] == false)
    );
    let docker_mount = find_check(&json, "agent.auth-usage.auth.docker_mount");
    assert_eq!(docker_mount["status"], "error");
    assert_eq!(docker_mount["severity"], "error");
    assert!(
        docker_mount["details"]["mounts_checked"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mount| mount.as_str() == Some(existing_mount.as_str()))
    );
    assert!(
        !docker_mount["details"]["mounts_checked"]
            .as_array()
            .unwrap()
            .iter()
            .any(|mount| mount.as_str() == Some(excluded_mount.as_str()))
    );
    let ssh_socket = find_check(&json, "agent.auth-usage.auth.ssh_socket");
    assert_eq!(ssh_socket["status"], "warning");
    assert_eq!(ssh_socket["severity"], "warning");
    let usage = find_check(&json, "agent.auth-usage.usage");
    assert_eq!(usage["status"], "warning");
    assert_eq!(usage["severity"], "warning");
    assert!(
        usage["message"]
            .as_str()
            .unwrap()
            .contains("unknown usage parser")
    );
}

fn find_check<'a>(json: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == id)
        .unwrap_or_else(|| panic!("missing doctor check {id}"))
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}
