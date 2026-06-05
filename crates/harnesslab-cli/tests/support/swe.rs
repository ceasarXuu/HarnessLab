use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_swe_json(
    home: &Path,
    root: &Path,
    bin: &Path,
    agent: &str,
    extra_env: &[(&str, &str)],
    expected_code: i32,
) -> (serde_json::Value, PathBuf) {
    let (results, run_dir, _) =
        run_swe_json_with_output(home, root, bin, agent, extra_env, expected_code);
    (results, run_dir)
}

pub fn run_swe_json_with_output(
    home: &Path,
    root: &Path,
    bin: &Path,
    agent: &str,
    extra_env: &[(&str, &str)],
    expected_code: i32,
) -> (serde_json::Value, PathBuf, serde_json::Value) {
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
    (results, run_dir, json)
}

pub fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
}

pub fn set_network_default_none(home: &Path) {
    let config_path = home.join("config.toml");
    let config = fs::read_to_string(&config_path)
        .unwrap()
        .replace("network_default = \"full\"", "network_default = \"none\"");
    fs::write(config_path, config).unwrap();
}

pub fn write_agent(home: &Path, command: &str) {
    write_agent_with_mode(home, command, "stdin", "workspace");
}

pub fn write_codex_agent(home: &Path) {
    let content = r#"schema_version = 1
name = "codex"
kind = "codex"
display_name = "Codex"
command = "codex exec --full-auto -"
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
"#;
    fs::write(home.join("agents/codex.toml"), content).unwrap();
}

pub fn write_agent_with_mode(home: &Path, command: &str, input_mode: &str, working_dir: &str) {
    let content = format!(
        r#"schema_version = 1
name = "fake"
kind = "fake"
display_name = "Fake"
command = "{command}"
input_mode = "{input_mode}"
working_dir = "{working_dir}"
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

pub fn write_swe_gold_agent(home: &Path) {
    write_swe_gold_agent_with_run_as(home, "current");
}

pub fn write_swe_gold_agent_with_run_as(home: &Path, run_as: &str) {
    let content = format!(
        r#"schema_version = 1
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

[setup]
preset = "none"
required_commands = []
run_as = "{run_as}"
commands = []

[usage]
parser = "none"

[labels]
swe_bench_pro_agent = "gold"
"#,
    );
    fs::write(home.join("agents/swe-gold.toml"), content).unwrap();
}

pub fn swe_bench_root() -> tempfile::TempDir {
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

pub fn fake_swe_tools() -> tempfile::TempDir {
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
if [ "${HARNESSLAB_FAKE_SWE_CORRUPT_EVAL_RESULTS:-}" = "1" ]; then
  printf '{"instance_demo":' > "$out/eval_results.json"
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
  run)
    state="$0.workspace"
    prev=""
    saw_image=0
    for arg in "$@"; do
      if [ "$arg" = "jefzda/sweap-images:demo-image" ]; then
        saw_image=1
      fi
      if [ "$prev" = "-v" ]; then
        case "$arg" in
          *:/workspace) printf '%s\n' "${arg%:/workspace}" > "$state" ;;
        esac
      fi
      prev="$arg"
    done
    if [ "$saw_image" != "1" ]; then
      echo "agent sandbox did not use official image" >&2
      exit 68
    fi
    if [ "${HARNESSLAB_FAKE_SWE_AGENT_DOCKER_FAIL:-}" = "1" ]; then
      echo "agent sandbox failed" >&2
      exit 69
    fi
    printf 'cid-demo\n'
    exit 0
    ;;
  create) printf 'cid-demo\n'; exit 0 ;;
  cp)
    dest="$3"
    mkdir -p "$dest"
    cd "$dest"
    if [ -e app.txt ]; then
      echo "workspace was dirty before official prep" >&2
      exit 66
    fi
    git init -q
    git config user.email harnesslab@example.invalid
    git config user.name HarnessLab
    printf 'old\n' > app.txt
    git add app.txt
    git commit -q -m init
    exit 0
    ;;
  exec)
    workspace="$(cat "$0.workspace")"
    cmd=""
    for arg in "$@"; do
      cmd="$arg"
    done
    cmd="$(printf '%s' "$cmd" | sed "s#/workspace#$workspace#g")"
    cd "$workspace"
    if [ "${HARNESSLAB_FAKE_REQUIRE_CODEX_SETUP:-}" = "1" ]; then
      case "$cmd" in
        *"npm install -g @openai/codex"*"codex exec"*) printf 'new\n' > app.txt; exit 0 ;;
        *"codex exec"*) echo "codex missing in sandbox" >&2; exit 127 ;;
      esac
    fi
    sh -c "$cmd"
    exit $?
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

pub fn path_with(bin: &Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    format!("{}:{current}", bin.display())
}

fn harnesslab() -> Command {
    Command::cargo_bin("harnesslab").unwrap()
}
