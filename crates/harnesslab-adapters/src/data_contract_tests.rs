use crate::{
    BenchmarkAdapter, FakePatchAdapter, FakeTerminalAdapter, SweBenchProAdapter,
    TerminalBenchAdapter, stable_file_checksum,
};
use harnesslab_core::DataState;
use std::path::Path;

#[test]
fn adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache() {
    crate::data_boundary_contract::assert_data_adapter_boundary_contract();
    assert_phase1_coverage_matrix_is_explicit();

    let terminal_root = tempfile::tempdir().unwrap();
    create_terminal_task(terminal_root.path(), "hello-world", "instruction: hi\n");
    let terminal = TerminalBenchAdapter::with_data_root(Some(terminal_root.path()));
    let before = filesystem_fingerprint(terminal_root.path());

    let descriptor = terminal.descriptor();
    let inspected = terminal.inspect_data();
    let after = filesystem_fingerprint(terminal_root.path());

    assert_eq!(before, after);
    assert_eq!(descriptor, inspected.descriptor);
    let expected_cache_path = terminal_root
        .path()
        .join("terminal-bench/terminal-bench-core-0.1.1")
        .canonicalize()
        .unwrap()
        .display()
        .to_string();
    assert_eq!(
        inspected.cache_manifest_path.as_deref(),
        Some(expected_cache_path.as_str())
    );

    let swe_root = tempfile::tempdir().unwrap();
    create_swe_fixture(swe_root.path(), &["instance_b", "instance_a"], 2);
    let swe = SweBenchProAdapter::with_data_root(Some(swe_root.path()));
    let before = filesystem_fingerprint(swe_root.path());

    let descriptor = swe.descriptor();
    let inspected = swe.inspect_data();
    let after = filesystem_fingerprint(swe_root.path());

    assert_eq!(before, after);
    assert_eq!(descriptor, inspected.descriptor);
}

#[test]
fn adapt_data_002_prepare_is_idempotent_and_rejects_unready_data() {
    let root = tempfile::tempdir().unwrap();
    create_terminal_task(root.path(), "hello-world", "instruction: hi\n");
    let adapter = TerminalBenchAdapter::with_data_root(Some(root.path()));

    let first = adapter.prepare("smoke").unwrap();
    let second = adapter.prepare("smoke").unwrap();

    assert_eq!(first, second);
    assert_eq!(first.data_state, DataState::Ready);
    assert_eq!(first.task_count, 1);

    let broken_root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(
        broken_root
            .path()
            .join("terminal-bench/terminal-bench-core-0.1.1/broken"),
    )
    .unwrap();
    let error = TerminalBenchAdapter::with_data_root(Some(broken_root.path()))
        .prepare("full")
        .unwrap_err();
    assert!(error.contains("data_state=corrupted"));

    let partial_root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(partial_root.path().join("terminal-bench")).unwrap();
    std::fs::write(partial_root.path().join("terminal-bench/.partial"), "").unwrap();
    let error = TerminalBenchAdapter::with_data_root(Some(partial_root.path()))
        .prepare("full")
        .unwrap_err();
    assert!(error.contains("data_state=partial"));

    let swe_root = tempfile::tempdir().unwrap();
    create_swe_fixture_without_scripts(swe_root.path(), 1);
    let error = SweBenchProAdapter::with_data_root(Some(swe_root.path()))
        .prepare("smoke")
        .unwrap_err();
    assert!(error.contains("data_state=corrupted"));

    let swe_partial_root = tempfile::tempdir().unwrap();
    let swe_partial_dir = swe_partial_root
        .path()
        .join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
    std::fs::create_dir_all(&swe_partial_dir).unwrap();
    std::fs::write(swe_partial_dir.join(".partial"), "").unwrap();
    let error = SweBenchProAdapter::with_data_root(Some(swe_partial_root.path()))
        .prepare("smoke")
        .unwrap_err();
    assert!(error.contains("data_state=partial"));
}

#[test]
fn adapt_data_003_list_tasks_returns_stable_task_ids_and_source_refs() {
    let root = tempfile::tempdir().unwrap();
    create_terminal_task(root.path(), "z-task", "instruction: z\n");
    create_terminal_task(root.path(), "hello-world", "instruction: hi\n");
    let adapter = TerminalBenchAdapter::with_data_root(Some(root.path()));
    let prepared = adapter.prepare("full").unwrap();

    let first = adapter.list_tasks(&prepared).unwrap();
    let second = adapter.list_tasks(&prepared).unwrap();

    assert_eq!(first, second);
    assert_eq!(task_ids(&first), vec!["hello-world", "z-task"]);
    for task in &first {
        assert_eq!(task.source_ref.benchmark, "terminal-bench");
        assert_eq!(task.source_ref.upstream_id, task.task_id);
        assert!(task.source_ref.checksum.starts_with("fnv64:"));
    }

    let swe_root = tempfile::tempdir().unwrap();
    create_swe_fixture(swe_root.path(), &["instance_z", "instance_a"], 2);
    let swe = SweBenchProAdapter::with_data_root(Some(swe_root.path()));
    let prepared = swe.prepare("full").unwrap();
    let first = swe.list_tasks(&prepared).unwrap();
    let second = swe.list_tasks(&prepared).unwrap();

    assert_eq!(first, second);
    assert_eq!(task_ids(&first), vec!["instance_a", "instance_z"]);
    assert!(
        first
            .iter()
            .all(|task| task.source_ref.benchmark == "swe-bench-pro")
    );

    std::fs::remove_dir_all(
        swe_root
            .path()
            .join("_src/SWE-bench_Pro-os/run_scripts/instance_z"),
    )
    .unwrap();
    let drift_error = swe.list_tasks(&prepared).unwrap_err();
    assert!(drift_error.contains("prepared data drift"));
    let drift_error = swe.create_task_plan(&prepared, &first[0]).unwrap_err();
    assert!(drift_error.contains("prepared data drift"));
    let drift_error = swe.snapshot_task(&prepared, &first[0]).unwrap_err();
    assert!(drift_error.contains("prepared data drift"));
}

#[test]
fn adapt_data_004_snapshot_task_captures_replay_sufficient_identity() {
    let root = tempfile::tempdir().unwrap();
    create_terminal_task(
        root.path(),
        "hello-world",
        "instruction: hi\nmax_test_timeout_sec: 60\n",
    );
    let adapter = TerminalBenchAdapter::with_data_root(Some(root.path()));
    let prepared = adapter.prepare("smoke").unwrap();
    let task = adapter.list_tasks(&prepared).unwrap().remove(0);

    let snapshot = adapter.snapshot_task(&prepared, &task).unwrap();
    let json = serde_json::to_string(&snapshot).unwrap();
    let roundtrip: harnesslab_core::RuntimeTaskSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(roundtrip.benchmark.name, "terminal-bench");
    assert_eq!(roundtrip.split, "smoke");
    assert_eq!(roundtrip.task_id, "hello-world");
    assert_eq!(roundtrip.source_ref, task.source_ref);
    assert_eq!(roundtrip.upstream_metadata_hash, task.source_ref.checksum);
    assert!(roundtrip.instruction_hash.starts_with("fnv64:"));
    assert!(roundtrip.task_plan_hash.starts_with("fnv64:"));
    assert_eq!(
        roundtrip
            .runtime_binding
            .as_ref()
            .unwrap()
            .authority
            .adapter_id
            .as_str(),
        "harnesslab.terminal-bench.runtime"
    );

    let fake_prepared = FakePatchAdapter.prepare("success").unwrap();
    let fake_task = FakePatchAdapter
        .list_tasks(&fake_prepared)
        .unwrap()
        .remove(0);
    let fake_snapshot = FakePatchAdapter
        .snapshot_task(&fake_prepared, &fake_task)
        .unwrap();
    let changed_fake_prepared = FakePatchAdapter.prepare("no-diff").unwrap();
    let changed_fake_task = FakePatchAdapter
        .list_tasks(&changed_fake_prepared)
        .unwrap()
        .remove(0);
    let changed_fake_snapshot = FakePatchAdapter
        .snapshot_task(&changed_fake_prepared, &changed_fake_task)
        .unwrap();
    assert_eq!(fake_snapshot.benchmark.name, "fake-patch");
    assert_eq!(fake_snapshot.task_id, fake_task.task_id);
    assert_eq!(fake_snapshot.source_ref, fake_task.source_ref);
    assert_eq!(
        fake_snapshot.upstream_metadata_hash,
        fake_task.source_ref.checksum
    );
    assert!(fake_snapshot.instruction_hash.starts_with("fnv64:"));
    assert!(fake_snapshot.task_plan_hash.starts_with("fnv64:"));
    assert_ne!(
        fake_snapshot.instruction_hash, changed_fake_snapshot.instruction_hash,
        "FakePatch snapshot must change when patch-style task instructions change"
    );
    assert_ne!(
        fake_snapshot.task_plan_hash, changed_fake_snapshot.task_plan_hash,
        "FakePatch snapshot must change when patch-style task plan content changes"
    );
    assert!(fake_snapshot.external_runner.is_none());

    let swe_root = tempfile::tempdir().unwrap();
    create_swe_fixture(swe_root.path(), &["instance_a"], 1);
    let swe = SweBenchProAdapter::with_data_root(Some(swe_root.path()));
    let prepared = swe.prepare("smoke").unwrap();
    let task = swe.list_tasks(&prepared).unwrap().remove(0);
    let snapshot = swe.snapshot_task(&prepared, &task).unwrap();
    assert_eq!(
        snapshot
            .runtime_binding
            .as_ref()
            .unwrap()
            .authority
            .adapter_id
            .as_str(),
        "harnesslab.swe-bench-pro.runtime"
    );
    let original_hash = snapshot.upstream_metadata_hash;
    std::fs::write(
        swe_root
            .path()
            .join("_src/SWE-bench_Pro-os/swe_bench_pro_eval.py"),
        "b",
    )
    .unwrap();
    let updated_prepared = swe.prepare("smoke").unwrap();
    let updated_task = swe.list_tasks(&updated_prepared).unwrap().remove(0);
    let updated_snapshot = swe.snapshot_task(&updated_prepared, &updated_task).unwrap();
    assert_ne!(updated_snapshot.upstream_metadata_hash, original_hash);
}

#[test]
fn adapt_data_005_create_task_plan_is_stable_and_plan_is_wrapper() {
    let root = tempfile::tempdir().unwrap();
    create_terminal_task(root.path(), "hello-world", "instruction: hi\n");
    create_terminal_task(root.path(), "other-task", "instruction: other\n");
    let adapter = TerminalBenchAdapter::with_data_root(Some(root.path()));
    let prepared = adapter.prepare("full").unwrap();
    let descriptors = adapter.list_tasks(&prepared).unwrap();

    let first = descriptors
        .iter()
        .map(|task| adapter.create_task_plan(&prepared, task).unwrap())
        .collect::<Vec<_>>();
    let second = descriptors
        .iter()
        .map(|task| adapter.create_task_plan(&prepared, task).unwrap())
        .collect::<Vec<_>>();
    let wrapped = adapter.plan("full").unwrap();

    assert_eq!(first, second);
    assert_eq!(wrapped.tasks, first);
    assert_eq!(wrapped.prepared_benchmark_ref, prepared.cache_manifest_path);
    assert_eq!(wrapped.run_config_overrides.timeout_sec, Some(3600));
    let snapshots = descriptors
        .iter()
        .map(|task| adapter.snapshot_task(&prepared, task).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(wrapped.task_runtime_snapshots, snapshots);

    let fake_plan = FakePatchAdapter.plan("success").unwrap();
    let fake_prepared = FakePatchAdapter.prepare("success").unwrap();
    let fake_task = FakePatchAdapter
        .list_tasks(&fake_prepared)
        .unwrap()
        .remove(0);
    let fake_created = FakePatchAdapter
        .create_task_plan(&fake_prepared, &fake_task)
        .unwrap();
    assert_eq!(fake_plan.tasks, vec![fake_created]);

    let fake_terminal_plan = FakeTerminalAdapter.plan("success").unwrap();
    let fake_terminal_prepared = FakeTerminalAdapter.prepare("success").unwrap();
    let fake_terminal_task = FakeTerminalAdapter
        .list_tasks(&fake_terminal_prepared)
        .unwrap()
        .remove(0);
    let fake_terminal_created = FakeTerminalAdapter
        .create_task_plan(&fake_terminal_prepared, &fake_terminal_task)
        .unwrap();
    assert_eq!(fake_terminal_plan.tasks, vec![fake_terminal_created]);

    let swe_root = tempfile::tempdir().unwrap();
    create_swe_fixture(swe_root.path(), &["instance_a", "instance_b"], 2);
    let swe = SweBenchProAdapter::with_data_root(Some(swe_root.path()));
    let swe_prepared = swe.prepare("full").unwrap();
    let swe_descriptors = swe.list_tasks(&swe_prepared).unwrap();
    let swe_created = swe_descriptors
        .iter()
        .map(|task| swe.create_task_plan(&swe_prepared, task).unwrap())
        .collect::<Vec<_>>();
    let swe_wrapped = swe.plan("full").unwrap();
    assert_eq!(swe_wrapped.tasks, swe_created);
    assert_eq!(swe_wrapped.run_config_overrides.timeout_sec, Some(7200));

    std::fs::remove_dir_all(
        swe_root
            .path()
            .join("_src/SWE-bench_Pro-os/run_scripts/instance_b"),
    )
    .unwrap();
    let error = swe.plan("full").unwrap_err();
    assert!(error.contains("data_state=corrupted"));
}

fn assert_phase1_coverage_matrix_is_explicit() {
    let coverage = include_str!(
        "../../../docs/archive/2026-06-15-pre-harbor-webui-redesign/plans/2026-06-04-benchmark-adapter-phase-1-coverage.md"
    );
    let rows: Vec<(&str, Vec<&str>)> = vec![
        (
            "ADAPT-DATA-001",
            vec![
                "descriptor",
                "inspect_data",
                "terminal-bench",
                "swe-bench-pro",
                "module-graph coverage",
                "dependency alias/package assertions",
                "ambient-env/process-state ban",
                "`std::fs` read allowlist",
                "mutable filesystem denylist",
                "production `#[path]` and `include!` bans",
            ],
        ),
        (
            "ADAPT-DATA-002",
            vec![
                "prepare",
                "terminal-bench",
                "swe-bench-pro",
                "idempotent ready prepare",
                "corrupted",
                "partial",
                "auth/missing data states",
                "",
            ],
        ),
        (
            "ADAPT-DATA-003",
            vec![
                "list_tasks",
                "terminal-bench",
                "swe-bench-pro",
                "deterministic task ids",
                "source refs",
                "prepared-data drift is rejected",
                "",
                "",
            ],
        ),
        (
            "ADAPT-DATA-004",
            vec![
                "snapshot_task",
                "terminal-bench",
                "fake-patch",
                "swe-bench-pro",
                "mutation-sensitive patch-style snapshot hashes",
                "upstream metadata hash changes",
                "",
                "",
            ],
        ),
        (
            "ADAPT-DATA-005",
            vec![
                "create_task_plan",
                "plan(split)",
                "fake-terminal",
                "fake-patch",
                "terminal-bench",
                "swe-bench-pro",
                "wrapper equivalence",
                "source/data skew after drift",
            ],
        ),
    ];

    for (id, expected_terms) in rows {
        let row = coverage
            .lines()
            .find(|line| line.contains(id))
            .unwrap_or_else(|| panic!("coverage matrix missing {id}"));
        for term in expected_terms {
            if term.is_empty() {
                continue;
            }
            assert!(
                row.contains(term),
                "coverage matrix row for {id} missing {term}"
            );
        }
    }

    for required in [
        "docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md",
        "crates/harnesslab-adapters/src/lib.rs",
        "crates/harnesslab-adapters/Cargo.toml",
        "fake adapter source files",
        "boundary helper",
        "scan helper",
        "production helper modules are covered",
    ] {
        assert!(
            coverage.contains(required),
            "coverage matrix missing {required}"
        );
    }
}

fn create_terminal_task(root: &Path, task_id: &str, task_yaml: &str) {
    let task_dir = root
        .join("terminal-bench/terminal-bench-core-0.1.1")
        .join(task_id);
    std::fs::create_dir_all(&task_dir).unwrap();
    std::fs::write(task_dir.join("task.yaml"), task_yaml).unwrap();
}

fn create_swe_fixture(root: &Path, task_ids: &[&str], task_count: usize) {
    create_swe_data(root, task_count);
    let source = root.join("_src/SWE-bench_Pro-os");
    for task_id in task_ids {
        std::fs::create_dir_all(source.join("run_scripts").join(task_id)).unwrap();
    }
    std::fs::write(source.join("swe_bench_pro_eval.py"), "a").unwrap();
}

fn create_swe_fixture_without_scripts(root: &Path, task_count: usize) {
    create_swe_data(root, task_count);
    let source = root.join("_src/SWE-bench_Pro-os");
    std::fs::create_dir_all(source.join("run_scripts")).unwrap();
    std::fs::write(source.join("swe_bench_pro_eval.py"), "a").unwrap();
}

fn create_swe_data(root: &Path, task_count: usize) {
    let data_dir = root.join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();
    std::fs::write(
        root.join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
        format!("splits:\n- name: test\n  num_examples: {task_count}\n"),
    )
    .unwrap();
}

fn filesystem_fingerprint(root: &Path) -> Vec<String> {
    let mut entries = Vec::new();
    collect_fingerprint(root, root, &mut entries);
    entries.sort();
    entries
}

fn collect_fingerprint(root: &Path, current: &Path, entries: &mut Vec<String>) {
    let Ok(children) = std::fs::read_dir(current) else {
        return;
    };
    for child in children.filter_map(Result::ok) {
        let path = child.path();
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let metadata = child.metadata().unwrap();
        let kind = if metadata.is_dir() { "dir" } else { "file" };
        let checksum = if metadata.is_file() {
            stable_file_checksum(&path)
        } else {
            "dir".to_string()
        };
        entries.push(format!(
            "{}:{}:{}:{}",
            relative.display(),
            kind,
            metadata.len(),
            checksum
        ));
        if metadata.is_dir() {
            collect_fingerprint(root, &path, entries);
        }
    }
}

fn task_ids(tasks: &[harnesslab_core::TaskDescriptor]) -> Vec<&str> {
    tasks
        .iter()
        .map(|task| task.task_id.as_str())
        .collect::<Vec<_>>()
}
