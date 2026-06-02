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

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
