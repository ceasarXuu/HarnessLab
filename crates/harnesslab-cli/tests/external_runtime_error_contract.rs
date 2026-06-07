#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use std::fs;
use terminal_bench_support::*;

#[test]
fn adapt_runtime_006_adapter_internal_error_is_classified_and_snapshotted() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(
        r#"out=""; run=""; task=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --task-id) task="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir cleanup-report.json
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"results":[{"task_id":"%s","is_resolved":true}]}' "$task" > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);

    let task = &results["tasks"][0];
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "agent_cleanup_failed");
    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    assert!(attempt_dir.join("result.json").is_file());
    assert!(attempt_dir.join("external-runtime.public.json").is_file());
    assert!(attempt_dir.join("external-runtime.private.json").is_file());
    assert!(attempt_dir.join("internal-error.public.json").is_file());
    assert!(attempt_dir.join("internal-error.private.json").is_file());
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("\"event\":\"external_runner_internal_error\""));
    assert!(events.contains("adapter_id=terminal-bench-runtime"));
    assert!(events.contains("adapter_phase=execute"));
    assert!(events.contains("adapter_subphase=post_execution_cleanup"));
    assert!(events.contains("failure_code=agent_cleanup_failed"));
    assert!(!events.contains("cleanup-report.json"));
    let public: serde_json::Value = serde_json::from_slice(
        &fs::read(attempt_dir.join("external-runtime.public.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(public["benchmark"], "terminal-bench");
    assert_eq!(
        public["runtime_diagnostics"]["stage"], "pre_execution",
        "late internal errors should not overwrite the adapter-owned runtime snapshot"
    );
    assert!(
        public["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["phase"] == "official_runner")
    );
    let internal_error: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_dir.join("internal-error.public.json")).unwrap())
            .unwrap();
    assert_eq!(
        internal_error["internal_error"]["adapter_id"],
        "terminal-bench-runtime"
    );
    assert_eq!(internal_error["internal_error"]["phase"], "execute");
    assert_eq!(
        internal_error["internal_error"]["subphase"],
        "post_execution_cleanup"
    );
    assert_eq!(
        internal_error["internal_error"]["failure_code"],
        "agent_cleanup_failed"
    );
    assert!(
        !fs::read_to_string(attempt_dir.join("internal-error.public.json"))
            .unwrap()
            .contains("cleanup-report.json")
    );
}
