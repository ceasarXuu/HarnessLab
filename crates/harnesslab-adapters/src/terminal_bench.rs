use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec,
    WorkspaceSpec, WorkspaceType,
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
        BenchmarkDescriptor {
            name: "terminal-bench".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "2.x".to_string(),
            homepage: "https://www.tbench.ai/".to_string(),
            splits: vec![
                BenchmarkSplit {
                    name: "smoke".to_string(),
                    task_count: 1,
                    data_state: DataState::Ready,
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
        match split {
            "smoke" => Ok(plan_from_tasks(
                self.descriptor(),
                split,
                vec![TaskPlan {
                    task_id: "terminal-bench-smoke".to_string(),
                    instruction: "Create result.txt with exactly: terminal-bench-smoke".to_string(),
                    workspace_spec: WorkspaceSpec {
                        workspace_type: WorkspaceType::Empty,
                        target_path: "workspace".to_string(),
                        clean: true,
                    },
                    sandbox_spec: SandboxSpec {
                        image: "ubuntu:24.04".to_string(),
                        mounts: Vec::new(),
                        env_vars: Vec::new(),
                        network: NetworkPolicy::None,
                        privileged: false,
                        resource_limits: ResourceHint {
                            cpu_cores: 1,
                            memory_mb: 512,
                        },
                    },
                    verifier_spec: VerifierSpec {
                        command: "test \"$(cat result.txt 2>/dev/null)\" = terminal-bench-smoke"
                            .to_string(),
                        working_dir: "workspace".to_string(),
                        timeout_sec: 30,
                        expected_exit_codes: vec![0],
                        environment_mode: VerifierEnvironment::HostProcess,
                        output_parser: "exit_code".to_string(),
                    },
                    artifact_spec: ArtifactSpec {
                        base_dir: "workspace".to_string(),
                        globs: vec!["**/*".to_string()],
                        required_paths: Vec::new(),
                        max_size_bytes: 1024 * 1024,
                    },
                    patch_spec: None,
                }],
            )),
            "full" => {
                let state = self
                    .data_root
                    .as_deref()
                    .map(discover_terminal_bench)
                    .map_or(DataState::NotDownloaded, |dataset| dataset.data_state);
                if state == DataState::Unsupported {
                    Err(
                        "terminal-bench full data is present, but official task execution is not implemented yet"
                            .to_string(),
                    )
                } else if state == DataState::Corrupted {
                    Err("terminal-bench full data is present but invalid or unreadable".to_string())
                } else {
                    Err(
                        "terminal-bench full data is not available locally; download it first"
                            .to_string(),
                    )
                }
            }
            other => Err(format!("unknown terminal-bench split {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TerminalBenchDataset {
    task_count: usize,
    data_state: DataState,
}

fn discover_terminal_bench(root: &Path) -> TerminalBenchDataset {
    let base = root.join("terminal-bench");
    let Ok(entries) = std::fs::read_dir(base) else {
        return TerminalBenchDataset {
            task_count: 0,
            data_state: DataState::NotDownloaded,
        };
    };
    let mut task_count = 0;
    let mut saw_core_dataset = false;
    for entry in entries.filter_map(Result::ok) {
        if !entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
            continue;
        }
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("terminal-bench-core-")
        {
            continue;
        }
        saw_core_dataset = true;
        task_count += count_task_yaml(&entry.path());
    }
    TerminalBenchDataset {
        task_count,
        data_state: if task_count > 0 {
            DataState::Unsupported
        } else if saw_core_dataset {
            DataState::Corrupted
        } else {
            DataState::NotDownloaded
        },
    }
}

fn count_task_yaml(dataset_dir: &Path) -> usize {
    let Ok(entries) = std::fs::read_dir(dataset_dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter(|entry| entry.path().join("task.yaml").is_file())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_bench_005_terminal_bench_full_reports_local_data_as_unsupported() {
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
        assert_eq!(full.data_state, DataState::Unsupported);
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
    fn c_bench_005_terminal_bench_full_plan_errors_match_data_state() {
        let missing = tempfile::tempdir().unwrap();
        let missing_error = TerminalBenchAdapter::with_data_root(Some(missing.path()))
            .plan("full")
            .unwrap_err();
        assert!(missing_error.contains("not available locally"));

        let present = tempfile::tempdir().unwrap();
        let task_dir = present
            .path()
            .join("terminal-bench/terminal-bench-core-0.1.1/hello-world");
        std::fs::create_dir_all(&task_dir).unwrap();
        std::fs::write(task_dir.join("task.yaml"), "instruction: hi").unwrap();
        let present_error = TerminalBenchAdapter::with_data_root(Some(present.path()))
            .plan("full")
            .unwrap_err();
        assert!(present_error.contains("data is present"));
    }
}
