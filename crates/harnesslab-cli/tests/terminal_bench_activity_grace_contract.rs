#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;
use std::fs;
use std::time::{Duration, Instant};
use terminal_bench_support::*;

#[test]
fn int_035_terminal_bench_stale_docker_activity_becomes_no_progress() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx_and_docker_buildx(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
docker-buildx 7
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let started = Instant::now();
    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "2"),
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "12"),
        ],
        1,
    );
    let elapsed = started.elapsed();

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
    assert!(
        elapsed < Duration::from_secs(9),
        "stale activity should expire before the 12s hard timeout"
    );
    let health: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("run-health.json")).unwrap()).unwrap();
    assert_eq!(health["external_runner_no_progress"], 1);
    assert_eq!(health["execution_stalls"], 1);
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("activity_grace_sec=2"));
    assert!(events.contains("external_runner_activity"));
    assert!(events.contains("external_runner_no_progress"));
    assert!(events.contains("activity_grace_exhausted=true"));
    assert!(events.contains("current_activity=pid="));
    assert!(events.contains("last_activity=pid="));
    assert!(events.contains("last_progress=none"));
    assert!(events.contains("pattern=docker-buildx"));
}
