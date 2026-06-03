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
