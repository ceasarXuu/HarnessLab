use harnesslab_core::{
    BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, RunConfigOverrides, TaskPlan,
};

pub trait BenchmarkAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor;
    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String>;
}

pub fn built_in_descriptors() -> Vec<BenchmarkDescriptor> {
    vec![
        crate::FakeTerminalAdapter.descriptor(),
        crate::FakePatchAdapter.descriptor(),
        crate::TerminalBenchAdapter.descriptor(),
        crate::SweBenchProAdapter.descriptor(),
    ]
}

pub fn adapter_for(name: &str) -> Option<Box<dyn BenchmarkAdapter>> {
    match name {
        "fake-terminal" => Some(Box::new(crate::FakeTerminalAdapter)),
        "fake-patch" => Some(Box::new(crate::FakePatchAdapter)),
        "terminal-bench" => Some(Box::new(crate::TerminalBenchAdapter)),
        "swe-bench-pro" => Some(Box::new(crate::SweBenchProAdapter)),
        _ => None,
    }
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

        assert!(names.contains(&"fake-terminal".to_string()));
        assert!(names.contains(&"fake-patch".to_string()));
        assert!(names.contains(&"terminal-bench".to_string()));
        assert!(names.contains(&"swe-bench-pro".to_string()));
    }

    #[test]
    fn c_bench_004_required_external_smoke_adapters_are_available() {
        let terminal = adapter_for("terminal-bench")
            .unwrap()
            .plan("smoke")
            .unwrap();
        let swe = adapter_for("swe-bench-pro").unwrap().plan("smoke").unwrap();

        assert_eq!(terminal.tasks.len(), 1);
        assert_eq!(terminal.tasks[0].sandbox_spec.image, "ubuntu:24.04");
        assert!(terminal.tasks[0].patch_spec.is_none());
        assert_eq!(swe.tasks.len(), 1);
        assert!(swe.tasks[0].patch_spec.is_some());
    }
}
