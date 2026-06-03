#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use std::fs;
use terminal_bench_support::*;

#[test]
fn int_046_terminal_bench_bridge_setup_failure_drops_stale_benchmark_warning() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(setup_failure_script());

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);
    let task = &results["tasks"][0];

    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "external_runner_setup_failed");
    assert_eq!(
        task["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|warning| *warning == "test_failed")
            .count(),
        0
    );
    let report = fs::read_to_string(run_dir.join("report.html")).unwrap();
    assert!(report.contains("execution/external_runner_setup_failed"));
    assert!(!report.contains("test_failed"));
}

fn setup_failure_script() -> &'static str {
    r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
log_dir="$out/$run/hello-world/hello-world.1-of-1.$run/agent-logs"
mkdir -p "$log_dir" "$out/$run"
printf 'harnesslab agent setup failed: exit_code=7\n' > "$log_dir/agent_setup_error.log"
printf '{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"unknown_agent_error"}]}' > "$out/$run/results.json"
exit 0
"#
}
