use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn int_008_resume_failed_run_recovers_once_and_reports_latest_attempt() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "case \"$PWD\" in */attempts/2/workspace) printf ok > result.txt;; *) exit 7;; esac",
    );
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
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
        .code(1)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());

    resume_success(home.path(), run_dir);
    resume_success(home.path(), run_dir);

    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["summary"]["total_tasks"], 1);
    assert_eq!(results["summary"]["success"], 1);
    assert_eq!(results["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(results["tasks"][0]["provenance"], "original");
    assert_eq!(results["tasks"][1]["provenance"], "recovery");
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("Resume: yes"));
    assert_report_row_provenance(
        &report,
        results["tasks"][0]["task_id"].as_str().unwrap(),
        1,
        "original",
    );
    assert_report_row_provenance(
        &report,
        results["tasks"][1]["task_id"].as_str().unwrap(),
        2,
        "recovery",
    );
    assert!(!report.contains("<td>resumed run</td>"));
    assert!(report.contains("Failure"));
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("run_resumed"));
    assert_eq!(events.matches("recovery_attempt_scheduled").count(), 1);
}

#[test]
fn int_008_resume_missing_planned_attempt_reports_resumed_provenance() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "case \"$PWD\" in */attempts/2/workspace) printf ok > result.txt;; *) exit 7;; esac",
    );
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
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
        .code(1)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    force_configured_attempts(run_dir, 2);

    resume_success(home.path(), run_dir);

    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["summary"]["total_tasks"], 1);
    assert_eq!(results["summary"]["success"], 1);
    assert_eq!(results["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(results["tasks"][0]["provenance"], "original");
    assert_eq!(results["tasks"][1]["provenance"], "resumed");
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert_report_row_provenance(
        &report,
        results["tasks"][0]["task_id"].as_str().unwrap(),
        1,
        "original",
    );
    assert_report_row_provenance(
        &report,
        results["tasks"][1]["task_id"].as_str().unwrap(),
        2,
        "resumed",
    );
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("run_resumed"));
    assert!(!events.contains("recovery_attempt_scheduled"));
}

#[test]
fn int_016_resume_interrupted_attempt_schedules_recovery_attempt() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(
        home.path(),
        "case \"$PWD\" in */attempts/2/workspace) printf ok > result.txt;; *) exit 7;; esac",
    );
    let output = Command::cargo_bin("harnesslab")
        .unwrap()
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
        .code(1)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    mark_first_attempt_interrupted(run_dir);

    resume_success(home.path(), run_dir);

    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["summary"]["total_tasks"], 1);
    assert_eq!(results["summary"]["success"], 1);
    assert_eq!(results["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(results["tasks"][0]["state"], "interrupted");
    assert_eq!(results["tasks"][0]["provenance"], "original");
    assert_eq!(results["tasks"][1]["provenance"], "recovery");
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("run_resumed"));
    assert_eq!(events.matches("recovery_attempt_scheduled").count(), 1);
}

fn resume_success(home: &Path, run_dir: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args([
            "--home",
            home.to_str().unwrap(),
            "run",
            "resume",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
}

fn force_configured_attempts(run_dir: &Path, attempts: u32) {
    let path = run_dir.join("run.json");
    let mut run: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    run["execution"]["attempts"] = serde_json::json!(attempts);
    fs::write(path, serde_json::to_vec_pretty(&run).unwrap()).unwrap();
}

fn mark_first_attempt_interrupted(run_dir: &Path) {
    let path = run_dir.join("tasks/fake-terminal-success/attempts/1/result.json");
    let mut result: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    result["state"] = serde_json::json!("interrupted");
    fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

fn assert_report_row_provenance(report: &str, task_id: &str, attempt: u32, provenance: &str) {
    let task_cell = format!("<td>{task_id}</td>");
    let mut offset = 0;
    while let Some(relative_pos) = report[offset..].find(&task_cell) {
        let task_pos = offset + relative_pos;
        let row_end = report[task_pos..].find("</tr>").unwrap_or_else(|| {
            panic!("report row for task {task_id} has no closing tag");
        });
        let row = &report[task_pos..task_pos + row_end];
        if row.contains(&format!("<td>{attempt}</td>"))
            && row.contains(&format!("<td>{provenance}</td>"))
        {
            return;
        }
        offset = task_pos + task_cell.len();
    }
    panic!(
        "report row for task {task_id} attempt {attempt} did not contain provenance {provenance}"
    );
}

fn init_home(home: &Path) {
    Command::cargo_bin("harnesslab")
        .unwrap()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
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
