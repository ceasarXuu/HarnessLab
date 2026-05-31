use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

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

fn run_terminal(
    home: &Path,
    root: &Path,
    bin: &Path,
    expected_code: i32,
) -> (serde_json::Value, std::path::PathBuf, serde_json::Value) {
    let output = harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root)
        .env("PATH", path_with(bin))
        .args([
            "--home",
            home.to_str().unwrap(),
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
        .code(expected_code)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = std::path::PathBuf::from(json["run_dir"].as_str().unwrap());
    let results = serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    (results, run_dir, json)
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, agent: &str, model: Option<&str>, import_path: Option<&str>) {
    let mut labels = format!("terminal_bench_agent = \"{agent}\"\n");
    if let Some(model) = model {
        labels.push_str(&format!("terminal_bench_model = \"{model}\"\n"));
    }
    if let Some(import_path) = import_path {
        labels.push_str(&format!(
            "terminal_bench_agent_import_path = \"{import_path}\"\n"
        ));
    }
    write_agent_with_labels(home, &labels);
}

fn write_agent_with_labels(home: &Path, labels: &str) {
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "true"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 5

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[usage]
parser = "none"

[labels]
{labels}"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn success_when_model_is(expected: &str) -> &'static str {
    match expected {
        "gpt-5" => {
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
"#
        }
        _ => unreachable!("test only supports gpt-5"),
    }
}

fn terminal_bench_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    fs::create_dir_all(&task_dir).unwrap();
    fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();
    root
}

fn fake_uvx(body: &str) -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    let uvx = bin.path().join("uvx");
    fs::write(&uvx, format!("#!/bin/sh\n{body}")).unwrap();
    let mut permissions = fs::metadata(&uvx).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(&uvx, permissions).unwrap();
    }
    bin
}

fn path_with(bin: &Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin.display())
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
