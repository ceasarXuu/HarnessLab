#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use terminal_bench_support::*;

#[test]
fn int_044_terminal_bench_runtime_exports_amd64_platform_by_default() {
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
[ "${DOCKER_DEFAULT_PLATFORM:-}" = "linux/amd64" ] || {
  echo "missing docker platform: ${DOCKER_DEFAULT_PLATFORM:-}" >&2
  exit 64
}
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("docker_platform=linux/amd64"));
}
