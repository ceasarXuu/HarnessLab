use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn int_012_swe_evaluator_ignores_ambient_python_environment() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let output = harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env("PATH", path_with(bin.path()))
        .env("PYTHONPATH", "/ambient/site-packages")
        .env("PYTHONHOME", "/ambient/python")
        .env("PYTHONUSERBASE", "/ambient/userbase")
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "swe-gold",
            "--benchmark",
            "swe-bench-pro",
            "--split",
            "smoke",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let run_dir = PathBuf::from(json["run_dir"].as_str().unwrap());
    let results: serde_json::Value =
        serde_json::from_slice(&fs::read(run_dir.join("results.json")).unwrap()).unwrap();
    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["benchmark_score"], 1.0);
}

fn init_home(home: &Path) {
    harnesslab()
        .args(["--home", home.to_str().unwrap(), "init"])
        .assert()
        .success();
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
	  if [ "${PYTHONPATH+x}" = "x" ] || [ "${PYTHONHOME+x}" = "x" ] || [ "${PYTHONUSERBASE+x}" = "x" ] || [ "${PYTHONNOUSERSITE:-}" != "1" ]; then
	    printf 'metadata python env not isolated\n' >&2
	    exit 77
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
	if [ "${PYTHONPATH+x}" = "x" ] || [ "${PYTHONHOME+x}" = "x" ] || [ "${PYTHONUSERBASE+x}" = "x" ] || [ "${PYTHONNOUSERSITE:-}" != "1" ]; then
	  printf 'evaluator python env not isolated\n' >&2
	  exit 77
	fi
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output_dir) out="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out"
printf '{"instance_demo":true}' > "$out/eval_results.json"
exit 0
"#,
    );
    write_executable(
        &bin.path().join("docker"),
        r#"#!/bin/sh
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
