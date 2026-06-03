use assert_cmd::Command;
use std::fs;
use std::path::Path;

const MISSING_DOCKER_HOST: &str = "unix:///tmp/harnesslab-test-missing-docker.sock";

#[test]
fn agt_reg_011_doctor_errors_when_required_command_is_missing_without_setup() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "missing-required",
        "fake",
        "none",
        "current",
        r#"["definitely-missing-harnesslab-required-command"]"#,
    );

    let json = doctor_json(home.path());
    let check = find_check(&json, "agent.missing-required.setup.required_commands");
    let command = required_command_entry(check, "definitely-missing-harnesslab-required-command");

    assert_eq!(check["status"], "error");
    assert_eq!(command["field"], "setup.required_commands[0]");
    assert_eq!(command["valid_name"], true);
    assert_eq!(command["host_available"], false);
    assert_eq!(command["provider"], "none");
    assert_eq!(command["status"], "error");
    assert!(
        command["message"]
            .as_str()
            .unwrap()
            .contains("no setup path")
    );
}

#[test]
fn agt_reg_011_doctor_explains_builtin_setup_can_provide_required_command() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "claude-required",
        "claude-code",
        "builtin",
        "harnesslab",
        r#"["claude"]"#,
    );

    let json = doctor_json(home.path());
    let check = find_check(&json, "agent.claude-required.setup.required_commands");
    let command = required_command_entry(check, "claude");

    assert_eq!(check["status"], "ok");
    assert_eq!(command["field"], "setup.required_commands[0]");
    assert_eq!(command["provider"], "builtin_setup");
    assert_eq!(command["status"], "ok");
    assert!(
        command["message"]
            .as_str()
            .unwrap()
            .contains("builtin setup")
    );
}

#[test]
fn agt_reg_011_doctor_marks_custom_setup_required_command_as_sandbox_dependent() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "custom-required",
        "custom",
        "custom",
        "current",
        r#"["definitely-missing-harnesslab-custom-command"]"#,
    );

    let json = doctor_json(home.path());
    let check = find_check(&json, "agent.custom-required.setup.required_commands");
    let command = required_command_entry(check, "definitely-missing-harnesslab-custom-command");

    assert_eq!(check["status"], "ok");
    assert_eq!(command["field"], "setup.required_commands[0]");
    assert_eq!(command["host_available"], false);
    assert_eq!(command["provider"], "custom_setup");
    assert_eq!(command["status"], "ok");
    assert!(
        command["message"]
            .as_str()
            .unwrap()
            .contains("custom setup")
    );
}

#[test]
fn agt_reg_011_doctor_reports_invalid_required_command_field() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "invalid-required",
        "fake",
        "none",
        "current",
        r#"["foo | bar"]"#,
    );

    let json = doctor_json(home.path());
    let validation = find_check(&json, "agent.invalid-required.validation");
    let check = find_check(&json, "agent.invalid-required.setup.required_commands");
    let command = required_command_entry(check, "foo | bar");

    assert_error_field(
        validation,
        "setup.required_commands",
        "letters",
        "bare command",
    );
    assert_eq!(check["status"], "error");
    assert_eq!(command["field"], "setup.required_commands[0]");
    assert_eq!(command["valid_name"], false);
    assert_eq!(command["status"], "error");
    assert!(
        command["message"]
            .as_str()
            .unwrap()
            .contains("bare command names")
    );
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(
    home: &Path,
    name: &str,
    kind: &str,
    setup_preset: &str,
    run_as: &str,
    required_commands: &str,
) {
    fs::write(
        home.join("agents").join(format!("{name}.toml")),
        format!(
            r#"schema_version = 1
name = "{name}"
kind = "{kind}"
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
preset = "{setup_preset}"
required_commands = {required_commands}
run_as = "{run_as}"
commands = []

[usage]
parser = "none"
"#,
        ),
    )
    .unwrap();
}

fn doctor_json(home: &Path) -> serde_json::Value {
    let output = harnesslab()
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

fn required_command_entry<'a>(
    check: &'a serde_json::Value,
    command: &str,
) -> &'a serde_json::Value {
    check["details"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["command"] == command)
        .unwrap_or_else(|| panic!("missing required command {command}"))
}

fn assert_error_field(check: &serde_json::Value, field: &str, accepted: &str, suggested: &str) {
    let error = check["details"]["errors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|error| error["field"] == field)
        .unwrap_or_else(|| panic!("missing error field {field}"));
    assert!(
        error["accepted_values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap_or_default().contains(accepted)),
        "missing accepted value {accepted} in {error}"
    );
    assert!(
        error["suggested_fix"].as_str().unwrap().contains(suggested),
        "missing suggested fix {suggested} in {error}"
    );
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
