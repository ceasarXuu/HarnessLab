use crate::BenchmarkAdapter;
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, BenchmarkSplit,
    BenchmarkStyle, DataState, ExternalRunnerKind, ExternalRunnerSpec, NetworkPolicy, ResourceHint,
    RunConfigOverrides, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec, WorkspaceSpec,
    WorkspaceType,
};
use std::path::{Path, PathBuf};

pub struct TerminalBenchAdapter {
    data_root: Option<PathBuf>,
}

impl TerminalBenchAdapter {
    pub fn new() -> Self {
        Self { data_root: None }
    }

    pub fn with_data_root(root: Option<&Path>) -> Self {
        Self {
            data_root: root.map(Path::to_path_buf),
        }
    }
}

impl Default for TerminalBenchAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkAdapter for TerminalBenchAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        let dataset = self.data_root.as_deref().map(discover_terminal_bench);
        let smoke_ready = dataset
            .as_ref()
            .is_some_and(|dataset| dataset.task_ids.iter().any(|id| id == "hello-world"));
        BenchmarkDescriptor {
            name: "terminal-bench".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "2.x".to_string(),
            homepage: "https://www.tbench.ai/".to_string(),
            splits: vec![
                BenchmarkSplit {
                    name: "smoke".to_string(),
                    task_count: usize::from(smoke_ready),
                    data_state: if smoke_ready {
                        DataState::Ready
                    } else if dataset
                        .as_ref()
                        .is_some_and(|dataset| dataset.data_state == DataState::Ready)
                    {
                        DataState::Corrupted
                    } else {
                        dataset
                            .as_ref()
                            .map_or(DataState::NotDownloaded, |dataset| dataset.data_state)
                    },
                },
                BenchmarkSplit {
                    name: "full".to_string(),
                    task_count: dataset.as_ref().map_or(0, |dataset| dataset.task_count),
                    data_state: dataset
                        .map_or(DataState::NotDownloaded, |dataset| dataset.data_state),
                },
            ],
        }
    }

    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
        let dataset = self
            .data_root
            .as_deref()
            .map(discover_terminal_bench)
            .ok_or_else(|| "terminal-bench data root is not configured".to_string())?;
        if dataset.data_state != DataState::Ready {
            return Err(format!(
                "terminal-bench data is not runnable: data_state={}",
                dataset.data_state
            ));
        }
        match split {
            "smoke" => {
                if !dataset.task_ids.iter().any(|id| id == "hello-world") {
                    return Err(
                        "terminal-bench smoke requires official hello-world task".to_string()
                    );
                }
                Ok(plan_from_dataset(
                    self.descriptor(),
                    split,
                    &dataset,
                    vec!["hello-world".to_string()],
                ))
            }
            "full" => Ok(plan_from_dataset(
                self.descriptor(),
                split,
                &dataset,
                dataset.task_ids.clone(),
            )),
            other => Err(format!("unknown terminal-bench split {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TerminalBenchDataset {
    task_count: usize,
    data_state: DataState,
    dataset_dir: PathBuf,
    task_ids: Vec<String>,
}

fn discover_terminal_bench(root: &Path) -> TerminalBenchDataset {
    let base = root.join("terminal-bench");
    let Ok(entries) = std::fs::read_dir(base) else {
        return TerminalBenchDataset {
            task_count: 0,
            data_state: DataState::NotDownloaded,
            dataset_dir: root.join("terminal-bench"),
            task_ids: Vec::new(),
        };
    };
    let mut core_dirs = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("terminal-bench-core-")
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    core_dirs.sort();
    let saw_core_dataset = !core_dirs.is_empty();
    let dataset_dir = core_dirs.pop();
    let dataset_dir = dataset_dir.unwrap_or_else(|| root.join("terminal-bench"));
    let dataset_dir = std::fs::canonicalize(&dataset_dir).unwrap_or(dataset_dir);
    let task_ids = collect_task_ids(&dataset_dir);
    let task_count = task_ids.len();
    TerminalBenchDataset {
        task_count,
        data_state: if task_count > 0 {
            DataState::Ready
        } else if saw_core_dataset {
            DataState::Corrupted
        } else {
            DataState::NotDownloaded
        },
        dataset_dir,
        task_ids,
    }
}

fn collect_task_ids(dataset_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dataset_dir) else {
        return Vec::new();
    };
    let mut ids = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter(|entry| entry.path().join("task.yaml").is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn plan_from_dataset(
    descriptor: BenchmarkDescriptor,
    split: &str,
    dataset: &TerminalBenchDataset,
    task_ids: Vec<String>,
) -> BenchmarkPlan {
    let tasks = task_ids
        .into_iter()
        .map(|task_id| terminal_bench_task(&task_id, &dataset.dataset_dir))
        .collect::<Vec<_>>();
    BenchmarkPlan {
        benchmark: BenchmarkIdentity {
            name: descriptor.name,
            version: descriptor.version,
        },
        split: split.to_string(),
        prepared_benchmark_ref: dataset.dataset_dir.display().to_string(),
        tasks,
        run_config_overrides: RunConfigOverrides {
            timeout_sec: None,
            network: Some(NetworkPolicy::Full),
        },
        warnings: Vec::new(),
    }
}

fn terminal_bench_task(task_id: &str, dataset_dir: &Path) -> TaskPlan {
    TaskPlan {
        task_id: task_id.to_string(),
        instruction: format!("Run official Terminal-Bench task {task_id}."),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::Empty,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: SandboxSpec {
            image: "terminal-bench-official".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::Full,
            privileged: false,
            resource_limits: ResourceHint {
                cpu_cores: 2,
                memory_mb: 4096,
            },
        },
        verifier_spec: VerifierSpec {
            command: "tb run".to_string(),
            working_dir: "workspace".to_string(),
            timeout_sec: 3600,
            expected_exit_codes: vec![0],
            environment_mode: VerifierEnvironment::HostProcess,
            output_parser: "terminal_bench_results_json".to_string(),
        },
        artifact_spec: ArtifactSpec {
            base_dir: "workspace".to_string(),
            globs: vec!["**/*".to_string()],
            required_paths: Vec::new(),
            max_size_bytes: 64 * 1024 * 1024,
        },
        patch_spec: None,
        external_runner: Some(ExternalRunnerSpec {
            kind: ExternalRunnerKind::TerminalBench,
            dataset_path: dataset_dir.display().to_string(),
            source_path: None,
        }),
    }
}

#[cfg(test)]
mod tests {
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
}
