#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use std::fs;
use std::path::Path;
use terminal_bench_support::*;

#[test]
fn agt_reg_004_terminal_bench_attempt_command_snapshot_redacts_setup_secret() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_secret_setup_agent(home.path());
    let root = terminal_bench_root();
    let bin = fake_uvx(success_result_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_ATTEMPT_SECRET", "pa'ss")],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    let command =
        fs::read_to_string(run_dir.join("tasks/hello-world/attempts/1/agent/command.txt")).unwrap();
    assert!(command.contains("[REDACTED]"));
    assert!(!command.contains("pa'ss"));
    assert!(!command.contains(r#"pa'\''ss"#));
}

fn success_result_script() -> &'static str {
    r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$out/$run"
printf '{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}' > "$out/$run/results.json"
exit 0
"#
}

fn write_secret_setup_agent(home: &Path) {
    fs::write(
        home.join("agents/fake.toml"),
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
inherit_env = ["HARNESSLAB_ATTEMPT_SECRET"]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "custom"
required_commands = []
run_as = "current"
commands = ["printf \"pa'ss\" >/tmp/harnesslab-attempt-secret"]

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = []

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"

[labels]
terminal_bench_agent = "oracle"
"#,
    )
    .unwrap();
}
