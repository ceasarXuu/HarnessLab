#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use assert_cmd::Command;
use std::fs;
use terminal_bench_support::*;

#[test]
fn adapt_protocol_006_replay_blocks_when_protocol_authority_incomplete() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(success_script());

    let (_results, run_dir, _json) = run_terminal(home.path(), root.path(), bin.path(), 0);

    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let private_path = attempt_dir.join("external-runtime.private.json");
    let public_path = attempt_dir.join("external-runtime.public.json");

    let mut private: serde_json::Value =
        serde_json::from_slice(&fs::read(&private_path).unwrap()).unwrap();
    let mut public: serde_json::Value =
        serde_json::from_slice(&fs::read(&public_path).unwrap()).unwrap();

    private
        .as_object_mut()
        .unwrap()
        .remove("protocol_authority");
    public.as_object_mut().unwrap().remove("protocol_authority");

    fs::write(&private_path, serde_json::to_vec_pretty(&private).unwrap()).unwrap();
    fs::write(&public_path, serde_json::to_vec_pretty(&public).unwrap()).unwrap();

    let run_count = fs::read_dir(home.path().join("runs")).unwrap().count();
    harnesslab()
        .env("PATH", path_with(bin.path()))
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("protocol_authority_incomplete"));
    assert_eq!(
        fs::read_dir(home.path().join("runs")).unwrap().count(),
        run_count
    );
}

#[test]
fn adapt_protocol_006_replay_blocks_when_protocol_authority_inconsistent() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf ok > result.txt", None, None);
    let root = terminal_bench_root();
    let bin = fake_uvx(success_script());

    let (_results, run_dir, _json) = run_terminal(home.path(), root.path(), bin.path(), 0);

    let benchmark_snapshot_path = run_dir.join("benchmark.snapshot.json");
    let mut benchmark_snapshot: serde_json::Value =
        serde_json::from_slice(&fs::read(&benchmark_snapshot_path).unwrap()).unwrap();

    let tasks = benchmark_snapshot["tasks"].as_array_mut().unwrap();
    for task in tasks {
        task["runtime_binding"] = serde_json::Value::Null;
    }
    let task_runtime_snapshots = benchmark_snapshot["task_runtime_snapshots"]
        .as_array_mut()
        .unwrap();
    for snapshot in task_runtime_snapshots {
        snapshot["runtime_binding"] = serde_json::Value::Null;
    }
    fs::write(
        &benchmark_snapshot_path,
        serde_json::to_vec_pretty(&benchmark_snapshot).unwrap(),
    )
    .unwrap();

    let task_runtime_snapshot_path = run_dir.join("tasks/hello-world/task-runtime.snapshot.json");
    let mut task_runtime_snapshot: serde_json::Value =
        serde_json::from_slice(&fs::read(&task_runtime_snapshot_path).unwrap()).unwrap();
    task_runtime_snapshot["runtime_binding"] = serde_json::Value::Null;
    fs::write(
        &task_runtime_snapshot_path,
        serde_json::to_vec_pretty(&task_runtime_snapshot).unwrap(),
    )
    .unwrap();

    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let private_path = attempt_dir.join("external-runtime.private.json");
    let public_path = attempt_dir.join("external-runtime.public.json");
    let mut private: serde_json::Value =
        serde_json::from_slice(&fs::read(&private_path).unwrap()).unwrap();
    let mut public: serde_json::Value =
        serde_json::from_slice(&fs::read(&public_path).unwrap()).unwrap();

    private["protocol_authority"] = serde_json::json!({
        "benchmark_id": "terminal-bench",
        "adapter_id": "harnesslab.terminal-bench.runtime",
        "protocol_version": "v1",
        "adapter_version": "terminal-bench-runtime.v1",
        "selected_mode": "official-runner",
        "capabilities": ["official.runner", "docker.orchestration"],
        "stability": "experimental",
    });
    public["protocol_authority"] = private["protocol_authority"].clone();

    fs::write(&private_path, serde_json::to_vec_pretty(&private).unwrap()).unwrap();
    fs::write(&public_path, serde_json::to_vec_pretty(&public).unwrap()).unwrap();

    let run_count = fs::read_dir(home.path().join("runs")).unwrap().count();
    harnesslab()
        .env("PATH", path_with(bin.path()))
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("protocol_authority_inconsistent"));
    assert_eq!(
        fs::read_dir(home.path().join("runs")).unwrap().count(),
        run_count
    );
}

fn success_script() -> &'static str {
    r#"out=""; run=""; task=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --task-id) task="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"%s","is_resolved":true}]}' "$task" > "$out/$run/results.json"
exit 0
"#
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
