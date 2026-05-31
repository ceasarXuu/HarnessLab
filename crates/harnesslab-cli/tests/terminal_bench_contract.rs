#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;
use predicates::prelude::*;
use std::fs;
use terminal_bench_support::*;

#[test]
fn int_011_terminal_bench_smoke_without_data_reports_readiness_blocker() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = tempfile::tempdir().unwrap();

    harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
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
        .code(3)
        .stderr(
            predicate::str::contains("terminal-bench/smoke")
                .and(predicate::str::contains("data_state=not_downloaded")),
        );
}

#[test]
fn int_011_terminal_bench_zero_exit_without_results_stays_task_failure() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx("exit 0\n");

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "evaluator_error");
    assert!(run_dir.join("report.html").is_file());
}

#[test]
fn int_011_terminal_bench_uses_benchmark_timeout_override() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; agent_timeout=""; test_timeout=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --global-agent-timeout-sec) agent_timeout="$2"; shift 2 ;;
    --global-test-timeout-sec) test_timeout="$2"; shift 2 ;;
    *) shift ;;
  esac
done
if [ "$agent_timeout" != "3600" ] || [ "$test_timeout" != "3600" ]; then
  echo "bad timeouts agent=$agent_timeout test=$test_timeout" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    let spec: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run.json")).unwrap()).unwrap();
    assert_eq!(spec["execution"]["timeout_sec"], 3600);
}

#[test]
fn int_011_terminal_bench_run_timeout_override_wins_over_benchmark_default() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; agent_timeout=""; test_timeout=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --global-agent-timeout-sec) agent_timeout="$2"; shift 2 ;;
    --global-test-timeout-sec) test_timeout="$2"; shift 2 ;;
    *) shift ;;
  esac
done
if [ "$agent_timeout" != "123" ] || [ "$test_timeout" != "123" ]; then
  echo "bad timeouts agent=$agent_timeout test=$test_timeout" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal_with_extra_args(
        home.path(),
        root.path(),
        bin.path(),
        &["--timeout-sec", "123"],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    let spec: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run.json")).unwrap()).unwrap();
    assert_eq!(spec["execution"]["timeout_sec"], 123);
}

#[test]
fn int_011_terminal_bench_nonzero_with_results_uses_benchmark_result() {
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
printf '{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"total_input_tokens":3,"total_output_tokens":4}]}' > "$out/$run/results.json"
exit 1
"#,
    );

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 2);

    assert_eq!(results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(results["tasks"][0]["failure_code"], "test_failed");
    assert_eq!(results["tasks"][0]["usage"]["total_tokens"], 7);
}

#[test]
fn int_011_terminal_bench_docker_network_exhaustion_aborts_remaining_tasks() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&["first-task", "second-task"]);
    let bin = fake_uvx(
        r#"echo "Error response from daemon: all predefined address pools have been fully subnetted" >&2
exit 1
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_and_extra_args(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &["--concurrency", "1"],
        130,
    );

    assert_eq!(results["summary"]["total_tasks"], 2);
    assert_eq!(results["summary"]["execution_failure"], 2);
    assert_eq!(results["summary"]["interrupted"], 1);
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "docker_network_pool_exhausted"
    );
    assert_eq!(results["tasks"][1]["state"], "interrupted");
    assert_eq!(results["tasks"][1]["failure_code"], "run_health_aborted");
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["status"], "invalid");
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("Run health:"));
    assert!(report.contains("docker network pool exhausted"));
    assert!(report.contains("interrupted 1"));
}

#[test]
fn int_011_terminal_bench_concurrent_abort_drains_active_chunk_only() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&["a", "b", "c", "d", "e"]);
    let marker = home.path().join("uvx-count.txt");
    let bin = fake_uvx(&format!(
        r#"printf run >> '{}'
echo "Error response from daemon: all predefined address pools have been fully subnetted" >&2
exit 1
"#,
        marker.display()
    ));

    let (results, _, _) = run_terminal_with_split_and_extra_args(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &["--concurrency", "2"],
        130,
    );

    assert_eq!(
        fs::read_to_string(marker).unwrap().matches("run").count(),
        2
    );
    assert_eq!(results["summary"]["total_tasks"], 5);
    assert_eq!(results["summary"]["interrupted"], 3);
    let tasks = results["tasks"].as_array().unwrap();
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task["failure_code"] == "docker_network_pool_exhausted")
            .count(),
        2
    );
    assert_eq!(
        tasks
            .iter()
            .filter(|task| task["failure_code"] == "run_health_aborted")
            .count(),
        3
    );
}

#[test]
fn int_011_terminal_bench_infra_log_overrides_normal_looking_results() {
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
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
echo "Error response from daemon: all predefined address pools have been fully subnetted" >&2
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(results["summary"]["success"], 0);
    assert_eq!(results["summary"]["total_score"], 0.0);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(
        results["tasks"][0]["failure_code"],
        "docker_network_pool_exhausted"
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["status"], "invalid");
}

#[test]
fn int_011_terminal_bench_outer_timeout_allows_setup_overhead() {
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

    let (results, _, _) = run_terminal_with_extra_args(
        home.path(),
        root.path(),
        bin.path(),
        &["--timeout-sec", "1"],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
}

#[test]
fn int_011_terminal_bench_builtin_agent_receives_model_label() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "codex", Some("gpt-5"), None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; model=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --model) model="$2"; shift 2 ;;
    *) shift ;;
  esac
done
if [ "$model" != "gpt-5" ]; then
  echo "missing model: $model" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["benchmark_score"], 1.0);
}

#[test]
fn int_011_terminal_bench_builtin_agent_accepts_model_fallback_label() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels(
        home.path(),
        r#"terminal_bench_agent = "codex"
model = "gpt-5"
"#,
    );
    let root = terminal_bench_root();
    let bin = fake_uvx(success_when_model_is("gpt-5"));

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
}

#[test]
fn int_011_terminal_bench_labeled_codex_requires_model() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels(
        home.path(),
        r#"terminal_bench_agent = "codex"
"#,
    );
    let root = terminal_bench_root();

    harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
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
        .code(3)
        .stderr(predicate::str::contains(
            "must set label terminal_bench_model or model",
        ));
}

#[test]
fn int_011_terminal_bench_non_model_builtin_does_not_receive_model_label() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels(
        home.path(),
        r#"terminal_bench_agent = "claude-code"
model = "ignored"
"#,
    );
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; model_seen=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --model) model_seen=1; shift 2 ;;
    *) shift ;;
  esac
done
if [ "$model_seen" != "0" ]; then
  echo "unexpected model flag" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
}

#[test]
fn int_011_terminal_bench_import_path_is_forwarded_without_model() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "ignored", None, Some("pkg.agent:Agent"));
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; import_path=""; model_seen=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --agent-import-path) import_path="$2"; shift 2 ;;
    --model) model_seen=1; shift 2 ;;
    *) shift ;;
  esac
done
if [ "$import_path" != "pkg.agent:Agent" ] || [ "$model_seen" != "0" ]; then
  echo "bad import_path=$import_path model_seen=$model_seen" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
}

#[test]
fn int_011_terminal_bench_import_agent_receives_registered_command_env() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels(
        home.path(),
        r#"terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"
terminal_bench_agent_pythonpath = "/opt/harnesslab/python"
"#,
    );
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
case ":${PYTHONPATH:-}:" in
  *":/opt/harnesslab/python:"*) ;;
  *) echo "missing pythonpath: ${PYTHONPATH:-}" >&2; exit 64 ;;
esac
if [ "${HARNESSLAB_AGENT_COMMAND:-}" != "true" ]; then
  echo "missing command env: ${HARNESSLAB_AGENT_COMMAND:-}" >&2
  exit 64
fi
if [ "${HARNESSLAB_AGENT_INPUT_MODE:-}" != "stdin" ]; then
  echo "missing input mode env: ${HARNESSLAB_AGENT_INPUT_MODE:-}" >&2
  exit 64
fi
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, _, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
}
