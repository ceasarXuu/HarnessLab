use super::*;

#[test]
fn c_bench_005_terminal_bench_full_reports_local_data_as_ready() {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();

    let descriptor = TerminalBenchAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 1);
    assert_eq!(full.data_state, DataState::Ready);
}

#[test]
fn c_bench_005_terminal_bench_full_reports_missing_data() {
    let root = tempfile::tempdir().unwrap();

    let descriptor = TerminalBenchAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 0);
    assert_eq!(full.data_state, DataState::NotDownloaded);
}

#[test]
fn c_bench_005_terminal_bench_full_reports_malformed_data_as_corrupted() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(
        root.path()
            .join("terminal-bench/terminal-bench-core-0.1.1/broken-task"),
    )
    .unwrap();

    let descriptor = TerminalBenchAdapter::with_data_root(Some(root.path())).descriptor();
    let full = descriptor
        .splits
        .iter()
        .find(|split| split.name == "full")
        .unwrap();

    assert_eq!(full.task_count, 0);
    assert_eq!(full.data_state, DataState::Corrupted);
}

#[test]
fn c_bench_005_terminal_bench_chooses_core_dataset_deterministically() {
    let root = tempfile::tempdir().unwrap();
    let old_task = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/old-task");
    let new_task = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.2.0/hello-world");
    std::fs::create_dir_all(&old_task).unwrap();
    std::fs::create_dir_all(&new_task).unwrap();
    std::fs::write(old_task.join("task.yaml"), "instruction: old").unwrap();
    std::fs::write(new_task.join("task.yaml"), "instruction: hi").unwrap();

    let plan = TerminalBenchAdapter::with_data_root(Some(root.path()))
        .plan("smoke")
        .unwrap();

    assert!(
        plan.prepared_benchmark_ref
            .contains("terminal-bench-core-0.2.0")
    );
}

#[test]
fn c_bench_005_terminal_bench_smoke_requires_hello_world() {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/other-task");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();

    let descriptor = TerminalBenchAdapter::with_data_root(Some(root.path())).descriptor();
    let smoke = descriptor
        .splits
        .iter()
        .find(|split| split.name == "smoke")
        .unwrap();

    assert_eq!(smoke.data_state, DataState::Corrupted);
}

#[test]
fn c_bench_005_terminal_bench_full_plan_errors_match_data_state() {
    let missing = tempfile::tempdir().unwrap();
    let missing_error = TerminalBenchAdapter::with_data_root(Some(missing.path()))
        .plan("full")
        .unwrap_err();
    assert!(missing_error.contains("data_state=not_downloaded"));

    let present = tempfile::tempdir().unwrap();
    let task_dir = present
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();
    let present_plan = TerminalBenchAdapter::with_data_root(Some(present.path()))
        .plan("full")
        .unwrap();
    assert_eq!(present_plan.tasks[0].task_id, "hello-world");
    assert!(present_plan.tasks[0].external_runner.is_some());
}

#[test]
fn c_bench_006_terminal_bench_maps_task_test_timeout() {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(
        task_dir.join("task.yaml"),
        "instruction: hi\nmax_agent_timeout_sec: 360.0\nmax_test_timeout_sec: 60.0\n",
    )
    .unwrap();

    let plan = TerminalBenchAdapter::with_data_root(Some(root.path()))
        .plan("full")
        .unwrap();

    assert_eq!(plan.tasks[0].verifier_spec.timeout_sec, 60);
    assert_eq!(
        plan.tasks[0]
            .external_runner
            .as_ref()
            .unwrap()
            .agent_timeout_sec,
        Some(360)
    );
}

#[test]
fn c_bench_006_terminal_bench_ignores_timeout_text_inside_block_scalars() {
    let root = tempfile::tempdir().unwrap();
    let task_dir = root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1/timeout-task");
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(
        task_dir.join("task.yaml"),
        "instruction: |\n  max_test_timeout_sec: 3\nmax_agent_timeout_sec: 12.2\n",
    )
    .unwrap();

    let plan = TerminalBenchAdapter::with_data_root(Some(root.path()))
        .plan("full")
        .unwrap();
    let task = plan.tasks.first().unwrap();

    assert_eq!(task.verifier_spec.timeout_sec, 3600);
    assert_eq!(
        task.external_runner.as_ref().unwrap().agent_timeout_sec,
        Some(13)
    );
}
