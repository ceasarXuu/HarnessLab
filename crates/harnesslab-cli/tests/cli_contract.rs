use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_001_help_lists_m0_commands() {
    let mut cmd = Command::cargo_bin("harnesslab").unwrap();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("agent"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("benchmark"))
        .stdout(predicate::str::contains("run"))
        .stdout(predicate::str::contains("report"));
}

#[test]
fn cli_002_resume_and_replay_are_nested_under_run() {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["resume", "/tmp/run"])
        .assert()
        .failure();

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["run", "resume", "/tmp/run", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"run resume\""));

    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["run", "replay", "/tmp/run", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\":\"run replay\""));
}

#[test]
fn doc_001_doctor_json_has_stable_shape() {
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["doctor", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["status"], "ok");
    let checks = json["checks"].as_array().unwrap();
    assert!(!checks.is_empty());
    let check = &checks[0];
    assert!(check["id"].as_str().is_some());
    assert!(check["status"].as_str().is_some());
    assert!(check["severity"].as_str().is_some());
    assert!(check["message"].as_str().is_some());
    assert!(check["details"].is_object());
}

#[test]
fn cli_003_m0_json_commands_have_stable_shape() {
    let cases = [
        (vec!["init", "--json"], "init"),
        (vec!["agent", "list", "--json"], "agent list"),
        (vec!["benchmark", "list", "--json"], "benchmark list"),
        (
            vec!["benchmark", "info", "terminal-bench", "--json"],
            "benchmark info",
        ),
        (
            vec![
                "run",
                "--agent",
                "codex-default",
                "--benchmark",
                "fake-terminal",
                "--split",
                "smoke",
                "--json",
            ],
            "run",
        ),
        (vec!["report", "open", "latest", "--json"], "report open"),
    ];

    for (args, command_name) in cases {
        let output = Command::cargo_bin("harnesslab")
            .unwrap()
            .args(args)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["command"], command_name);
    }
}

#[test]
fn cli_004_m0_text_commands_succeed() {
    let cases = [
        vec!["init"],
        vec!["agent", "list"],
        vec!["doctor"],
        vec!["benchmark", "list"],
        vec!["benchmark", "info", "terminal-bench"],
        vec![
            "run",
            "--agent",
            "codex-default",
            "--benchmark",
            "fake-terminal",
            "--split",
            "smoke",
        ],
        vec!["run", "resume", "/tmp/run"],
        vec!["run", "replay", "/tmp/run"],
        vec!["report", "open", "latest"],
    ];

    for args in cases {
        Command::cargo_bin("harnesslab")
            .unwrap()
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}
