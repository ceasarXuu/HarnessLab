#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;
use std::fs;
use terminal_bench_support::*;

#[test]
fn int_022_terminal_bench_official_agent_timeout_is_benchmark_verdict() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"agent_timeout"}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 2);

    assert_eq!(results["summary"]["benchmark_failure"], 1);
    assert_eq!(results["summary"]["execution_failure"], 0);
    assert_eq!(results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(results["tasks"][0]["failure_code"], "agent_timeout");
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["status"], "ok");
    assert_eq!(health["agent_timeouts"], 0);
    assert_eq!(health["non_timeout_completed"], 1);
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("benchmark/agent_timeout"));
    assert!(!report.contains("benchmark/test_failed"));
    let verifier =
        fs::read_to_string(run_dir.join("tasks/hello-world/attempts/1/verifier/stdout.log"))
            .unwrap();
    assert!(verifier.contains("official_results_path="));
    assert!(verifier.contains("\"failure_mode\": \"agent_timeout\""));
}

#[test]
fn int_023_terminal_bench_repeated_official_agent_timeouts_do_not_abort_run() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&[
        "timeout-a",
        "timeout-b",
        "timeout-c",
        "timeout-d",
        "timeout-e",
        "timeout-f",
    ]);
    let marker = home.path().join("uvx-count.txt");
    let bin = fake_uvx(&format!(
        r#"out=""; run=""; task=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --task-id) task="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf x >> '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{{"task_id":"%s","is_resolved":false,"failure_mode":"agent_timeout"}}]}}' "$task" > "$out/$run/results.json"
exit 0
"#,
        marker.display()
    ));

    let (results, run_dir, _) = run_terminal_with_split_and_extra_args(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &["--concurrency", "1"],
        2,
    );

    assert_eq!(fs::read_to_string(marker).unwrap().matches('x').count(), 6);
    assert_eq!(results["summary"]["total_tasks"], 6);
    assert_eq!(results["summary"]["benchmark_failure"], 6);
    assert_eq!(results["summary"]["execution_failure"], 0);
    assert_eq!(results["summary"]["interrupted"], 0);
    let tasks = results["tasks"].as_array().unwrap();
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task["failure_code"] == "agent_timeout")
            .count(),
        6
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["status"], "ok");
    assert_eq!(health["agent_timeouts"], 0);
    assert_eq!(health["non_timeout_completed"], 6);
    assert_eq!(health["completed"], 6);
}

#[test]
fn int_024_terminal_bench_success_with_agent_timeout_gets_warning() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true,"failure_mode":"agent_timeout"}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["failure_class"], "none");
    assert_eq!(results["tasks"][0]["failure_code"], serde_json::Value::Null);
    assert!(
        results["tasks"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "agent_timeout")
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["agent_timeouts"], 0);
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("<th>Warnings</th>"));
    assert!(report.contains("agent_timeout"));
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("task_warning"));
    assert!(events.contains("AgentTimeout"));
}

#[test]
fn int_025_terminal_bench_default_no_output_watchdog_is_enabled() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
sleep 2
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("external_runner_configured"));
    assert!(events.contains("process_timeout_sec=9000 no_output_timeout_sec=3720"));
    assert!(!events.contains("no_output_timeout_sec=disabled"));
    assert!(!events.contains("external_runner_no_progress"));
}

#[test]
fn int_026_terminal_bench_no_progress_overrides_official_result() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"agent_timeout"}]}' > "$out/$run/results.json"
echo "official result written"
sleep 20
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "3")],
        1,
    );

    assert_eq!(results["summary"]["execution_failure"], 1);
    assert_eq!(results["summary"]["benchmark_failure"], 0);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_no_progress"
    );
    assert!(
        results["tasks"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "agent_timeout")
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["external_runner_no_progress"], 1);
    assert_eq!(health["execution_stalls"], 1);
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("external_runner_no_progress"));
    assert!(events.contains("task_warning"));
    assert!(events.contains("AgentTimeout"));
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/external_runner_no_progress"));
    assert!(report.contains("agent_timeout"));
}

#[test]
fn int_027_terminal_bench_repeated_no_progress_aborts_run() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&[
        "stall-a", "stall-b", "stall-c", "stall-d", "stall-e", "stall-f",
    ]);
    let bin = fake_uvx(
        r#"echo "runner started"
sleep 8
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &["--concurrency", "1"],
        &[("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "3")],
        130,
    );

    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(results["summary"]["execution_failure"], 6);
    assert_eq!(results["summary"]["interrupted"], 1);
    assert_eq!(health["status"], "invalid");
    assert_eq!(health["external_runner_no_progress"], 5);
    assert_eq!(health["execution_stalls"], 5);
    assert_eq!(health["abort_attempt"], 1);
    assert!(
        health["reason"]
            .as_str()
            .unwrap()
            .contains("execution stall threshold")
    );
    assert_eq!(
        results["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|task| task["failure_code"] == "run_health_aborted")
            .count(),
        1
    );
    assert_eq!(
        results["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|task| task["failure_code"] == "external_runner_no_progress")
            .count(),
        5
    );
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("run_health_aborted"));
    assert!(events.contains("task_interrupted"));
}

#[test]
fn int_028_terminal_bench_hard_timeout_overrides_official_result() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"agent_timeout"}]}' > "$out/$run/results.json"
echo "official result written"
sleep 8
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "8"),
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "off"),
        ],
        1,
    );

    assert_eq!(results["summary"]["execution_failure"], 1);
    assert_eq!(results["summary"]["benchmark_failure"], 0);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_timeout"
    );
    assert!(
        results["tasks"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "agent_timeout")
    );
    assert_eq!(
        results["tasks"][0]["agent"]["termination_reason"],
        "timeout"
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["agent_timeouts"], 0);
    assert_eq!(health["external_runner_timeouts"], 1);
    assert_eq!(health["execution_stalls"], 1);
    assert_eq!(
        health["reason"],
        "external runner hard timeout; benchmark runner budget or setup is unhealthy"
    );
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("no_output_timeout_sec=disabled"));
    assert!(events.contains("external_runner_timeout"));
    assert!(!events.contains("external_runner_no_progress"));
}

#[test]
fn int_021_terminal_bench_silent_official_runner_is_no_progress() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"for tick in 1 2 3 4 5 6 7 8 9 10; do
  echo "official runner started $tick"
  sleep 0.2
done
sleep 8
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "5")],
        1,
    );

    assert_eq!(results["summary"]["execution_failure"], 1);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_no_progress"
    );
    assert_eq!(
        results["tasks"][0]["agent"]["termination_reason"],
        "no_progress"
    );
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("external_runner_no_progress"));
    let stdout =
        fs::read_to_string(run_dir.join("tasks/hello-world/attempts/1/agent/stdout.log")).unwrap();
    assert!(stdout.contains("official runner started"));
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/external_runner_no_progress"));
}
