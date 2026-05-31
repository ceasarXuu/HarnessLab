use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn int_001_init_empty_home_creates_config_and_profiles() {
    let home = tempfile::tempdir().unwrap();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.path().to_str().unwrap(), "init", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"init\""));

    for path in [
        "config.toml",
        "agents/codex-default.toml",
        "agents/claude-code-default.toml",
        "agents/opencode-default.toml",
        "agents/pi-coding-agent-default.toml",
        "runs",
    ] {
        assert!(home.path().join(path).exists());
    }

    let codex = fs::read_to_string(home.path().join("agents/codex-default.toml")).unwrap();
    assert!(codex.contains("OPENAI_API_KEY"));
    assert!(codex.contains("~/.codex:/root/.codex:rw"));
    let pi = fs::read_to_string(home.path().join("agents/pi-coding-agent-default.toml")).unwrap();
    assert!(pi.contains("pi coding --version || pi --version"));
}

#[test]
fn int_001_report_open_latest_uses_configured_runs_dir() {
    let home = tempfile::tempdir().unwrap();
    harnesslab()
        .args(["--home", home.path().to_str().unwrap(), "init"])
        .assert()
        .success();
    let mut config = fs::read_to_string(home.path().join("config.toml")).unwrap();
    config = config.replace(
        "runs_dir = \"~/.harnesslab/runs\"",
        "runs_dir = \"custom-runs\"",
    );
    fs::write(home.path().join("config.toml"), config).unwrap();
    write_agent(home.path());

    let output = harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
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
    let run_dir = json["run_dir"].as_str().unwrap();
    assert!(run_dir.contains("custom-runs"));

    harnesslab()
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "report",
            "open",
            "latest",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-runs"));
}

fn write_agent(home: &Path) {
    fs::write(
        home.join("agents/fake.toml"),
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "printf ok > result.txt"
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
"#,
    )
    .unwrap();
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
