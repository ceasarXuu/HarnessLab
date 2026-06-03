use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn int_004_fake_terminal_test_fail_exits_0_with_benchmark_verdict() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt");

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
            "test-fail",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["verdict"], "benchmark_failure");
    assert_eq!(json["summary"]["benchmark_failure"], 1);

    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    assert_eq!(
        json["report_path"],
        run_dir.join("report.html").display().to_string()
    );
    assert_eq!(
        json["results_path"],
        run_dir.join("results.json").display().to_string()
    );
    let results: serde_json::Value =
        serde_json::from_reader(fs::File::open(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["report_path"], json["report_path"]);
    assert_eq!(results["summary"]["benchmark_failure"], 1);

    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("exit_code=0"));
    assert!(events.contains("benchmark_failure=1"));
    assert!(events.contains("report_path="));
}

#[test]
fn agt_reg_004_run_persists_redacted_materialized_runtime_snapshot() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_custom_setup(home.path(), "do-not-leak");

    let output = harnesslab()
        .env("HARNESSLAB_AGENT_REG_SECRET", "do-not-leak")
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
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let command = fs::read_to_string(run_dir.join("command.txt")).unwrap();
    assert!(command.contains("agent_materialized_snapshot=agent-runtime.materialized.json"));

    let materialized = fs::read_to_string(run_dir.join("agent-runtime.materialized.json")).unwrap();
    assert!(materialized.contains("advanced_custom_setup"));
    assert!(materialized.contains("[REDACTED]"));
    assert!(!materialized.contains("do-not-leak"));

    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("Setup:"));
    assert!(report.contains("Skills:"));
    assert!(report.contains("Tools:"));
    assert!(report.contains("Hooks:"));
}

fn init_home(home: &Path) {
    harnesslab()
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

fn write_agent_with_custom_setup(home: &Path, secret: &str) {
    let content = format!(
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
inherit_env = ["HARNESSLAB_AGENT_REG_SECRET"]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "custom"
required_commands = []
run_as = "current"
commands = ["printf {secret} >/tmp/harnesslab-agent-registry-secret"]

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = []

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
