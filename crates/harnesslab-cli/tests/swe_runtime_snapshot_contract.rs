mod support;

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use support::swe::{
    fake_swe_tools, init_home, path_with, run_swe_json, swe_bench_root, write_agent,
    write_swe_gold_agent,
};

#[test]
fn swepro_005_replay_requires_stored_swe_runtime_materials() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root();
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "swe-gold", &[], 0);
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let attempt_dir = run_dir.join("tasks").join(task_id).join("attempts/1");
    assert_swe_external_runtime_snapshots(&attempt_dir, root.path(), &run_dir, task_id);

    Command::cargo_bin("harnesslab")
        .unwrap()
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
        .success();

    let public_path = attempt_dir.join("external-runtime.public.json");
    let private_path = attempt_dir.join("external-runtime.private.json");
    let public_original = fs::read(&public_path).unwrap();
    let private_original = fs::read(&private_path).unwrap();

    fs::rename(
        &public_path,
        attempt_dir.join("external-runtime.public.json.bak"),
    )
    .unwrap();
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime.public.json missing",
    );
    fs::write(&public_path, &public_original).unwrap();

    let run_count = fs::read_dir(home.path().join("runs")).unwrap().count();
    fs::rename(
        &private_path,
        attempt_dir.join("external-runtime.private.json.bak"),
    )
    .unwrap();
    let stderr = Command::cargo_bin("harnesslab")
        .unwrap()
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
        .get_output()
        .stderr
        .clone();
    assert!(
        String::from_utf8(stderr)
            .unwrap()
            .contains("external-runtime.private.json missing")
    );
    assert_eq!(
        fs::read_dir(home.path().join("runs")).unwrap().count(),
        run_count
    );
    fs::write(&private_path, &private_original).unwrap();

    mutate_json(&public_path, |snapshot| {
        snapshot["schema_version"] = serde_json::json!(2);
    });
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime snapshot mismatch",
    );
    fs::write(&public_path, &public_original).unwrap();

    mutate_json(&public_path, |snapshot| {
        snapshot["commands"][0]["command"] = serde_json::json!("tampered");
    });
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime snapshot mismatch",
    );
    fs::write(&public_path, &public_original).unwrap();

    mutate_json(&private_path, |snapshot| {
        snapshot["commands"][0]["command"] = serde_json::json!("tampered");
    });
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime snapshot mismatch",
    );
    fs::write(&private_path, &private_original).unwrap();

    mutate_json(&private_path, |snapshot| {
        snapshot["replay_materials"][0]["checksum"] = serde_json::json!("fnv64:0000000000000000");
    });
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime snapshot mismatch",
    );
    fs::write(&private_path, &private_original).unwrap();

    fs::create_dir_all(run_dir.join("tasks").join(task_id).join("attempts/2")).unwrap();
    assert_replay_blocker(
        home.path(),
        bin.path(),
        &run_dir,
        "external-runtime.private.json missing",
    );

    assert_public_snapshot_redacts_shell_escaped_private_paths();
    assert_snapshot_uses_executed_parquet_after_dataset_mutation();
}

fn assert_public_snapshot_redacts_shell_escaped_private_paths() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_swe_gold_agent(home.path());
    let root = swe_bench_root_with_prefix("swe'root");
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "swe-gold", &[], 0);
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let public_text = fs::read_to_string(
        run_dir
            .join("tasks")
            .join(task_id)
            .join("attempts/1/external-runtime.public.json"),
    )
    .unwrap();
    let raw_root = root.path().display().to_string();
    let escaped_root = raw_root.replace('\'', "'\\''");
    assert!(!public_text.contains(&raw_root));
    assert!(!public_text.contains(&escaped_root));
    assert!(!public_text.contains(&format!("'{escaped_root}'")));
}

fn assert_snapshot_uses_executed_parquet_after_dataset_mutation() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    let root = swe_bench_root();
    let late_parquet = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data/000-after-extraction.parquet");
    write_agent(
        home.path(),
        &format!(
            "printf new > app.txt; printf late > {}",
            shell_quote(&late_parquet.display().to_string())
        ),
    );
    let bin = fake_swe_tools();

    let (results, run_dir) = run_swe_json(home.path(), root.path(), bin.path(), "fake", &[], 0);
    let task_id = results["tasks"][0]["task_id"].as_str().unwrap();
    let private: serde_json::Value = serde_json::from_slice(
        &fs::read(
            run_dir
                .join("tasks")
                .join(task_id)
                .join("attempts/1/external-runtime.private.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let metadata_command = private["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|command| command["phase"] == "metadata_extraction")
        .unwrap()["command"]
        .as_str()
        .unwrap();
    assert!(metadata_command.contains("test-00000-of-00001.parquet"));
    assert!(!metadata_command.contains("000-after-extraction.parquet"));
}

fn assert_swe_external_runtime_snapshots(
    attempt_dir: &Path,
    benchmark_root: &Path,
    run_dir: &Path,
    task_id: &str,
) {
    let public_path = attempt_dir.join("external-runtime.public.json");
    let private_path = attempt_dir.join("external-runtime.private.json");
    assert!(public_path.is_file());
    assert!(private_path.is_file());

    let public_text = fs::read_to_string(&public_path).unwrap();
    let private_text = fs::read_to_string(&private_path).unwrap();
    assert!(!public_text.contains(&benchmark_root.display().to_string()));
    assert!(!public_text.contains(&run_dir.display().to_string()));
    assert!(!public_text.contains("\"dataset_path\""));
    assert!(!public_text.contains("\"source_path\""));
    assert!(!public_text.contains("\"working_dir\""));
    assert!(!public_text.contains("\"redaction_basis\""));
    assert!(public_text.contains("[REDACTED]"));
    assert!(private_text.contains(&benchmark_root.display().to_string()));
    assert!(private_text.contains("\"dataset_path\""));
    assert!(private_text.contains("\"source_path\""));
    assert!(private_text.contains("\"working_dir\""));

    let public: serde_json::Value = serde_json::from_str(&public_text).unwrap();
    let private: serde_json::Value = serde_json::from_str(&private_text).unwrap();
    assert_eq!(public["visibility"], "public");
    assert_eq!(private["visibility"], "private");
    assert_eq!(public["benchmark"], "swe-bench-pro");
    assert_eq!(private["benchmark"], "swe-bench-pro");
    assert_eq!(public["task_id"], task_id);
    assert_eq!(private["task_id"], task_id);
    assert_eq!(public["runner_kind"], "swe_bench_pro");
    assert_eq!(private["runner_kind"], "swe_bench_pro");
    assert!(public.get("redaction_basis").is_none());
    assert!(
        public["runtime_fingerprint"]
            .as_str()
            .unwrap()
            .starts_with("fnv64:")
    );
    assert!(
        private["runtime_fingerprint"]
            .as_str()
            .unwrap()
            .starts_with("fnv64:")
    );
    assert!(
        private["public_fingerprint"]
            .as_str()
            .unwrap()
            .starts_with("fnv64:")
    );
    assert_eq!(
        public["runtime_fingerprint"],
        private["runtime_fingerprint"]
    );
    assert_json_array_has_name(&public["runtime_materials"], "parquet");
    assert_json_array_has_name(&public["runtime_materials"], "evaluator");
    assert_json_array_has_name(&private["replay_materials"], "raw_sample");
    assert_json_array_has_name(&private["replay_materials"], "prediction_eval_json");
    assert_json_array_has_name(&private["commands"], "metadata_extraction");
    assert_json_array_has_name(&private["commands"], "workspace_preparation");
    assert_json_array_has_name(&private["commands"], "evaluation");
}

fn assert_replay_blocker(home: &Path, bin: &Path, run_dir: &Path, message: &str) {
    let run_count = fs::read_dir(home.join("runs")).unwrap().count();
    let stderr = Command::cargo_bin("harnesslab")
        .unwrap()
        .env("PATH", path_with(bin))
        .args([
            "--home",
            home.to_str().unwrap(),
            "run",
            "replay",
            run_dir.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    assert!(String::from_utf8(stderr).unwrap().contains(message));
    assert_eq!(fs::read_dir(home.join("runs")).unwrap().count(), run_count);
}

fn mutate_json(path: &Path, mutate: impl FnOnce(&mut serde_json::Value)) {
    let mut snapshot: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    mutate(&mut snapshot);
    fs::write(path, serde_json::to_vec_pretty(&snapshot).unwrap()).unwrap();
}

fn swe_bench_root_with_prefix(prefix: &str) -> tempfile::TempDir {
    let root = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn assert_json_array_has_name(array: &serde_json::Value, name: &str) {
    assert!(
        array
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["name"] == name || entry["phase"] == name),
        "missing {name} in {array}"
    );
}
