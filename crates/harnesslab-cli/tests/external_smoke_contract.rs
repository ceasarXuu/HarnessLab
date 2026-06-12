mod support;

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use support::swe::{
    fake_swe_tools, init_home, path_with, run_swe_json, run_swe_json_with_output,
    set_network_default_none, swe_bench_root, write_agent, write_agent_with_mode,
    write_codex_agent, write_swe_gold_agent, write_swe_gold_agent_with_run_as,
};

const INT_011_RUNTIME_ARTIFACTS: &str =
    include_str!("../../../tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt");

#[test]
fn int_011_swe_bench_pro_smoke_runs_external_evaluator_contract() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir, json) =
        run_swe_json_with_output(home.path(), root.path(), bin.path(), "swe-gold", &[], 0);
    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["benchmark_score"], 1.0);
    assert_eq!(results["tasks"][0]["patch"]["status"], "captured");
    assert_swe_runtime_artifacts(&results, &run_dir, &json);
}

#[test]
fn agt_reg_012_swe_gold_run_as_blocks_before_run_dir() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent_with_run_as(home.path(), "harnesslab");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let output = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env("PATH", path_with(bin.path()))
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
        .code(3)
        .get_output()
        .stderr
        .clone();

    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("setup.run_as"));
    assert!(stderr.contains("swe-bench-pro gold host path"));
    assert!(stderr.contains("current"));
    assert_eq!(fs::read_dir(home.path().join("runs")).unwrap().count(), 0);
}

#[test]
fn int_011_swe_bench_pro_real_profile_runs_in_prepared_workspace() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    set_network_default_none(home.path());
    write_agent(home.path(), "printf 'new\\n' > app.txt");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["benchmark_score"], 1.0);
    assert_eq!(results["tasks"][0]["patch"]["status"], "captured");
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let sandbox: serde_json::Value = serde_json::from_slice(
        &fs::read(
            run_dir
                .join("tasks")
                .join(task_id)
                .join("attempts/1/swe-bench-pro/agent-sandbox.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(sandbox["image"], "jefzda/sweap-images:demo-image");
    assert_eq!(sandbox["network"], "full");
    assert_eq!(sandbox["timeout_sec"], 7200);
    assert_eq!(
        sandbox["effective_docker_request"]["image"],
        "jefzda/sweap-images:demo-image"
    );
    assert_eq!(sandbox["effective_docker_request"]["network"], "full");
    assert!(
        fs::read_to_string(run_dir.join("events.jsonl"))
            .unwrap()
            .contains("external_runner_agent_sandbox_starting")
    );
    assert!(run_dir.join("report.html").is_file());
}

#[test]
fn int_011_swe_bench_pro_codex_profile_provisions_cli_in_sandbox() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_codex_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "codex",
        &[("HARNESSLAB_FAKE_REQUIRE_CODEX_SETUP", "1")],
        0,
    );

    assert_eq!(results["tasks"][0]["state"], "success");
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let command = fs::read_to_string(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/agent/command.txt"),
    )
    .unwrap();
    assert!(command.contains("npm install -g @openai/codex"));
}

#[test]
fn int_011_swe_bench_pro_real_profile_supports_argument_input_mode() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_mode(
        home.path(),
        "case {{instruction}} in *Repository:*) printf new > app.txt;; *) exit 67;; esac",
        "argument",
        "workspace",
    );
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, _) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 0);

    assert_eq!(results["tasks"][0]["state"], "success");
    assert_eq!(results["tasks"][0]["patch"]["status"], "captured");
}

#[test]
fn int_011_swe_bench_pro_real_profile_supports_file_input_mode() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_mode(
        home.path(),
        "test -s {{instruction_file}} && printf new > app.txt",
        "file",
        "workspace",
    );
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 0);
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();

    assert_eq!(results["tasks"][0]["state"], "success");
    assert!(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/workspace/instruction.txt")
            .is_file()
    );
    assert!(
        fs::read_to_string(
            run_dir
                .join("tasks")
                .join(task_id)
                .join("attempts/1/agent/command.txt")
        )
        .unwrap()
        .contains("input_mode=File")
    );
}

#[test]
fn int_011_swe_bench_pro_no_diff_exits_0_with_benchmark_verdict() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "true");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir, json) =
        run_swe_json_with_output(home.path(), root.path(), bin.path(), "fake", &[], 0);
    assert_eq!(results["tasks"][0]["failure_class"], "benchmark");
    assert_eq!(results["tasks"][0]["failure_code"], "no_valid_diff");
    assert_eq!(json["verdict"], "benchmark_failure");
    assert_eq!(json["summary"]["benchmark_failure"], 1);
    assert_eq!(json["report_path"], results["report_path"]);
    assert_eq!(
        json["results_path"],
        run_dir.join("results.json").display().to_string()
    );
}

#[test]
fn int_011_swe_bench_pro_agent_sandbox_create_failure_is_preserved() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "true");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(
        home.path(),
        root.path(),
        bin.path(),
        "fake",
        &[("HARNESSLAB_FAKE_SWE_AGENT_DOCKER_FAIL", "1")],
        1,
    );

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "sandbox_create_failed");
    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("external_runner_agent_sandbox_starting"));
    assert!(events.contains("external_runner_agent_sandbox_failed"));
    assert!(!events.contains("external_runner_agent_sandbox_started"));
}

#[test]
fn int_011_swe_bench_pro_git_diff_failure_is_execution_failure() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent(home.path(), "rm -rf .git");
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 1);
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();

    assert_eq!(results["tasks"][0]["failure_class"], "execution");
    assert_eq!(results["tasks"][0]["failure_code"], "patch_apply_failed");
    assert_eq!(results["tasks"][0]["patch"]["status"], "apply_failed");
    assert!(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/git-diff.status.json")
            .is_file()
    );
    assert!(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/git-diff.stderr.log")
            .is_file()
    );
    let status: serde_json::Value = serde_json::from_slice(
        &fs::read(
            run_dir
                .join("tasks")
                .join(task_id)
                .join("attempts/1/git-diff.status.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(status["success"], false);
    assert_ne!(status["exit_code"], 0);
    assert!(
        !fs::read_to_string(
            run_dir
                .join("tasks")
                .join(task_id)
                .join("attempts/1/git-diff.stderr.log")
        )
        .unwrap()
        .is_empty()
    );
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
        let expected_code = if env_key == "HARNESSLAB_FAKE_SWE_METADATA_FAIL" {
            "metadata_extraction_failed"
        } else {
            "workspace_prep_failed"
        };
        assert_eq!(results["tasks"][0]["failure_code"], expected_code);
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

fn assert_swe_runtime_artifacts(
    results: &serde_json::Value,
    run_dir: &Path,
    json: &serde_json::Value,
) {
    assert_eq!(json["exit_code"], 0);
    assert_eq!(json["verdict"], "success");
    assert_eq!(json["summary"]["success"], 1);
    assert_eq!(
        json["results_path"],
        run_dir.join("results.json").display().to_string()
    );
    assert_eq!(
        json["report_path"],
        run_dir.join("report.html").display().to_string()
    );
    assert_eq!(
        results["report_path"],
        run_dir.join("report.html").display().to_string()
    );

    let command = fs::read_to_string(run_dir.join("command.txt")).unwrap();
    assert!(command.contains("agent_runtime_snapshot=agent-profile.runtime.json"));
    assert!(command.contains("agent_report_snapshot=agent-profile.snapshot.json"));
    assert!(command.contains("agent_materialized_snapshot=agent-runtime.materialized.json"));

    let events = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(events.contains("\"event\":\"external_runner_preflight\""));
    assert!(events.contains("adapter_id=harnesslab.swe-bench-pro.runtime"));
    assert!(events.contains("adapter_phase=preflight"));
    assert!(events.contains("agent_bridge_mode=swe-bench-pro-gold"));
    assert!(events.contains("readiness_status=ready"));
    assert!(events.contains("host_execution_reason=swe-bench-pro gold host path"));
    assert!(events.contains("blocking_reason=none"));
    assert!(events.contains("compatibility_exception=host-agent-run-as-current-only"));
    assert!(events.contains("compatibility_label_keys=swe_bench_pro_agent"));
    assert!(events.contains("run_finished"));
    assert!(events.contains("exit_code=0"));
    assert!(events.contains("success=1"));
    assert!(events.contains("report_path="));

    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let attempt_dir = run_dir.join("tasks").join(task_id).join("attempts/1");
    for relative in int_011_runtime_artifact_contract() {
        let resolved = relative.replace("<task-id>", task_id);
        assert!(run_dir.join(&resolved).is_file(), "missing {resolved}");
    }

    let attempt_result: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_dir.join("result.json")).unwrap()).unwrap();
    assert_eq!(attempt_result["task_id"], task_id);
    assert_eq!(attempt_result["patch"]["diff_path"], "patch.diff");
    assert_eq!(
        attempt_result["patch"]["prediction_path"],
        "prediction.jsonl"
    );
    assert!(
        fs::read_to_string(attempt_dir.join("prediction.jsonl"))
            .unwrap()
            .contains(task_id)
    );
}

fn int_011_runtime_artifact_contract() -> Vec<&'static str> {
    INT_011_RUNTIME_ARTIFACTS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
}
