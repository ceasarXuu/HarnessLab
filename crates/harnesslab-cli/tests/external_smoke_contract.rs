use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn int_011_terminal_bench_smoke_without_docker_reports_sandbox_failure() {
    if docker_is_available() {
        return;
    }
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf terminal-bench-smoke > result.txt");

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "terminal-bench",
            "--split",
            "smoke",
            "--json",
        ])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"status\":\"failure\""))
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let results_path = Path::new(json["run_dir"].as_str().unwrap()).join("results.json");
    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(results_path).unwrap()).unwrap();
    assert_eq!(results["tasks"][0]["failure_code"], "sandbox_create_failed");
}

fn docker_is_available() -> bool {
    std::process::Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str) {
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
