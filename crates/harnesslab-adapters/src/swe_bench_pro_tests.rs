use super::*;

#[test]
fn c_bench_006_swe_bench_pro_full_reports_local_data_as_ready() {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();
    std::fs::write(
        root.path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 1\n",
    )
    .unwrap();
    create_source(root.path());

    let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 1);
    assert_eq!(full.data_state, DataState::Ready);
}

#[test]
fn c_bench_006_swe_bench_pro_full_reports_missing_data_as_requires_auth() {
    let root = tempfile::tempdir().unwrap();

    let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 0);
    assert_eq!(full.data_state, DataState::RequiresAuth);
}

#[test]
fn c_bench_006_swe_bench_pro_num_examples_parser_handles_comments() {
    assert_eq!(
        read_num_examples_from_str("splits:\n- name: test\n  num_examples: 731 # comment\n"),
        Some(731)
    );
    assert_eq!(
        read_num_examples_from_str("  num_examples:   731  \r\n"),
        Some(731)
    );
    assert_eq!(read_num_examples_from_str("num_examples: many\n"), None);
    assert_eq!(read_num_examples_from_str("splits: []\n"), None);
}

#[test]
fn c_bench_006_swe_bench_pro_parquet_without_readme_is_corrupted() {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();

    let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 0);
    assert_eq!(full.data_state, DataState::Corrupted);
}

#[test]
fn c_bench_006_swe_bench_pro_readme_without_parquet_is_corrupted() {
    let root = tempfile::tempdir().unwrap();
    let dataset_dir = root.path().join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
    std::fs::create_dir_all(&dataset_dir).unwrap();
    std::fs::write(dataset_dir.join("README.md"), "num_examples: 731\n").unwrap();

    let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 0);
    assert_eq!(full.data_state, DataState::Corrupted);
}

#[test]
fn c_bench_006_swe_bench_pro_source_data_count_mismatch_is_corrupted() {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();
    std::fs::write(
        root.path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 2\n",
    )
    .unwrap();
    create_source(root.path());

    let inspected = SweBenchProAdapter::with_data_root(Some(root.path())).inspect_data();
    let full = inspected
        .descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.data_state, DataState::Corrupted);
    assert!(inspected.warnings[0].contains("task count mismatch"));
}

#[test]
fn c_bench_006_swe_bench_pro_full_plan_errors_match_data_state() {
    let missing = tempfile::tempdir().unwrap();
    let missing_error = SweBenchProAdapter::with_data_root(Some(missing.path()))
        .plan("full")
        .unwrap_err();
    assert!(missing_error.contains("data_state=requires_auth"));

    let present = tempfile::tempdir().unwrap();
    let data_dir = present
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();
    std::fs::write(
        present
            .path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        "splits:\n- name: test\n  num_examples: 731\n",
    )
    .unwrap();
    let present_error = SweBenchProAdapter::with_data_root(Some(present.path()))
        .plan("full")
        .unwrap_err();
    assert!(present_error.contains("data_state=corrupted"));
}

#[test]
fn c_bench_006_swe_bench_pro_task_uses_external_runner() {
    let root = tempfile::tempdir().unwrap();
    let dataset = SweBenchProDataset {
        task_count: 1,
        data_state: DataState::Ready,
        dataset_dir: root.path().join("dataset"),
        source_dir: root.path().join("source"),
        task_ids: vec!["instance_demo".to_string()],
        warnings: Vec::new(),
    };

    let task = swe_bench_pro_task("instance_demo", &dataset);

    assert_eq!(
        task.external_runner.as_ref().unwrap().kind,
        ExternalRunnerKind::SweBenchPro
    );
    assert!(task.patch_spec.is_some());
}

fn create_source(root: &Path) {
    let source = root.join("_src/SWE-bench_Pro-os");
    std::fs::create_dir_all(source.join("run_scripts/instance_demo")).unwrap();
    std::fs::write(source.join("swe_bench_pro_eval.py"), "").unwrap();
}
