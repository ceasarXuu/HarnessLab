use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn use_005_usage_regex_parser_records_tokens_and_report_text() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf 'input_tokens=4 output_tokens=6\\n'; printf ok > result.txt",
    );
    patch_usage_config(home.path(), "parser = \"regex\"");

    let (results, report) = run_and_load(home.path());

    assert_eq!(results["tasks"][0]["usage"]["total_tokens"], 10);
    assert!(report.contains("10 tokens"));
}

#[test]
fn use_005_usage_json_path_parser_records_cost_and_report_text() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "printf '{\"usage\":{\"input\":4,\"output\":6,\"cost\":0.012}}'; printf ok > result.txt",
    );
    patch_usage_config(
        home.path(),
        &[
            "parser = \"json_path\"",
            "source = \"agent_stdout\"",
            "input_tokens_key = \"usage.input\"",
            "output_tokens_key = \"usage.output\"",
            "cost_usd_key = \"usage.cost\"",
        ]
        .join("\n"),
    );

    let (results, report) = run_and_load(home.path());

    assert_eq!(results["tasks"][0]["usage"]["total_tokens"], 10);
    assert_eq!(results["tasks"][0]["usage"]["cost_usd"], 0.012);
    assert!(report.contains("$0.012000"));
}

#[test]
fn use_005_usage_parser_failure_is_persisted_and_reported() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf no-usage; printf ok > result.txt");
    patch_usage_config(home.path(), "parser = \"regex\"");

    let (results, report) = run_and_load(home.path());

    assert_eq!(
        results["tasks"][0]["warnings"][0],
        serde_json::json!("usage_parser_failed")
    );
    assert!(report.contains("parse error: usage tokens not found"));
    assert!(report.contains("cost not comparable"));
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn patch_usage_config(home: &Path, replacement: &str) {
    let profile = home.join("agents/fake.toml");
    let content = fs::read_to_string(&profile)
        .unwrap()
        .replace("parser = \"none\"", replacement);
    fs::write(&profile, content).unwrap();
}

fn write_agent(home: &Path, command: &str) {
    let command = command.replace('\\', "\\\\").replace('"', "\\\"");
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{command}"
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

[usage]
parser = "none"
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn run_and_load(home: &Path) -> (serde_json::Value, String) {
    let output = Command::cargo_bin("harnesslab")
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
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let results = serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    (results, report)
}
