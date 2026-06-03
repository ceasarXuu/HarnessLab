use assert_cmd::Command;
use std::fs;
use std::path::Path;

const MISSING_DOCKER_HOST: &str = "unix:///tmp/harnesslab-test-missing-docker.sock";

#[test]
fn agt_reg_012_doctor_warns_non_current_run_as_requires_sandbox() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "sandbox-user", "harnesslab");

    let output = doctor_json(home.path());
    let check = find_check(&output, "agent.sandbox-user.setup.run_as");

    assert_eq!(check["status"], "warning");
    assert_eq!(check["severity"], "warning");
    assert_eq!(check["details"]["field"], "setup.run_as");
    assert_eq!(check["details"]["run_as"], "harnesslab");
    assert_eq!(check["details"]["host_supported"], false);
    assert_eq!(check["details"]["sandbox_required"], true);
    assert!(
        check["message"]
            .as_str()
            .unwrap()
            .contains("host tasks cannot switch users")
    );
}

#[test]
fn agt_reg_012_doctor_warns_root_run_as_requires_sandbox() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "root-user", "root");

    let output = doctor_json(home.path());
    let check = find_check(&output, "agent.root-user.setup.run_as");

    assert_eq!(check["status"], "warning");
    assert_eq!(check["details"]["run_as"], "root");
    assert_eq!(check["details"]["host_supported"], false);
    assert_eq!(check["details"]["sandbox_required"], true);
}

#[test]
fn agt_reg_012_doctor_errors_for_host_agent_subpath_with_non_current_run_as() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels(
        home.path(),
        "terminal-import",
        "harnesslab",
        r#"terminal_bench_agent_import_path = "pkg.agent:Agent"
"#,
    );

    let output = doctor_json(home.path());
    let check = find_check(&output, "agent.terminal-import.setup.run_as");

    assert_eq!(check["status"], "error");
    assert_eq!(check["severity"], "error");
    assert_eq!(check["details"]["run_as"], "harnesslab");
    assert!(
        check["details"]["blocked_host_agent_paths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "terminal-bench import agent host path")
    );
}

#[test]
fn agt_reg_012_doctor_accepts_current_run_as_for_host() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "current-user", "current");

    let output = doctor_json(home.path());
    let check = find_check(&output, "agent.current-user.setup.run_as");

    assert_eq!(check["status"], "ok");
    assert_eq!(check["severity"], "warning");
    assert_eq!(check["details"]["run_as"], "current");
    assert_eq!(check["details"]["host_supported"], true);
    assert_eq!(check["details"]["sandbox_required"], false);
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn doctor_json(home: &Path) -> serde_json::Value {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("DOCKER_HOST", MISSING_DOCKER_HOST)
        .args(["--home", home.to_str().unwrap(), "doctor", "--json"])
        .assert()
        .code(3)
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

fn find_check<'a>(json: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == id)
        .unwrap_or_else(|| panic!("missing doctor check {id}"))
}

fn write_agent(home: &Path, name: &str, run_as: &str) {
    write_agent_with_labels(home, name, run_as, "");
}

fn write_agent_with_labels(home: &Path, name: &str, run_as: &str, labels: &str) {
    fs::write(
        home.join(format!("agents/{name}.toml")),
        format!(
            r#"schema_version = 1
name = "{name}"
kind = "fake"
display_name = "{name}"
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
mount_docker_socket = false

[setup]
preset = "none"
required_commands = []
run_as = "{run_as}"
commands = []

[usage]
parser = "none"

[labels]
{labels}
"#,
        ),
    )
    .unwrap();
}
