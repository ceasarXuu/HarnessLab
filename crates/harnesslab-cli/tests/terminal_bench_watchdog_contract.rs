#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;
use std::fs;
use terminal_bench_support::*;

#[test]
fn int_031_terminal_bench_progress_deferral_still_hard_times_out() {
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
echo "official result written"
for tick in 1 2 3 4 5 6 7 8 9 10; do
  printf "progress %s\n" "$tick" >> "$out/$run/run.log"
  sleep 1
done
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
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "5"),
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "2"),
        ],
        1,
    );

    assert_eq!(results["summary"]["execution_failure"], 1);
    assert_eq!(results["summary"]["benchmark_failure"], 0);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "agent_timeout");
    assert_eq!(
        results["tasks"][0]["agent"]["termination_reason"],
        "timeout"
    );
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("progress file path="));
    assert!(events.contains("external_runner_timeout"));
    assert!(!events.contains("external_runner_no_progress"));
}
