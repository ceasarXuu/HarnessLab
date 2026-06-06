#![allow(dead_code)]

use assert_cmd::Command;
use std::fs;
use std::path::Path;

pub fn run_terminal(
    home: &Path,
    root: &Path,
    bin: &Path,
    expected_code: i32,
) -> (serde_json::Value, std::path::PathBuf, serde_json::Value) {
    run_terminal_with_extra_args(home, root, bin, &[], expected_code)
}

pub fn run_terminal_with_extra_args(
    home: &Path,
    root: &Path,
    bin: &Path,
    extra_args: &[&str],
    expected_code: i32,
) -> (serde_json::Value, std::path::PathBuf, serde_json::Value) {
    run_terminal_with_split_and_extra_args(home, root, bin, "smoke", extra_args, expected_code)
}

pub fn run_terminal_with_split_and_extra_args(
    home: &Path,
    root: &Path,
    bin: &Path,
    split: &str,
    extra_args: &[&str],
    expected_code: i32,
) -> (serde_json::Value, std::path::PathBuf, serde_json::Value) {
    run_terminal_with_split_extra_args_and_env(
        home,
        root,
        bin,
        split,
        extra_args,
        &[],
        expected_code,
    )
}

pub fn run_terminal_with_split_extra_args_and_env(
    home: &Path,
    root: &Path,
    bin: &Path,
    split: &str,
    extra_args: &[&str],
    extra_env: &[(&str, &str)],
    expected_code: i32,
) -> (serde_json::Value, std::path::PathBuf, serde_json::Value) {
    let mut args = vec![
        "--home",
        home.to_str().unwrap(),
        "run",
        "--agent",
        "fake",
        "--benchmark",
        "terminal-bench",
        "--split",
        split,
        "--json",
    ];
    args.extend_from_slice(extra_args);
    let mut command = harnesslab();
    command
        .env("HARNESSLAB_BENCHMARKS_DIR", root)
        .env("PATH", path_with(bin));
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let output = command
        .args(args)
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

pub fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

pub fn write_agent(home: &Path, agent: &str, model: Option<&str>, import_path: Option<&str>) {
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

pub fn write_agent_with_labels(home: &Path, labels: &str) {
    write_agent_with_labels_and_run_as(home, labels, "current");
}

pub fn write_agent_with_labels_and_run_as(home: &Path, labels: &str, run_as: &str) {
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

[setup]
preset = "none"
required_commands = []
run_as = "{run_as}"
commands = []

[usage]
parser = "none"

[labels]
{labels}"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}

pub fn success_when_model_is(expected: &str) -> &'static str {
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

pub fn terminal_bench_root() -> tempfile::TempDir {
    terminal_bench_root_with_tasks(&["hello-world"])
}

pub fn terminal_bench_root_with_tasks(tasks: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for task in tasks {
        let task_dir = root
            .path()
            .join("terminal-bench/terminal-bench-core-0.1.1")
            .join(task);
        fs::create_dir_all(&task_dir).unwrap();
        fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();
    }
    root
}

pub fn fake_uvx(body: &str) -> tempfile::TempDir {
    fake_uvx_and_docker(body, Some("exit 0\n"))
}

pub fn fake_uvx_and_docker(body: &str, docker_body: Option<&str>) -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    write_executable(&bin.path().join("uvx"), body);
    if let Some(docker_body) = docker_body {
        write_executable(&bin.path().join("docker"), docker_body);
    }
    bin
}

pub fn fake_uvx_and_docker_buildx(body: &str) -> tempfile::TempDir {
    let bin = tempfile::tempdir().unwrap();
    write_executable(&bin.path().join("uvx"), body);
    write_executable(&bin.path().join("docker"), "exit 0\n");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/bin/sleep", bin.path().join("docker-buildx")).unwrap();
    #[cfg(not(unix))]
    write_executable(&bin.path().join("docker-buildx"), "sleep \"$1\"\n");
    bin
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}")).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

pub fn harnesslab() -> Command {
    if let Some(path) = option_env!("CARGO_BIN_EXE_harnesslab") {
        Command::new(path)
    } else {
        Command::cargo_bin("harnesslab").unwrap()
    }
}

pub fn path_with(bin: &Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin.display())
}
