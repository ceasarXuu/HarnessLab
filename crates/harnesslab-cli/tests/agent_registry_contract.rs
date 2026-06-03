use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn agt_reg_003_agent_schema_json_exposes_profile_field_ranges() {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["agent", "schema", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["command"], "agent schema");
    assert_field_value(&json, "setup.preset", "builtin");
    assert_field_value(&json, "setup.run_as", "harnesslab");
    assert_field_example(&json, "skills.allow", serde_json::json!(["skill-a"]));
    assert_field_example(&json, "tools.deny", serde_json::json!(["web_search"]));
    assert_field_example(&json, "hooks.deny", serde_json::json!(["post_tool_use"]));
    assert_field_example(&json, "usage.source", serde_json::json!("agent_logs"));
    assert_field_value(
        &json,
        "labels.terminal_bench_agent_import_path",
        "python import path",
    );
    assert_eq!(
        find_field(&json, "labels.sandbox_setup_command")["status"],
        "legacy"
    );
}

#[test]
fn agt_reg_007_agent_schema_json_covers_supported_profile_parameters() {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["agent", "schema", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let expected = [
        "schema_version",
        "name",
        "kind",
        "display_name",
        "command",
        "input_mode",
        "working_dir",
        "timeout_sec",
        "version_command",
        "auth.inherit",
        "auth.inherit_env",
        "auth.include_paths",
        "auth.exclude_paths",
        "auth.mount_ssh_socket",
        "auth.mount_docker_socket",
        "setup.preset",
        "setup.required_commands",
        "setup.run_as",
        "setup.commands",
        "skills.inherit",
        "skills.allow",
        "skills.deny",
        "skills.include_paths",
        "tools.inherit",
        "tools.allow",
        "tools.deny",
        "hooks.inherit",
        "hooks.allow",
        "hooks.deny",
        "usage.parser",
        "usage.source",
        "usage.input_tokens_key",
        "usage.output_tokens_key",
        "usage.total_tokens_key",
        "usage.cost_usd_key",
        "labels",
        "labels.model",
        "labels.terminal_bench_agent",
        "labels.terminal_bench_agent_import_path",
        "labels.terminal_bench_agent_pythonpath",
        "labels.terminal_bench_model",
        "labels.sandbox_setup_command",
    ];
    for path in expected {
        let field = find_field(&json, path);
        assert!(
            field["description"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "missing description for {path}"
        );
        assert!(
            field["allowed_values"]
                .as_array()
                .is_some_and(|values| !values.is_empty()),
            "missing allowed values for {path}"
        );
        assert!(field.get("example").is_some(), "missing example for {path}");
        assert!(
            field["status"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "missing status for {path}"
        );
    }
}

#[test]
fn agt_reg_006_non_materializable_policy_blocks_run_before_run_dir() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_bad_tools_agent(home.path());

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "bad-tools",
            "--benchmark",
            "fake-terminal",
            "--split",
            "success",
            "--json",
        ])
        .assert()
        .code(3)
        .get_output()
        .stderr
        .clone();

    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("tools"));
    assert!(stderr.contains("not materializable"));
    assert_eq!(fs::read_dir(home.path().join("runs")).unwrap().count(), 0);
}

fn assert_field_value(json: &serde_json::Value, path: &str, expected: &str) {
    let field = find_field(json, path);
    assert!(
        field["allowed_values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str() == Some(expected))
    );
}

fn assert_field_example(json: &serde_json::Value, path: &str, expected: serde_json::Value) {
    let field = find_field(json, path);
    assert_eq!(field["example"], expected);
}

fn find_field<'a>(json: &'a serde_json::Value, path: &str) -> &'a serde_json::Value {
    json["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|field| field["path"] == path)
        .unwrap_or_else(|| panic!("missing schema field {path}"))
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_bad_tools_agent(home: &Path) {
    fs::write(
        home.join("agents/bad-tools.toml"),
        r#"schema_version = 1
name = "bad-tools"
kind = "custom"
display_name = "Bad Tools"
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
run_as = "current"
commands = []

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = ["bash"]

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"
"#,
    )
    .unwrap();
}
