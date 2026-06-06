use std::fs;
use std::path::Path;

pub fn write_import_agent_with_private_material(
    home: &Path,
    import_path: &str,
    pythonpath: &str,
    agent_command: &str,
    setup_command: &str,
) {
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{agent_command}"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 5

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "custom"
required_commands = []
run_as = "current"
commands = ["{setup_command}"]

[usage]
parser = "none"

[labels]
terminal_bench_agent = "oracle"
terminal_bench_agent_import_path = "{import_path}"
terminal_bench_agent_pythonpath = "{pythonpath}"
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

pub fn assert_json_array_has_name(array: &serde_json::Value, name: &str) {
    assert!(
        array
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["name"] == name),
        "missing name {name}: {array:#?}"
    );
}

pub fn assert_json_array_missing_name(array: &serde_json::Value, name: &str) {
    assert!(
        !array
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["name"] == name),
        "unexpected name {name}: {array:#?}"
    );
}

pub fn assert_json_array_has_phase(array: &serde_json::Value, phase: &str) {
    assert!(
        array
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value["phase"] == phase),
        "missing phase {phase}: {array:#?}"
    );
}

pub fn material_public_path(array: &serde_json::Value, name: &str) -> String {
    array
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["name"] == name)
        .and_then(|value| value["public_path"].as_str())
        .unwrap_or_else(|| panic!("missing public_path for material {name}: {array:#?}"))
        .to_string()
}

pub fn material_kind(array: &serde_json::Value, name: &str) -> String {
    array
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["name"] == name)
        .and_then(|value| value["kind"].as_str())
        .unwrap_or_else(|| panic!("missing kind for material {name}: {array:#?}"))
        .to_string()
}

pub fn artifact_list_contains(array: &serde_json::Value, artifact: &str) -> bool {
    array
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value.as_str() == Some(artifact))
}

pub fn assert_public_artifacts_eq(array: &serde_json::Value, expected: &[&str]) {
    let mut actual = array
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let mut expected = expected
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected);
    for forbidden in [
        "agent/command.txt",
        "agent/stdout.log",
        "agent/stderr.log",
        "verifier/stdout.log",
        "verifier/stderr.log",
    ] {
        assert!(!actual.iter().any(|artifact| artifact == forbidden));
    }
}

pub fn assert_public_text_file_does_not_contain(path: &Path, secret: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(!content.contains(secret), "public artifact leaked");
}

pub fn assert_public_text_file_contains(path: &Path, expected: &str) {
    let content = fs::read_to_string(path).unwrap();
    assert!(content.contains(expected), "public artifact missed marker");
}
