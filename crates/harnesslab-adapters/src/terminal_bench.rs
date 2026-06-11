use crate::{
    BenchmarkAdapter, built_in_protocol_registry, prepared_with_identity, stable_file_checksum,
    terminal_bench_protocol_descriptor,
};
use harnesslab_core::{
    AdapterId, ArtifactSpec, BenchmarkDataState, BenchmarkDescriptor, BenchmarkSplit,
    BenchmarkStyle, DataState, ExternalRunnerKind, ExternalRunnerSpec, NetworkPolicy,
    PreparedBenchmark, ResourceHint, RunConfigOverrides, SandboxSpec, SourceRef, TaskDescriptor,
    TaskPlan, TaskRuntimeBinding, VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
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

    fn protocol_descriptor(&self) -> Option<crate::ProtocolAdapterDescriptor> {
        Some(terminal_bench_protocol_descriptor(self.descriptor()))
    }

    fn inspect_data(&self) -> BenchmarkDataState {
        let dataset = self.data_root.as_deref().map(discover_terminal_bench);
        BenchmarkDataState {
            descriptor: self.descriptor(),
            cache_manifest_path: dataset.map(|dataset| dataset.dataset_dir.display().to_string()),
            warnings: Vec::new(),
        }
    }

    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String> {
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
        let task_ids = task_ids_for_split(split, &dataset)?;
        Ok(prepared_with_identity(
            self.descriptor(),
            split,
            dataset.dataset_dir.display().to_string(),
            task_ids.len(),
            None,
            task_ids,
            None,
        ))
    }

    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String> {
        let dataset_dir = PathBuf::from(&prepared.cache_manifest_path);
        let task_ids = if prepared.selected_task_ids.is_empty() {
            collect_task_ids(&dataset_dir)
        } else {
            prepared.selected_task_ids.clone()
        };
        let dataset = TerminalBenchDataset {
            task_count: task_ids.len(),
            data_state: prepared.data_state,
            task_ids,
            dataset_dir,
        };
        task_ids_for_split(&prepared.split, &dataset)?
            .into_iter()
            .map(|task_id| {
                terminal_bench_task_descriptor(&prepared.split, &task_id, &dataset.dataset_dir)
            })
            .collect()
    }

    fn create_task_plan(
        &self,
        prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String> {
        let dataset_dir = PathBuf::from(&prepared.cache_manifest_path);
        if !dataset_dir.join(&task.task_id).join("task.yaml").is_file() {
            return Err(format!(
                "terminal-bench task {} is missing task.yaml",
                task.task_id
            ));
        }
        Ok(terminal_bench_task(&task.task_id, &dataset_dir))
    }

    fn run_config_overrides(&self, _prepared: &PreparedBenchmark) -> RunConfigOverrides {
        RunConfigOverrides {
            timeout_sec: Some(3600),
            network: Some(NetworkPolicy::Full),
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
    if base.join(".partial").is_file() {
        return TerminalBenchDataset {
            task_count: 0,
            data_state: DataState::Partial,
            dataset_dir: base,
            task_ids: Vec::new(),
        };
    }
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

fn task_ids_for_split(split: &str, dataset: &TerminalBenchDataset) -> Result<Vec<String>, String> {
    match split {
        "smoke" => {
            if !dataset.task_ids.iter().any(|id| id == "hello-world") {
                return Err("terminal-bench smoke requires official hello-world task".to_string());
            }
            Ok(vec!["hello-world".to_string()])
        }
        "full" => Ok(dataset.task_ids.clone()),
        other => Err(format!("unknown terminal-bench split {other}")),
    }
}

fn terminal_bench_task_descriptor(
    split: &str,
    task_id: &str,
    dataset_dir: &Path,
) -> Result<TaskDescriptor, String> {
    let task_dir = dataset_dir.join(task_id);
    if !task_dir.join("task.yaml").is_file() {
        return Err(format!(
            "terminal-bench task {task_id} is missing task.yaml"
        ));
    }
    let metadata = read_task_metadata(&task_dir);
    Ok(TaskDescriptor {
        task_id: task_id.to_string(),
        split: split.to_string(),
        estimated_timeout_sec: metadata.max_test_timeout_sec.unwrap_or(3600),
        resource_hint: ResourceHint {
            cpu_cores: 2,
            memory_mb: 4096,
        },
        source_ref: SourceRef {
            benchmark: "terminal-bench".to_string(),
            upstream_id: task_id.to_string(),
            checksum: stable_file_checksum(&task_dir.join("task.yaml")),
        },
    })
}

fn terminal_bench_task(task_id: &str, dataset_dir: &Path) -> TaskPlan {
    let metadata = read_task_metadata(&dataset_dir.join(task_id));
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
            timeout_sec: metadata.max_test_timeout_sec.unwrap_or(3600),
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
            agent_timeout_sec: metadata.max_agent_timeout_sec,
        }),
        runtime_binding: Some(terminal_bench_runtime_binding(task_id, dataset_dir)),
    }
}

fn terminal_bench_runtime_binding(task_id: &str, dataset_dir: &Path) -> TaskRuntimeBinding {
    TaskRuntimeBinding {
        authority: built_in_protocol_registry()
            .binding_for_adapter_id(
                &AdapterId::new("harnesslab.terminal-bench.runtime")
                    .expect("terminal-bench adapter id is valid"),
            )
            .expect("terminal-bench protocol binding is registered")
            .authority(),
        dataset_ref: dataset_dir.display().to_string(),
        task_ref: task_id.to_string(),
        artifact_contract_id: "artifact.basic.v1".to_string(),
        readiness_contract_id: "readiness.basic.v1".to_string(),
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TerminalBenchTaskMetadata {
    max_agent_timeout_sec: Option<u64>,
    max_test_timeout_sec: Option<u64>,
}

fn read_task_metadata(task_dir: &Path) -> TerminalBenchTaskMetadata {
    let Ok(content) = std::fs::read_to_string(task_dir.join("task.yaml")) else {
        return TerminalBenchTaskMetadata::default();
    };
    TerminalBenchTaskMetadata {
        max_agent_timeout_sec: parse_yaml_timeout_field(&content, "max_agent_timeout_sec"),
        max_test_timeout_sec: parse_yaml_timeout_field(&content, "max_test_timeout_sec"),
    }
}

fn parse_yaml_timeout_field(content: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key}:");
    content.lines().find_map(|line| {
        if line.starts_with(char::is_whitespace) {
            return None;
        }
        let line = line.split('#').next().unwrap_or("").trim();
        let value = line.strip_prefix(&prefix)?.trim();
        parse_timeout_seconds(value)
    })
}

fn parse_timeout_seconds(value: &str) -> Option<u64> {
    let token = value.split_whitespace().next()?.trim_matches('"');
    if let Ok(seconds) = token.parse::<u64>() {
        return (seconds > 0).then_some(seconds);
    }
    let seconds = token.parse::<f64>().ok()?;
    (seconds.is_finite() && seconds > 0.0).then(|| seconds.ceil() as u64)
}

#[cfg(test)]
#[path = "terminal_bench_tests.rs"]
mod tests;
