use harnesslab_core::{
    BenchmarkDataState, BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, PreparedBenchmark,
    RunConfigOverrides, RuntimeTaskSnapshot, TaskDescriptor, TaskPlan,
};
use std::path::Path;

pub trait BenchmarkAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor;
    fn inspect_data(&self) -> BenchmarkDataState {
        BenchmarkDataState {
            descriptor: self.descriptor(),
            cache_manifest_path: None,
            warnings: Vec::new(),
        }
    }
    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String>;
    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String>;
    fn create_task_plan(
        &self,
        prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String>;
    fn run_config_overrides(&self, _prepared: &PreparedBenchmark) -> RunConfigOverrides {
        RunConfigOverrides {
            timeout_sec: None,
            network: None,
        }
    }
    fn snapshot_task(
        &self,
        prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<RuntimeTaskSnapshot, String> {
        let task_plan = self.create_task_plan(prepared, task)?;
        snapshot_from_task_plan(prepared, task, &task_plan)
    }
    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
        let prepared = self.prepare(split)?;
        let task_descriptors = self.list_tasks(&prepared)?;
        let mut tasks = Vec::with_capacity(task_descriptors.len());
        let mut task_runtime_snapshots = Vec::with_capacity(task_descriptors.len());
        for task in &task_descriptors {
            let task_plan = self.create_task_plan(&prepared, task)?;
            let task_snapshot = snapshot_from_task_plan(&prepared, task, &task_plan)?;
            tasks.push(task_plan);
            task_runtime_snapshots.push(task_snapshot);
        }
        Ok(BenchmarkPlan {
            benchmark: BenchmarkIdentity {
                name: prepared.descriptor.name.clone(),
                version: prepared.descriptor.version.clone(),
            },
            split: prepared.split.clone(),
            prepared_benchmark_ref: prepared.cache_manifest_path.clone(),
            tasks,
            task_runtime_snapshots,
            run_config_overrides: self.run_config_overrides(&prepared),
            warnings: prepared.warnings.clone(),
        })
    }
}

fn snapshot_from_task_plan(
    prepared: &PreparedBenchmark,
    task: &TaskDescriptor,
    task_plan: &TaskPlan,
) -> Result<RuntimeTaskSnapshot, String> {
    Ok(RuntimeTaskSnapshot {
        benchmark: BenchmarkIdentity {
            name: prepared.descriptor.name.clone(),
            version: prepared.descriptor.version.clone(),
        },
        split: prepared.split.clone(),
        task_id: task.task_id.clone(),
        source_ref: task.source_ref.clone(),
        upstream_metadata_hash: task.source_ref.checksum.clone(),
        instruction_hash: stable_checksum(&task_plan.instruction),
        task_plan_hash: stable_task_plan_hash(task_plan)?,
        external_runner: task_plan.external_runner.clone(),
        runtime_binding: task_plan.runtime_binding.clone(),
        external_runtime_attempts: Vec::new(),
    })
}

pub fn built_in_descriptors() -> Vec<BenchmarkDescriptor> {
    production_descriptors(None)
}

pub fn production_descriptors(root: Option<&Path>) -> Vec<BenchmarkDescriptor> {
    vec![
        crate::TerminalBenchAdapter::with_data_root(root).descriptor(),
        crate::SweBenchProAdapter::with_data_root(root).descriptor(),
    ]
}

pub fn built_in_descriptors_with_root(root: Option<&Path>) -> Vec<BenchmarkDescriptor> {
    production_descriptors(root)
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
        task_runtime_snapshots: Vec::new(),
        run_config_overrides: RunConfigOverrides {
            timeout_sec: None,
            network: None,
        },
        warnings: Vec::new(),
    }
}

pub(crate) fn prepared_from_descriptor(
    descriptor: BenchmarkDescriptor,
    split: &str,
    cache_manifest_path: String,
    task_count: usize,
) -> PreparedBenchmark {
    prepared_with_identity(
        descriptor,
        split,
        cache_manifest_path,
        task_count,
        None,
        Vec::new(),
        None,
    )
}

pub(crate) fn prepared_with_identity(
    descriptor: BenchmarkDescriptor,
    split: &str,
    cache_manifest_path: String,
    task_count: usize,
    source_manifest_path: Option<String>,
    selected_task_ids: Vec<String>,
    data_snapshot_hash: Option<String>,
) -> PreparedBenchmark {
    PreparedBenchmark {
        descriptor,
        split: split.to_string(),
        data_state: harnesslab_core::DataState::Ready,
        prepared_at: "deterministic-adapter-prepare".to_string(),
        task_count,
        cache_manifest_path,
        source_manifest_path,
        selected_task_ids,
        data_snapshot_hash,
        size_bytes: 0,
        warnings: Vec::new(),
    }
}

pub(crate) fn stable_checksum(value: &str) -> String {
    stable_checksum_bytes(value.as_bytes())
}

pub(crate) fn stable_file_checksum(path: &Path) -> String {
    match std::fs::read(path) {
        Ok(bytes) => stable_checksum_bytes(&bytes),
        Err(_) => stable_checksum(&format!("missing:{}", path.display())),
    }
}

fn stable_task_plan_hash(task_plan: &TaskPlan) -> Result<String, String> {
    let bytes = serde_json::to_vec(task_plan)
        .map_err(|error| format!("failed to serialize task plan for snapshot: {error}"))?;
    Ok(stable_checksum_bytes(&bytes))
}

fn stable_checksum_bytes(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv64:{hash:016x}")
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
}
