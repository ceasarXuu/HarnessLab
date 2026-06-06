#[path = "support/external_runtime_assertions.rs"]
mod external_runtime_assertions;
#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use external_runtime_assertions::*;
use std::fs;
use std::path::Path;
use terminal_bench_support::*;

#[test]
fn adapt_runtime_003_external_runtime_snapshots_are_written_and_redacted() {
    let secret = "sk-phase6-runtime-secret";
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_phase6_secret_agent(home.path(), secret);
    let root = terminal_bench_root();
    let bin = fake_uvx(success_script());

    let (results, run_dir, _) = run_terminal_with_split_extra_args_and_env(
        home.path(),
        root.path(),
        bin.path(),
        "smoke",
        &[],
        &[("HARNESSLAB_PHASE6_SECRET", secret)],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let public_text = fs::read_to_string(attempt_dir.join("external-runtime.public.json")).unwrap();
    let private_text =
        fs::read_to_string(attempt_dir.join("external-runtime.private.json")).unwrap();
    let public: serde_json::Value = serde_json::from_str(&public_text).unwrap();
    let private: serde_json::Value = serde_json::from_str(&private_text).unwrap();

    assert_eq!(public["visibility"], "public");
    assert_eq!(private["visibility"], "private");
    assert_eq!(public["benchmark"], "terminal-bench");
    assert_eq!(private["benchmark"], "terminal-bench");
    assert_eq!(public["runner_kind"], "terminal_bench");
    assert_eq!(private["runner_kind"], "terminal_bench");
    assert_eq!(public["adapter_version"], "terminal-bench-runtime.v1");
    assert_eq!(private["adapter_version"], "terminal-bench-runtime.v1");
    assert_eq!(
        public["runtime_fingerprint"],
        private["runtime_fingerprint"]
    );
    assert!(
        private["public_fingerprint"]
            .as_str()
            .unwrap()
            .starts_with("fnv64:")
    );
    assert_json_array_has_phase(&public["commands"], "official_runner");
    assert_json_array_has_phase(&private["commands"], "official_runner");
    assert_json_array_has_name(&public["runtime_materials"], "source_dataset");
    assert_json_array_has_name(&public["runtime_materials"], "runtime_dataset");
    assert_json_array_has_name(&public["runtime_materials"], "official_result");
    assert_json_array_has_name(&public["runtime_materials"], "cleanup_report");
    assert_json_array_missing_name(&public["runtime_materials"], "command_snapshot");
    assert_json_array_missing_name(&public["runtime_materials"], "runner_stdout");
    assert_json_array_missing_name(&public["runtime_materials"], "runner_stderr");
    assert_json_array_has_name(&private["replay_materials"], "official_result");
    assert_json_array_has_name(&private["replay_materials"], "command_snapshot");
    assert_json_array_has_name(&private["replay_materials"], "runner_stdout");
    assert_json_array_has_name(&private["replay_materials"], "runner_stderr");
    assert_eq!(
        material_kind(&private["replay_materials"], "source_dataset"),
        "directory"
    );
    assert_eq!(
        material_kind(&private["replay_materials"], "runtime_dataset"),
        "directory"
    );
    assert_eq!(
        material_kind(&private["replay_materials"], "official_result"),
        "file"
    );
    let official_result_public_path =
        material_public_path(&public["runtime_materials"], "official_result");
    assert_public_artifacts_eq(
        &public["public_artifacts"],
        &["cleanup-report.json", &official_result_public_path],
    );
    assert!(!artifact_list_contains(
        &public["public_artifacts"],
        "agent/command.txt"
    ));
    assert_eq!(
        public["runtime_diagnostics"]["cleanup"]["final_verdict_effect"],
        "none"
    );
    assert_eq!(
        private["runtime_diagnostics"]["cleanup"]["final_verdict_effect"],
        "none"
    );
    assert_eq!(
        public["runtime_diagnostics"]["cleanup"]["phases"][0]["phase"],
        "pre_task"
    );
    assert_eq!(
        public["runtime_diagnostics"]["cleanup"]["phases"][1]["phase"],
        "post_task"
    );
    assert_eq!(
        private["runtime_diagnostics"]["cleanup"]["phases"][0]["token"],
        private["runtime_diagnostics"]["cleanup"]["phases"][1]["token"]
    );

    assert!(!public_text.contains(secret));
    assert!(!public_text.contains(&root.path().display().to_string()));
    assert!(!public_text.contains(&run_dir.display().to_string()));
    assert!(!public_text.contains(results["run_id"].as_str().unwrap()));
    assert!(public_text.contains("official/terminal-bench/results.json"));
    assert!(!public_text.contains("official/terminal-bench/fake-terminal-bench"));
    assert!(public_text.contains("--run-id [PRIVATE_RUN_ID]"));
    for raw_path in [
        "agent/command.txt",
        "agent/stdout.log",
        "agent/stderr.log",
        "verifier/stdout.log",
        "verifier/stderr.log",
    ] {
        assert!(
            !public_text.contains(raw_path),
            "public snapshot leaked {raw_path}"
        );
    }
    assert!(!public_text.contains("\"dataset_path\""));
    assert!(!public_text.contains("\"source_path\""));
    assert!(!public_text.contains("\"working_dir\""));
    assert!(!public_text.contains("\"redaction_basis\""));
    assert!(private_text.contains(secret));
    assert!(private_text.contains("\"dataset_path\""));
    assert!(private_text.contains("\"working_dir\""));
    let command = fs::read_to_string(attempt_dir.join("agent/command.txt")).unwrap();
    assert!(!command.contains(secret));
    assert!(!command.contains(&root.path().display().to_string()));
    assert!(!command.contains(&run_dir.display().to_string()));
    assert!(command.contains("[REDACTED]"));
    assert_public_text_file_does_not_contain(&run_dir.join("events.jsonl"), secret);
    assert_public_text_file_does_not_contain(
        &run_dir.join("events.jsonl"),
        results["run_id"].as_str().unwrap(),
    );
    assert_public_text_file_contains(&run_dir.join("events.jsonl"), "[PRIVATE_RUN_ID]");
    assert_public_text_file_does_not_contain(
        &run_dir.join("events.jsonl"),
        &root.path().display().to_string(),
    );
    assert_public_text_file_does_not_contain(
        &run_dir.join("events.jsonl"),
        &run_dir.display().to_string(),
    );
    assert_public_text_file_does_not_contain(&attempt_dir.join("cleanup-report.json"), secret);
    assert_public_text_file_does_not_contain(&run_dir.join("report.html"), secret);
    assert_public_text_file_does_not_contain(
        &run_dir.join("report.html"),
        results["run_id"].as_str().unwrap(),
    );
    assert_public_text_file_contains(&run_dir.join("report.html"), "[PRIVATE_RUN_ID]");
    for raw_link in [
        "command.txt",
        "agent/stdout.log",
        "agent/stderr.log",
        "verifier/stdout.log",
        "verifier/stderr.log",
    ] {
        assert_public_text_file_does_not_contain(&run_dir.join("report.html"), raw_link);
    }
    assert_public_text_file_does_not_contain(
        &run_dir.join("report.html"),
        &root.path().display().to_string(),
    );

    harnesslab()
        .env("PATH", path_with(bin.path()))
        .env("HARNESSLAB_PHASE6_SECRET", secret)
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();

    assert_import_path_public_surface_is_redacted();
}

#[test]
fn adapt_runtime_004_cleanup_report_is_structured_and_affects_final_verdict() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{{"task_id":"hello-world","is_resolved":true}}]}}' > "$out/$run/results.json"
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));

    let (results, run_dir, json) = run_terminal(home.path(), root.path(), bin.path(), 1);

    assert_eq!(json["verdict"], "execution_failure");
    assert_eq!(results["summary"]["execution_failure"], 1);
    let task = &results["tasks"][0];
    assert_eq!(task["failure_class"], "execution");
    assert_eq!(task["failure_code"], "agent_cleanup_failed");

    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_dir.join("cleanup-report.json")).unwrap())
            .unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["benchmark"], "terminal-bench");
    assert_eq!(report["task_id"], "hello-world");
    assert_eq!(report["final_verdict_effect"], "cleanup_overrode_result");
    assert_eq!(report["official_failure"]["class"], "none");
    assert_eq!(report["final_failure"]["class"], "execution");
    assert_eq!(report["final_failure"]["code"], "agent_cleanup_failed");
    let post_task = report["phases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|phase| phase["phase"] == "post_task")
        .unwrap();
    assert_eq!(post_task["required"], false);
    assert_eq!(post_task["success"], false);
    assert_eq!(post_task["has_error"], true);
    assert_eq!(post_task["containers_removed"], 0);
    assert_eq!(post_task["networks_removed"], 0);
    assert_eq!(post_task["projects_count"], 1);
    let report_text = fs::read_to_string(attempt_dir.join("cleanup-report.json")).unwrap();
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    let report_html = fs::read_to_string(run_dir.join("report.html")).unwrap();
    for raw in [
        "actual-prefix-",
        "metadata write failed",
        "container:c1",
        "network:n1",
    ] {
        assert!(!report_text.contains(raw), "cleanup-report leaked {raw}");
        assert!(!events.contains(raw), "events leaked {raw}: {events}");
        assert!(!report_html.contains(raw), "report leaked {raw}");
    }
    let public_snapshot: serde_json::Value = serde_json::from_slice(
        &fs::read(attempt_dir.join("external-runtime.public.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        public_snapshot["runtime_diagnostics"]["cleanup"]["final_verdict_effect"],
        "cleanup_overrode_result"
    );

    let warning_report = run_cleanup_warning_only_case();
    assert_eq!(
        warning_report["final_verdict_effect"],
        "cleanup_warning_only"
    );
    assert_eq!(warning_report["official_failure"]["class"], "execution");
    assert_eq!(warning_report["final_failure"]["class"], "execution");
    let warning_public = warning_report["public_snapshot"].clone();
    assert_eq!(
        warning_public["runtime_diagnostics"]["cleanup"]["final_verdict_effect"],
        "cleanup_warning_only"
    );
}

fn run_cleanup_warning_only_case() -> serde_json::Value {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "oracle", None, None);
    let root = terminal_bench_root();
    let marker = home.path().join("compose-project.txt");
    let uvx = format!(
        r#"out=""; run=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-path) out="$2"; shift 2 ;;
    --run-id) run="$2"; shift 2 ;;
    *) shift ;;
  esac
done
printf 'actual-prefix-%s' "$run" > '{}'
mkdir -p "$out/$run"
printf '{{not-json' > "$out/$run/results.json"
exit 0
"#,
        marker.display()
    );
    let docker = format!(
        r#"if [ "$1 $2 $3 $4" = "ps -a --filter label=com.docker.compose.project" ]; then
  if [ -f '{}' ]; then
    project=$(cat '{}')
    printf 'c1\t%s\n' "$project"
  fi
  exit 0
fi
case "$*" in
  ps\ -aq\ --filter\ label=com.docker.compose.project=actual-prefix-*) printf 'c1\n'; exit 0 ;;
  rm\ -f\ c1) echo "metadata write failed" >&2; exit 64 ;;
  network\ ls\ -q\ --filter\ label=com.docker.compose.project=actual-prefix-*) exit 0 ;;
esac
exit 0
"#,
        marker.display(),
        marker.display()
    );
    let bin = fake_uvx_and_docker(&uvx, Some(&docker));
    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 1);
    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_dir.join("cleanup-report.json")).unwrap())
            .unwrap();
    report["public_snapshot"] = serde_json::from_slice(
        &fs::read(attempt_dir.join("external-runtime.public.json")).unwrap(),
    )
    .unwrap();
    report
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

fn assert_import_path_public_surface_is_redacted() {
    let import_path = "bench_agents.fake:Agent";
    let pythonpath = "/tmp/phase8-private-pythonpath";
    let agent_command = "printf phase8-private-agent-command";
    let setup_command = "printf phase8-private-setup-command";
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_import_agent_with_private_material(
        home.path(),
        import_path,
        pythonpath,
        agent_command,
        setup_command,
    );
    let root = terminal_bench_root();
    let bin = fake_uvx(success_script());
    let (results, run_dir, _) = run_terminal(home.path(), root.path(), bin.path(), 0);
    assert_eq!(results["tasks"][0]["state"], "success");
    let attempt_dir = run_dir.join("tasks/hello-world/attempts/1");
    let public: serde_json::Value = serde_json::from_slice(
        &fs::read(attempt_dir.join("external-runtime.public.json")).unwrap(),
    )
    .unwrap();
    let official_result_public_path =
        material_public_path(&public["runtime_materials"], "official_result");
    assert_public_artifacts_eq(
        &public["public_artifacts"],
        &["cleanup-report.json", &official_result_public_path],
    );
    for path in [run_dir.join("events.jsonl"), run_dir.join("report.html")] {
        assert_public_text_file_does_not_contain(&path, import_path);
        assert_public_text_file_does_not_contain(&path, &root.path().display().to_string());
        assert_public_text_file_does_not_contain(&path, &run_dir.display().to_string());
        assert_public_text_file_does_not_contain(&path, results["run_id"].as_str().unwrap());
    }
    let public_text = fs::read_to_string(attempt_dir.join("external-runtime.public.json")).unwrap();
    let command_text = fs::read_to_string(attempt_dir.join("agent/command.txt")).unwrap();
    assert!(!public_text.contains(import_path));
    assert!(public_text.contains("--agent-import-path [PRIVATE_AGENT_IMPORT]"));
    for raw in [import_path, pythonpath, agent_command, setup_command] {
        assert!(
            !public_text.contains(raw),
            "public runtime snapshot leaked {raw}"
        );
        assert!(
            !command_text.contains(raw),
            "public command snapshot leaked {raw}"
        );
    }
    assert!(command_text.contains("--agent-import-path '[REDACTED]'"));
    assert_public_text_file_contains(&run_dir.join("events.jsonl"), "[PRIVATE_RUN_ID]");
    assert_public_text_file_contains(&run_dir.join("report.html"), "[PRIVATE_RUN_ID]");
}

fn write_phase6_secret_agent(home: &Path, secret: &str) {
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
inherit_env = ["HARNESSLAB_PHASE6_SECRET"]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "custom"
required_commands = []
run_as = "current"
commands = ["printf '{secret}' >/tmp/harnesslab-phase6-secret"]

[usage]
parser = "none"

[labels]
terminal_bench_agent = "oracle"
"#
    );
    fs::write(home.join("agents/fake.toml"), content).unwrap();
}
