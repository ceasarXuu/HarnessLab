mod support;

use std::fs;
use std::path::Path;
use support::swe::{
    fake_swe_tools, init_home, run_swe_json, swe_bench_root, write_agent, write_swe_gold_agent,
};

#[test]
fn swepro_001_metadata_failure_is_classified_and_observable() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "swe-gold",
        &[("HARNESSLAB_FAKE_SWE_METADATA_FAIL", "1")],
        1,
    );

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "metadata_extraction_failed"
    );
    assert_eq!(
        results["tasks"][0]["health_impact"],
        "environment_unhealthy"
    );
    let events = events(&run_dir);
    assert_events_include(
        &events,
        &[
            "external_runner_started",
            "swe_bench_pro_metadata_extraction_started",
            "external_runner_setup_failed",
            "swe_bench_pro_setup_failed",
        ],
    );
    assert_event_message_contains(
        &events,
        "external_runner_setup_failed",
        "metadata extraction failed",
    );
    assert_event_message_contains(
        &events,
        "swe_bench_pro_setup_failed",
        "phase=metadata_extraction",
    );
    assert_event_absent(&events, "external_runner_workspace_started");
    assert_event_absent(&events, "external_runner_agent_started");
    assert_external_runtime_snapshots_exist(&results, &run_dir);
}

#[test]
fn swepro_002_workspace_failure_is_classified_and_observable() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "swe-gold",
        &[("HARNESSLAB_FAKE_SWE_DOCKER_FAIL", "1")],
        1,
    );

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "workspace_prep_failed");
    assert_eq!(
        results["tasks"][0]["health_impact"],
        "environment_unhealthy"
    );
    let events = events(&run_dir);
    assert_events_include(
        &events,
        &[
            "external_runner_started",
            "swe_bench_pro_metadata_extraction_started",
            "external_runner_workspace_started",
            "swe_bench_pro_workspace_prep_started",
            "external_runner_setup_failed",
            "swe_bench_pro_setup_failed",
        ],
    );
    assert_event_message_contains(
        &events,
        "external_runner_setup_failed",
        "workspace preparation failed",
    );
    assert_event_message_contains(
        &events,
        "swe_bench_pro_setup_failed",
        "phase=workspace_preparation",
    );
    assert_event_absent(&events, "external_runner_agent_started");
    assert_external_runtime_snapshots_exist(&results, &run_dir);
}

#[test]
fn swepro_003_diff_capture_failure_and_empty_patch_are_distinct() {
    let empty = run_fake_agent("true", 0);
    assert_eq!(empty.results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(empty.results["tasks"][0]["failure_code"], "no_valid_diff");
    assert_eq!(empty.results["tasks"][0]["patch"]["status"], "empty");
    assert_events_include(
        &empty.events,
        &[
            "external_runner_agent_started",
            "swe_bench_pro_agent_started",
            "external_runner_patch_started",
            "swe_bench_pro_patch_capture_started",
            "external_runner_patch_captured",
            "swe_bench_pro_patch_captured",
        ],
    );
    assert_event_message_contains(
        &empty.events,
        "swe_bench_pro_patch_captured",
        "phase=patch_capture status=empty",
    );
    assert_event_absent(&empty.events, "external_runner_evaluator_started");
    assert!(empty.git_diff_status_exists);

    let diff_failure = run_fake_agent("rm -rf .git", 1);
    assert_eq!(
        diff_failure.results["tasks"][0]["failure_class"],
        "execution"
    );
    assert_eq!(
        diff_failure.results["tasks"][0]["failure_code"],
        "patch_apply_failed"
    );
    assert_eq!(
        diff_failure.results["tasks"][0]["patch"]["status"],
        "apply_failed"
    );
    assert!(diff_failure.report_exists);
    assert!(diff_failure.git_diff_status_exists);
    assert!(!diff_failure.git_diff_stderr.is_empty());
    assert_event_message_contains(
        &diff_failure.events,
        "swe_bench_pro_patch_captured",
        "phase=patch_capture status=apply_failed",
    );
}

#[test]
fn swepro_004_evaluator_parse_corruption_is_not_patch_failure() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "swe-gold",
        &[("HARNESSLAB_FAKE_SWE_CORRUPT_EVAL_RESULTS", "1")],
        1,
    );

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "evaluator_error");
    assert_ne!(results["tasks"][0]["failure_code"], "patch_apply_failed");
    assert_ne!(results["tasks"][0]["failure_code"], "no_valid_diff");
    let events = events(&run_dir);
    assert_events_include(
        &events,
        &[
            "external_runner_patch_captured",
            "swe_bench_pro_patch_captured",
            "external_runner_evaluator_started",
            "swe_bench_pro_evaluator_started",
            "external_result_parse_failed",
            "swe_bench_pro_result_parse_failed",
        ],
    );
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let stderr = fs::read_to_string(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/verifier/stderr.log"),
    )
    .unwrap();
    assert!(stderr.contains("official eval_results unavailable"));
    assert_event_message_contains(
        &events,
        "external_result_parse_failed",
        "official eval_results unavailable",
    );
}

fn assert_external_runtime_snapshots_exist(results: &serde_json::Value, run_dir: &Path) {
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let attempt_dir = run_dir.join("tasks").join(task_id).join("attempts/1");
    assert!(attempt_dir.join("external-runtime.public.json").is_file());
    assert!(attempt_dir.join("external-runtime.private.json").is_file());
    let private: serde_json::Value = serde_json::from_slice(
        &fs::read(attempt_dir.join("external-runtime.private.json")).unwrap(),
    )
    .unwrap();
    assert!(
        private["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["phase"] == "metadata_extraction")
    );
}

struct FakeRun {
    results: serde_json::Value,
    events: Vec<serde_json::Value>,
    report_exists: bool,
    git_diff_status_exists: bool,
    git_diff_stderr: String,
}

fn run_fake_agent(command: &str, expected_code: i32) -> FakeRun {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), command);
    let root = swe_bench_root();
    let bin = fake_swe_tools();
    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "fake",
        &[],
        expected_code,
    );
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let attempt_dir = run_dir.join("tasks").join(task_id).join("attempts/1");
    FakeRun {
        results,
        events: events(&run_dir),
        report_exists: run_dir.join("report.html").is_file(),
        git_diff_status_exists: attempt_dir.join("git-diff.status.json").is_file(),
        git_diff_stderr: fs::read_to_string(attempt_dir.join("git-diff.stderr.log"))
            .unwrap_or_default(),
    }
}

fn assert_events_include(events: &[serde_json::Value], names: &[&str]) {
    let event_names = events
        .iter()
        .map(|record| record["event"].as_str().expect("event name"))
        .collect::<Vec<_>>();
    for name in names {
        assert!(event_names.contains(name), "missing {name}");
    }
}

fn assert_event_absent(events: &[serde_json::Value], name: &str) {
    assert!(
        !events
            .iter()
            .any(|record| record["event"].as_str() == Some(name)),
        "unexpected {name}"
    );
}

fn assert_event_message_contains(events: &[serde_json::Value], name: &str, needle: &str) {
    let messages = events
        .iter()
        .filter(|record| record["event"].as_str() == Some(name))
        .filter_map(|record| record["message"].as_str())
        .collect::<Vec<_>>();
    assert!(
        messages.iter().any(|message| message.contains(needle)),
        "missing message fragment {needle} for {name}: {messages:?}"
    );
}

fn events(run_dir: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(run_dir.join("events.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
