use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn int_011_terminal_bench_smoke_without_data_reports_readiness_blocker() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "printf terminal-bench-smoke > result.txt");
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
    write_terminal_agent(home.path());
    let root = terminal_bench_root();
    let bin = fake_uvx("exit 0\n");

    let output = harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env("PATH", path_with(bin.path()))
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
        .code(1)
        .stdout(predicate::str::contains("\"status\":\"failure\""))
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = Path::new(json["run_dir"].as_str().unwrap());
    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "evaluator_error");
    assert!(run_dir.join("report.html").is_file());
}

#[test]
fn int_011_terminal_bench_nonzero_with_results_uses_benchmark_result() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_terminal_agent(home.path());
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

    let output = harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env("PATH", path_with(bin.path()))
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
        .code(2)
        .stdout(predicate::str::contains("\"status\":\"failure\""))
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let results: serde_json::Value = serde_json::from_slice(
        &fs::read(Path::new(json["run_dir"].as_str().unwrap()).join("results.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(results["tasks"][0]["failure_code"], "test_failed");
    assert_eq!(results["tasks"][0]["usage"]["total_tokens"], 7);
}

#[test]
fn int_011_swe_bench_pro_smoke_runs_external_evaluator_contract() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "swe-gold", &[], 0);
    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["benchmark_score"], 1.0);
    assert_eq!(results["tasks"][0]["patch"]["status"], "captured");
    assert!(run_dir.join("report.html").is_file());
}

#[test]
fn int_011_swe_bench_pro_no_diff_is_task_benchmark_failure() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "true");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, _) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 2);
    assert_eq!(results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(results["tasks"][0]["failure_code"], "no_valid_diff");
}

#[test]
fn int_011_swe_bench_pro_workspace_failure_stays_task_failure() {
    for env_key in [
        "HARNESSLAB_FAKE_SWE_METADATA_FAIL",
        "HARNESSLAB_FAKE_SWE_DOCKER_FAIL",
    ] {
        let home = tempfile::tempdir().unwrap();
        init_home(home.path());
        write_swe_gold_agent(home.path());
        let root = swe_bench_root();
        let bin = fake_swe_tools();

        let (results, run_dir) = run_swe_json(
            home.path(),
            root.path(),
            bin.path(),
            "swe-gold",
            &[(env_key, "1")],
            1,
        );
        assert_eq!(results["tasks"][0]["failure_class"], "execution");
        assert_eq!(results["tasks"][0]["failure_code"], "workspace_prep_failed");
        assert!(run_dir.join("report.html").is_file());
        assert!(
            fs::read_to_string(run_dir.join("events.jsonl"))
                .unwrap()
                .contains("external_runner_setup_failed")
        );
    }
}

#[test]
fn int_011_swe_bench_pro_missing_eval_results_is_evaluator_error() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "swe-gold",
        &[("HARNESSLAB_FAKE_SWE_SKIP_EVAL_RESULTS", "1")],
        1,
    );
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "evaluator_error");
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let stderr = fs::read_to_string(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/verifier/stderr.log"),
    )
    .unwrap();
    assert!(stderr.contains("official eval_results unavailable"));
    assert!(
        fs::read_to_string(run_dir.join("events.jsonl"))
            .unwrap()
            .contains("external_result_parse_failed")
    );
}

fn run_swe_json(
    home: &Path,
    root: &Path,
    bin: &Path,
    agent: &str,
    extra_env: &[(&str, &str)],
    expected_code: i32,
) -> (serde_json::Value, PathBuf) {
    let mut command = harnesslab();
    command
        .env("HARNESSLAB_BENCHMARKS_DIR", root)
        .env("PATH", path_with(bin))
        .args([
            "--home",
            home.to_str().unwrap(),
            "run",
            "--agent",
            agent,
            "--benchmark",
            "swe-bench-pro",
            "--split",
            "smoke",
            "--json",
        ]);
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let output = command
        .assert()
        .code(expected_code)
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = PathBuf::from(json["run_dir"].as_str().unwrap());
    let results = serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    (results, run_dir)
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

fn write_agent(home: &Path, command: &str) {
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{command}"
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
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn write_terminal_agent(home: &Path) {
    let content = r#"schema_version = 1
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
terminal_bench_agent = "oracle"
"#;
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

fn write_swe_gold_agent(home: &Path) {
    let content = r#"schema_version = 1
name = "swe-gold"
kind = "fake"
display_name = "SWE Gold"
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
swe_bench_pro_agent = "gold"
"#;
    fs::write(home.join("agents/swe-gold.toml"), content).unwrap();
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

fn swe_bench_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();
    fs::write(
        root.path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 1\n",
    )
    .unwrap();
    let source = root.path().join("_src/SWE-bench_Pro-os");
    fs::create_dir_all(source.join("run_scripts/instance_demo")).unwrap();
    fs::write(source.join("swe_bench_pro_eval.py"), "").unwrap();
    fs::write(source.join("run_scripts/instance_demo/run_script.sh"), "").unwrap();
    fs::write(source.join("run_scripts/instance_demo/parser.py"), "").unwrap();
    root
}

fn fake_swe_tools() -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    write_executable(
        &bin.path().join("uv"),
        r#"#!/bin/sh
if printf '%s\n' "$@" | grep -q -- '-c'; then
  printf 'instance_demo\n'
  exit 0
fi
if printf '%s\n' "$@" | grep -q 'extract_instance.py'; then
  if [ "${HARNESSLAB_FAKE_SWE_METADATA_FAIL:-}" = "1" ]; then
    exit 42
  fi
  raw=""
  info=""
  prev=""
  for arg in "$@"; do
    if [ "$prev" = "instance_demo" ]; then raw="$arg"; fi
    if [ -n "$raw" ] && [ "$prev" = "$raw" ]; then info="$arg"; fi
    prev="$arg"
  done
  printf '{"instance_id":"instance_demo"}\n' > "$raw"
  cat > "$info" <<'JSON'
{"instance_id":"instance_demo","repo":"demo/repo","base_commit":"abc123","dockerhub_tag":"demo-image","problem_statement":"Fix app.txt","requirements":"Change old to new","patch":"diff --git a/app.txt b/app.txt\n--- a/app.txt\n+++ b/app.txt\n@@ -1 +1 @@\n-old\n+new\n"}
JSON
  exit 0
fi
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output_dir) out="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out"
if [ "${HARNESSLAB_FAKE_SWE_SKIP_EVAL_RESULTS:-}" = "1" ]; then
  exit 0
fi
printf '{"instance_demo":true}' > "$out/eval_results.json"
exit 0
"#,
    );
    write_executable(
        &bin.path().join("docker"),
        r#"#!/bin/sh
if [ "${HARNESSLAB_FAKE_SWE_DOCKER_FAIL:-}" = "1" ]; then
  exit 42
fi
case "$1" in
  pull) exit 0 ;;
  run) printf 'cid-demo\n'; exit 0 ;;
  create) printf 'cid-demo\n'; exit 0 ;;
  cp)
    dest="$3"
    mkdir -p "$dest"
    cd "$dest"
    git init -q
    git config user.email harnesslab@example.invalid
    git config user.name HarnessLab
    printf 'old\n' > app.txt
    git add app.txt
    git commit -q -m init
    exit 0
    ;;
  rm) exit 0 ;;
esac
exit 0
"#,
    );
    bin
}

fn write_executable(path: &Path, content: &str) {
    fs::write(path, content).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

fn path_with(bin: &Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin.display())
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
