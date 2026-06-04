use harnesslab_core::{
    BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, RunConfigOverrides, TaskPlan,
};
use std::path::Path;

pub trait BenchmarkAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor;
    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String>;
}

pub fn built_in_descriptors() -> Vec<BenchmarkDescriptor> {
    let mut descriptors = production_descriptors(None);
    if internal_fixtures_enabled() {
        descriptors.splice(0..0, fixture_descriptors());
    }
    descriptors
}

pub fn production_descriptors(root: Option<&Path>) -> Vec<BenchmarkDescriptor> {
    vec![
        crate::TerminalBenchAdapter::with_data_root(root).descriptor(),
        crate::SweBenchProAdapter::with_data_root(root).descriptor(),
    ]
}

pub fn built_in_descriptors_with_root(root: Option<&Path>) -> Vec<BenchmarkDescriptor> {
    let mut descriptors = production_descriptors(root);
    if internal_fixtures_enabled() {
        descriptors.splice(0..0, fixture_descriptors());
    }
    descriptors
}

fn fixture_descriptors() -> Vec<BenchmarkDescriptor> {
    vec![
        crate::FakeTerminalAdapter.descriptor(),
        crate::FakePatchAdapter.descriptor(),
    ]
}

pub fn adapter_for(name: &str) -> Option<Box<dyn BenchmarkAdapter>> {
    adapter_for_with_root(name, None)
}

pub fn adapter_for_with_root(name: &str, root: Option<&Path>) -> Option<Box<dyn BenchmarkAdapter>> {
    match name {
        "fake-terminal" => Some(Box::new(crate::FakeTerminalAdapter)),
        "fake-patch" => Some(Box::new(crate::FakePatchAdapter)),
        "terminal-bench" => Some(Box::new(crate::TerminalBenchAdapter::with_data_root(root))),
        "swe-bench-pro" => Some(Box::new(crate::SweBenchProAdapter::with_data_root(root))),
        _ => None,
    }
}

fn internal_fixtures_enabled() -> bool {
    std::env::var("HARNESSLAB_ENABLE_FAKE_BENCHMARKS").as_deref() == Ok("1")
}

pub fn plan_from_tasks(
    descriptor: BenchmarkDescriptor,
    split: &str,
    tasks: Vec<TaskPlan>,
) -> BenchmarkPlan {
    BenchmarkPlan {
        benchmark: BenchmarkIdentity {
            name: descriptor.name,
            version: descriptor.version,
        },
        split: split.to_string(),
        prepared_benchmark_ref: "built-in-fixture".to_string(),
        tasks,
        run_config_overrides: RunConfigOverrides {
            timeout_sec: None,
            network: None,
        },
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_bench_001_built_in_descriptors_include_required_benchmarks() {
        let names = built_in_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.name)
            .collect::<Vec<_>>();

        assert!(names.contains(&"terminal-bench".to_string()));
        assert!(names.contains(&"swe-bench-pro".to_string()));
    }

    #[test]
    fn c_bench_001_fake_descriptors_are_internal_only() {
        let names = built_in_descriptors()
            .into_iter()
            .map(|descriptor| descriptor.name)
            .collect::<Vec<_>>();

        assert!(!names.contains(&"fake-terminal".to_string()));
        assert!(!names.contains(&"fake-patch".to_string()));
    }

    #[test]
    fn c_bench_004_required_external_smoke_adapters_are_available() {
        let root = tempfile::tempdir().unwrap();
        let task_dir = root
            .path()
            .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
        std::fs::create_dir_all(&task_dir).unwrap();
        std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();
        let terminal = adapter_for_with_root("terminal-bench", Some(root.path()))
            .unwrap()
            .plan("smoke")
            .unwrap();
        let swe = adapter_for("swe-bench-pro").unwrap().descriptor();

        assert_eq!(terminal.tasks.len(), 1);
        assert_eq!(
            terminal.tasks[0].external_runner.as_ref().unwrap().kind,
            harnesslab_core::ExternalRunnerKind::TerminalBench
        );
        assert!(terminal.tasks[0].patch_spec.is_none());
        assert_eq!(swe.name, "swe-bench-pro");
    }

    #[test]
    fn adapt_data_000_current_benchmark_adapter_gap_is_explicit() {
        let source = include_str!("registry.rs");
        let trait_source = source.split("pub fn built_in_descriptors").next().unwrap();

        assert!(trait_source.contains("pub trait BenchmarkAdapter"));
        assert!(
            !trait_source.contains("fn inspect_data("),
            "Phase 1 must replace this gap sentinel with ADAPT-DATA-001"
        );
        assert!(
            !trait_source.contains("fn prepare("),
            "Phase 1 must replace this gap sentinel with ADAPT-DATA-002"
        );
        assert!(
            !trait_source.contains("fn list_tasks("),
            "Phase 1 must replace this gap sentinel with ADAPT-DATA-003"
        );
        assert!(
            !trait_source.contains("fn create_task_plan("),
            "Phase 1 must replace this gap sentinel with ADAPT-DATA-005"
        );
        assert!(
            !trait_source.contains("fn snapshot_task("),
            "Phase 1 must replace this gap sentinel with ADAPT-DATA-004"
        );
    }
}
