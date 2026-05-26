use harnesslab_core::{
    BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle,
    DataState, RunConfigOverrides, TaskPlan,
};

pub trait BenchmarkAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor;
    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String>;
}

pub fn built_in_descriptors() -> Vec<BenchmarkDescriptor> {
    vec![
        crate::FakeTerminalAdapter.descriptor(),
        crate::FakePatchAdapter.descriptor(),
        external_descriptor(
            "terminal-bench",
            BenchmarkStyle::Terminal,
            "2.x",
            "https://terminalbench.lol/",
            DataState::NotDownloaded,
        ),
        external_descriptor(
            "swe-bench-pro",
            BenchmarkStyle::Patch,
            "2026",
            "https://www.swebench.com/",
            DataState::RequiresAuth,
        ),
    ]
}

pub fn adapter_for(name: &str) -> Option<Box<dyn BenchmarkAdapter>> {
    match name {
        "fake-terminal" => Some(Box::new(crate::FakeTerminalAdapter)),
        "fake-patch" => Some(Box::new(crate::FakePatchAdapter)),
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

fn external_descriptor(
    name: &str,
    style: BenchmarkStyle,
    version: &str,
    homepage: &str,
    state: DataState,
) -> BenchmarkDescriptor {
    BenchmarkDescriptor {
        name: name.to_string(),
        style,
        version: version.to_string(),
        homepage: homepage.to_string(),
        splits: vec![
            BenchmarkSplit {
                name: "smoke".to_string(),
                task_count: 1,
                data_state: state,
            },
            BenchmarkSplit {
                name: "full".to_string(),
                task_count: 0,
                data_state: state,
            },
        ],
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
}
