use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

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
    assert!(codex.contains("~/.codex:/root/.codex:ro"));
    let pi = fs::read_to_string(home.path().join("agents/pi-coding-agent-default.toml")).unwrap();
    assert!(pi.contains("pi coding --version || pi --version"));
}
