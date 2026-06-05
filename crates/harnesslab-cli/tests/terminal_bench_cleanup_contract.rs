#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;
use std::fs;
use terminal_bench_support::*;

#[test]
fn int_011_terminal_bench_cleans_actual_compose_labels_after_task() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let docker_log = home.path().join("docker.log");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'UVX %s\n' "$run" >> '{}'
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{{"task_id":"hello-world","is_resolved":true}}]}}' > "$out/$run/results.json"
exit 0
"#,
        docker_log.display(),
        marker.display()
    );
    let docker = format!(
        r#"printf 'DOCKER %s\n' "$*" >> '{}'
if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
if [ "$1 $2 $3" = "network ls --filter" ] && [ "$4" = "label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'n1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) exit 0 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'n1\n'; exit 0 ;;
  network\ rm\ n1) exit 0 ;;
esac
printf 'unexpected docker args: %s\n' "$*" >&2
exit 64
"#,
        docker_log.display(),
        marker.display(),
        marker.display(),
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    let docker_log = fs::read_to_string(docker_log).unwrap();
    let uvx_pos = docker_log.find("UVX ").unwrap();
    let first_cleanup_pos = docker_log
        .find("DOCKER ps -a --filter label=com.docker.compose.project")
        .unwrap();
    let post_cleanup_pos = docker_log.rfind("DOCKER network rm n1").unwrap();
    assert!(first_cleanup_pos < uvx_pos);
    assert!(post_cleanup_pos > uvx_pos);
    assert!(docker_log.contains("actual-prefix-"));
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("terminal_bench_cleanup"));
    assert!(!events.contains("projects=actual-prefix-"));
    assert!(events.contains("projects_count=1"));
    assert!(events.contains("containers_removed=1"));
    assert!(events.contains("networks_removed=1"));
    assert!(
        run_dir
            .join("terminal-bench-compose-projects.json")
            .is_file()
    );
}

#[test]
fn int_011_terminal_bench_pre_task_cleanup_failure_blocks_agent_launch() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("uvx-started");
    let uvx = format!(
        r#"touch '{}'
exit 0
"#,
        marker.display()
    );
    let docker = r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  echo "docker label scan denied" >&2
  exit 64
fi
exit 0
"#;
    let bin = fake_uvx_and_docker(&uvx, Some(docker));

    harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.path().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
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
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "terminal-bench cleanup pre_task failed",
        ));

    assert!(!marker.exists());
}

#[test]
fn int_040_terminal_bench_post_task_cleanup_failure_is_execution_failure() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{{"task_id":"hello-world","is_resolved":true}}]}}' > "$out/$run/results.json"
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, json) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(json["verdict"], "execution_failure");
    assert_eq!(results["summary"]["execution_failure"], 1);
    let task = &results["tasks"][0];
    assert_eq!(task["state"], "failure");
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "agent_cleanup_failed");
    assert_eq!(task["benchmark_score"], 0.0);
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("terminal-bench cleanup post_task warning"));
    assert!(events.contains("has_error=true"));
    assert!(!events.contains("metadata write failed"));
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/agent_cleanup_failed"));
}

#[test]
fn int_041_cleanup_failure_does_not_mask_no_progress_health() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{{"task_id":"hello-world","is_resolved":true}}]}}' > "$out/$run/results.json"
echo "official result written"
sleep 20
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "6")],
        1,
    );

    let task = &results["tasks"][0];
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "external_runner_no_progress");
    assert!(
        task["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "agent_cleanup_failed")
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["external_runner_no_progress"], 1);
    assert_eq!(health["execution_stalls"], 1);
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/external_runner_no_progress"));
    assert!(report.contains("agent_cleanup_failed"));
}

#[test]
fn int_042_cleanup_failure_overrides_benchmark_failure_with_warning() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{{"task_id":"hello-world","is_resolved":false,"failure_mode":"test_failed"}}]}}' > "$out/$run/results.json"
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, json) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(json["verdict"], "execution_failure");
    assert_eq!(results["summary"]["benchmark_failure"], 0);
    assert_eq!(results["summary"]["execution_failure"], 1);
    let task = &results["tasks"][0];
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "agent_cleanup_failed");
    assert!(
        task["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "test_failed")
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["status"], "ok");
    assert_eq!(health["execution_stalls"], 0);
    assert!(
        run_dir
            .join("terminal-bench-compose-projects.json")
            .is_file()
    );
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/agent_cleanup_failed"));
    assert!(report.contains("test_failed"));
}

#[test]
fn int_043_cleanup_failure_does_not_mask_runner_timeout_health() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{{"task_id":"hello-world","is_resolved":true}}]}}' > "$out/$run/results.json"
echo "official result written"
sleep 8
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "4"),
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "off"),
        ],
        1,
    );

    let task = &results["tasks"][0];
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "external_runner_timeout");
    assert!(
        task["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "agent_cleanup_failed")
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["external_runner_timeouts"], 1);
    assert_eq!(health["execution_stalls"], 1);
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/external_runner_timeout"));
    assert!(report.contains("agent_cleanup_failed"));
}
