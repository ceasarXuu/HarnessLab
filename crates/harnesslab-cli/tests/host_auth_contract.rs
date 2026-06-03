use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn agt_reg_012_host_agent_does_not_see_ambient_env_when_auth_inherit_false() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        r#"if [ -n "${HARNESSLAB_HOST_AUTH_SECRET:-}" ]; then exit 66; fi; printf ok > result.txt"#,
        false,
        &[],
    );

    let output = run_success_with_env(home.path(), "HARNESSLAB_HOST_AUTH_SECRET", "hidden");
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    assert_eq!(json["verdict"], "success");
    let stdout =
        fs::read_to_string(run_dir.join("tasks/fake-terminal-success/attempts/1/agent/stdout.log"))
            .unwrap();
    assert!(stdout.is_empty());
}

#[test]
fn agt_reg_012_host_agent_sees_declared_env_when_auth_inherit_true() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        r#"test "$HARNESSLAB_HOST_AUTH_SECRET" = "allowed" && printf ok > result.txt"#,
        true,
        &["HARNESSLAB_HOST_AUTH_SECRET"],
    );

    let output = run_success_with_env(home.path(), "HARNESSLAB_HOST_AUTH_SECRET", "allowed");
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["verdict"], "success");
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str, inherit: bool, inherit_env: &[&str]) {
    let inherit_env = inherit_env
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        home.join("agents/fake.toml"),
        format!(
            r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{}"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 5

[auth]
inherit = {inherit}
inherit_env = [{inherit_env}]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "none"
required_commands = []
run_as = "current"
commands = []

[usage]
parser = "none"
"#,
            escape_toml(command),
        ),
    )
    .unwrap();
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn run_success_with_env(home: &Path, key: &str, value: &str) -> Vec<u8> {
    harnesslab()
        .env(key, value)
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
        .stdout(predicate::str::contains("\"verdict\":\"success\""))
        .get_output()
        .stdout
        .clone()
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
