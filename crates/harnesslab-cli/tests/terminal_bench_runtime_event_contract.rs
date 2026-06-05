#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use terminal_bench_support::*;

#[test]
fn adapt_runtime_005_terminal_bench_event_taxonomy_is_stable() {
    let (success_events, success_command, argv_log) = run_command_preservation_case();
    assert_event_message(
        &success_events,
        "external_runner_configured",
        &[
            "process_timeout_sec=",
            "no_output_timeout_sec=",
            "docker_platform=linux/amd64",
            "progress_paths=official/terminal-bench/<run-id>/run.log",
            "activity_patterns=docker compose,docker-compose,docker build,docker buildx,docker-buildx,docker pull",
            "official_result_path=",
            "command_snapshot_path=",
        ],
    );
    assert_event_message(
        &success_events,
        "external_runner_started",
        &[
            "dataset=",
            "runtime_dataset=",
            "official_run_id=",
            "output=",
        ],
    );
    assert_event_message(
        &success_events,
        "terminal_bench_cleanup",
        &["pre_task", "token=", "removed"],
    );
    assert_event_message(&success_events, "task_warning", &["AgentTimeout"]);
    assert_command_snapshot(&success_command);
    assert_argv_log(&argv_log, "--agent", "oracle");

    let (import_command, import_argv) = run_import_path_command_case();
    assert!(import_command.contains("--agent-import-path"));
    assert!(import_command.contains("bench_agents.fake:Agent"));
    assert_argv_log(
        &import_argv,
        "--agent-import-path",
        "bench_agents.fake:Agent",
    );
    assert_argv_flag_value(&import_argv, "--global-agent-timeout-sec", "3630");

    assert_event_message(
        &run_qemu_dataset_case(),
        "terminal_bench_dataset_prepared",
        &[
            "compatibility=amd64_qemu_make_j1",
            "source_dataset=",
            "runtime_dataset=",
        ],
    );
    assert_event_message(
        &run_setup_failure_case(),
        "external_runner_setup_failed",
        &["terminal-bench", "runtime dataset preparation failed"],
    );
    assert_event_message(
        &run_timeout_case(),
        "external_runner_timeout",
        &["hard timeout"],
    );
    let no_progress_events = run_no_progress_activity_case();
    assert_event_message(
        &no_progress_events,
        "external_runner_activity",
        &["pattern=docker-buildx"],
    );
    assert_event_message(
        &no_progress_events,
        "external_runner_no_progress",
        &["activity_grace_exhausted=true"],
    );
    assert_event_message(
        &run_parse_failure_case(),
        "external_result_parse_failed",
        &["terminal-bench results parse failed"],
    );
}

fn run_command_preservation_case() -> (Vec<serde_json::Value>, String, String) {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let argv_log_dir = tempfile::tempdir().unwrap();
    let argv_log_path = argv_log_dir.path().join("argv.log");
    let bin = fake_uvx(command_recording_success_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_TB_ARGV_LOG", argv_log_path.to_str().unwrap())],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    let argv = fs::read_to_string(argv_log_path).unwrap();
    let command = read_agent_command_snapshot(&run_dir);
    (read_events(&run_dir), command, argv)
}

fn run_import_path_command_case() -> (String, String) {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, Some("bench_agents.fake:Agent"));
    let root = terminal_bench_root();
    let argv_log_dir = tempfile::tempdir().unwrap();
    let argv_log_path = argv_log_dir.path().join("argv.log");
    let bin = fake_uvx(command_recording_success_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_TB_ARGV_LOG", argv_log_path.to_str().unwrap())],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    (
        read_agent_command_snapshot(&run_dir),
        fs::read_to_string(argv_log_path).unwrap(),
    )
}

fn run_qemu_dataset_case() -> Vec<serde_json::Value> {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&["build-tcc-qemu"]);
    fs::write(
        root.path()
            .join("terminal-bench/terminal-bench-core-0.1.1/build-tcc-qemu/Dockerfile"),
        canonical_qemu_dockerfile(),
    )
    .unwrap();
    let bin = fake_uvx(command_recording_success_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &[],
        &[("HARNESSLAB_TERMINAL_BENCH_DOCKER_PLATFORM", "linux/amd64")],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    read_events(&run_dir)
}

fn run_setup_failure_case() -> Vec<serde_json::Value> {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root_with_tasks(&["build-tcc-qemu"]);
    let bin = fake_uvx(command_recording_success_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "full",
        &[],
        &[("HARNESSLAB_TERMINAL_BENCH_DOCKER_PLATFORM", "linux/amd64")],
        1,
    );

    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_setup_failed"
    );
    read_events(&run_dir)
}

fn run_timeout_case() -> Vec<serde_json::Value> {
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
printf '{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"agent_timeout"}]}' > "$out/$run/results.json"
sleep 20
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "2"),
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "off"),
        ],
        1,
    );

    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_timeout"
    );
    read_events(&run_dir)
}

fn run_no_progress_activity_case() -> Vec<serde_json::Value> {
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
docker-buildx 5
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
"#,
    );

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[
            ("HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC", "2"),
            ("HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC", "8"),
        ],
        1,
    );

    assert_eq!(
        results["tasks"][0]["failure_code"],
        "external_runner_no_progress"
    );
    read_events(&run_dir)
}

fn run_parse_failure_case() -> Vec<serde_json::Value> {
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
printf '{not-json' > "$out/$run/results.json"
exit 0
"#,
    );

    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    read_events(&run_dir)
}

fn command_recording_success_script() -> &'static str {
    r#"if [ -n "${HARNESSLAB_TB_ARGV_LOG:-}" ]; then
  printf '%s\n' "$@" > "$HARNESSLAB_TB_ARGV_LOG"
fi
out=""; run=""; task=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    --task-id) task="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"%s","is_resolved":true,"failure_mode":"agent_timeout"}]}' "$task" > "$out/$run/results.json"
exit 0
"#
}

fn assert_command_snapshot(command: &str) {
    for expected in [
        "uvx --from terminal-bench tb run",
        "--dataset-path",
        "--global-agent-timeout-sec",
        "--global-test-timeout-sec",
        "--output-path",
        "--run-id",
        "--no-upload-results",
        "--agent",
        "oracle",
    ] {
        assert!(
            command.contains(expected),
            "command snapshot missing {expected}: {command}"
        );
    }
}

fn read_agent_command_snapshot(run_dir: &Path) -> String {
    let command_path = find_agent_command_snapshot(run_dir).unwrap_or_else(|| {
        panic!(
            "missing agent command snapshot under {}; command candidates={:?}",
            run_dir.display(),
            find_command_snapshots(run_dir)
        )
    });
    fs::read_to_string(command_path).unwrap()
}

fn assert_argv_log(argv: &str, branch_flag: &str, branch_value: &str) {
    let args = argv.lines().collect::<Vec<_>>();
    let arg_set = args.iter().copied().collect::<BTreeSet<_>>();
    for expected in [
        "--from",
        "terminal-bench",
        "tb",
        "run",
        "--dataset-path",
        "--task-id",
        "--global-agent-timeout-sec",
        "--global-test-timeout-sec",
        "--output-path",
        "--run-id",
        "--no-upload-results",
        branch_flag,
        branch_value,
    ] {
        assert!(
            arg_set.contains(expected),
            "argv missing {expected}: {argv}"
        );
    }
}

fn assert_argv_flag_value(argv: &str, flag: &str, expected: &str) {
    let args = argv.lines().collect::<Vec<_>>();
    let value = args
        .windows(2)
        .find_map(|pair| (pair[0] == flag).then_some(pair[1]))
        .unwrap_or_else(|| panic!("argv missing {flag}: {argv}"));
    assert_eq!(value, expected, "argv flag {flag} mismatch: {argv}");
}

fn find_agent_command_snapshot(root: &Path) -> Option<std::path::PathBuf> {
    for entry in fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "command.txt")
            && path
                .parent()
                .and_then(Path::file_name)
                .is_some_and(|name| name == "agent")
        {
            return Some(path);
        }
        if path.is_dir()
            && let Some(found) = find_agent_command_snapshot(&path)
        {
            return Some(found);
        }
    }
    None
}

fn find_command_snapshots(root: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    collect_command_snapshots(root, &mut paths);
    paths
}

fn collect_command_snapshots(root: &Path, paths: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == "command.txt") {
            paths.push(path.clone());
        }
        if path.is_dir() {
            collect_command_snapshots(&path, paths);
        }
    }
}

fn read_events(run_dir: &Path) -> Vec<serde_json::Value> {
    fs::read_to_string(run_dir.join("events.jsonl"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn assert_event_message(events: &[serde_json::Value], name: &str, fragments: &[&str]) {
    let event = events
        .iter()
        .find(|event| event["event"] == name)
        .unwrap_or_else(|| panic!("missing event {name}; events={events:#?}"));
    assert_eq!(event["schema_version"], 1);
    assert!(
        event["run_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(
        event["task_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty())
    );
    let message = event["message"].as_str().unwrap_or_default();
    for fragment in fragments {
        assert!(
            message.contains(fragment),
            "event {name} missing {fragment}; message={message}"
        );
    }
}

fn canonical_qemu_dockerfile() -> &'static str {
    "FROM ubuntu\nRUN apt-get install -y build-essential libncurses-dev bison flex libssl-dev libelf-dev qemu-system bc cpio wget expect\nRUN cd linux-6.9 && make defconfig\nRUN cd linux-6.9 && make olddefconfig\nRUN cd linux-6.9 && make -j$(nproc)\n"
}
