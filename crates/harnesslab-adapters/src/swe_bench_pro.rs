use crate::{
    BenchmarkAdapter, built_in_protocol_registry, prepared_with_identity, stable_checksum,
    stable_file_checksum, swe_bench_pro_protocol_descriptor,
};
use harnesslab_core::{
    AdapterId, ArtifactSpec, BenchmarkDataState, BenchmarkDescriptor, BenchmarkSplit,
    BenchmarkStyle, DataState, ExternalRunnerSpec, NetworkPolicy, PatchSpec, PreparedBenchmark,
    ResourceHint, RunConfigOverrides, SandboxSpec, SourceRef, TaskDescriptor, TaskPlan,
    TaskRuntimeBinding, VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
};
use std::path::{Path, PathBuf};

pub struct SweBenchProAdapter {
    data_root: Option<PathBuf>,
}

impl SweBenchProAdapter {
    pub fn new() -> Self {
        Self { data_root: None }
    }

    pub fn with_data_root(root: Option<&Path>) -> Self {
        Self {
            data_root: root.map(Path::to_path_buf),
        }
    }
}

impl Default for SweBenchProAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BenchmarkAdapter for SweBenchProAdapter {
    fn descriptor(&self) -> BenchmarkDescriptor {
        let dataset = self.data_root.as_deref().map(discover_swe_bench_pro);
        let ready = dataset
            .as_ref()
            .is_some_and(|dataset| dataset.data_state == DataState::Ready);
        BenchmarkDescriptor {
            name: "swe-bench-pro".to_string(),
            style: BenchmarkStyle::Patch,
            version: "2026".to_string(),
            homepage: "https://www.swebench.com/".to_string(),
            splits: vec![
                BenchmarkSplit {
                    name: "smoke".to_string(),
                    task_count: usize::from(ready),
                    data_state: dataset
                        .as_ref()
                        .map_or(DataState::RequiresAuth, |dataset| dataset.data_state),
                },
                BenchmarkSplit {
                    name: "full".to_string(),
                    task_count: dataset.as_ref().map_or(0, |dataset| dataset.task_count),
                    data_state: dataset
                        .map_or(DataState::RequiresAuth, |dataset| dataset.data_state),
                },
            ],
        }
    }

    fn protocol_descriptor(&self) -> Option<crate::ProtocolAdapterDescriptor> {
        Some(swe_bench_pro_protocol_descriptor(self.descriptor()))
    }

    fn inspect_data(&self) -> BenchmarkDataState {
        let dataset = self.data_root.as_deref().map(discover_swe_bench_pro);
        let warnings = dataset
            .as_ref()
            .map_or_else(Vec::new, |dataset| dataset.warnings.clone());
        BenchmarkDataState {
            descriptor: self.descriptor(),
            cache_manifest_path: dataset.map(|dataset| dataset.dataset_dir.display().to_string()),
            warnings,
        }
    }

    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String> {
        let dataset = self
            .data_root
            .as_deref()
            .map(discover_swe_bench_pro)
            .ok_or_else(|| "swe-bench-pro data root is not configured".to_string())?;
        if dataset.data_state != DataState::Ready {
            return Err(format!(
                "swe-bench-pro data is not runnable: data_state={}",
                dataset.data_state
            ));
        }
        let ids = instance_ids_for_split(split, &dataset)?;
        let snapshot_hash = swe_dataset_snapshot_hash(&dataset, &ids);
        let mut prepared = prepared_with_identity(
            self.descriptor(),
            split,
            dataset.dataset_dir.display().to_string(),
            ids.len(),
            Some(dataset.source_dir.display().to_string()),
            ids,
            Some(snapshot_hash),
        );
        prepared.warnings = dataset.warnings;
        Ok(prepared)
    }

    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String> {
        let dataset = swe_dataset_from_prepared(prepared)?;
        ensure_prepared_snapshot_matches(prepared, &dataset)?;
        prepared
            .selected_task_ids
            .iter()
            .map(|task_id| swe_bench_pro_task_descriptor(&prepared.split, task_id, &dataset))
            .collect()
    }

    fn create_task_plan(
        &self,
        prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String> {
        let dataset = swe_dataset_from_prepared(prepared)?;
        ensure_prepared_snapshot_matches(prepared, &dataset)?;
        if !prepared
            .selected_task_ids
            .iter()
            .any(|id| id == &task.task_id)
        {
            return Err(format!("unknown swe-bench-pro task {}", task.task_id));
        }
        Ok(swe_bench_pro_task(&task.task_id, &dataset))
    }

    fn run_config_overrides(&self, _prepared: &PreparedBenchmark) -> RunConfigOverrides {
        RunConfigOverrides {
            timeout_sec: Some(7200),
            network: Some(NetworkPolicy::Full),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SweBenchProDataset {
    task_count: usize,
    data_state: DataState,
    dataset_dir: PathBuf,
    source_dir: PathBuf,
    task_ids: Vec<String>,
    warnings: Vec<String>,
}

fn swe_dataset_from_prepared(prepared: &PreparedBenchmark) -> Result<SweBenchProDataset, String> {
    if prepared.selected_task_ids.is_empty() {
        return Err("swe-bench-pro prepared task ids are missing".to_string());
    }
    let source_dir = prepared
        .source_manifest_path
        .as_ref()
        .ok_or_else(|| "swe-bench-pro prepared source manifest path is missing".to_string())?;
    Ok(SweBenchProDataset {
        task_count: prepared.selected_task_ids.len(),
        data_state: prepared.data_state,
        dataset_dir: PathBuf::from(&prepared.cache_manifest_path),
        source_dir: PathBuf::from(source_dir),
        task_ids: prepared.selected_task_ids.clone(),
        warnings: prepared.warnings.clone(),
    })
}

fn ensure_prepared_snapshot_matches(
    prepared: &PreparedBenchmark,
    dataset: &SweBenchProDataset,
) -> Result<(), String> {
    let Some(expected) = prepared.data_snapshot_hash.as_ref() else {
        return Err("swe-bench-pro prepared data snapshot hash is missing".to_string());
    };
    let actual = swe_dataset_snapshot_hash(dataset, &prepared.selected_task_ids);
    if &actual != expected {
        return Err(format!(
            "swe-bench-pro prepared data drift detected: expected {expected}, actual {actual}"
        ));
    }
    Ok(())
}

fn discover_swe_bench_pro(root: &Path) -> SweBenchProDataset {
    let dataset_dir = root.join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
    let dataset_dir = std::fs::canonicalize(&dataset_dir).unwrap_or(dataset_dir);
    let source_dir = root.join("_src/SWE-bench_Pro-os");
    let source_dir = std::fs::canonicalize(&source_dir).unwrap_or(source_dir);
    if dataset_dir.join(".partial").is_file() {
        return SweBenchProDataset {
            task_count: 0,
            data_state: DataState::Partial,
            dataset_dir,
            source_dir,
            task_ids: Vec::new(),
            warnings: vec!["swe-bench-pro dataset is marked partial".to_string()],
        };
    }
    let data_dir = dataset_dir.join("data");
    let has_parquet = std::fs::read_dir(data_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .find(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "parquet")
                && entry.metadata().is_ok_and(|meta| meta.len() > 0)
        })
        .is_some();
    let readme = dataset_dir.join("README.md");
    let task_count = read_num_examples(&readme);
    let has_evaluator = source_dir.join("swe_bench_pro_eval.py").is_file()
        && source_dir.join("run_scripts").is_dir();
    let task_ids = collect_run_script_ids(&source_dir);
    let mut warnings = Vec::new();
    if has_parquet && task_count.is_some_and(|count| count > 0) {
        let task_count = task_count.unwrap();
        let task_id_count = task_ids.len();
        if has_evaluator && task_id_count != task_count {
            warnings.push(format!(
                "swe-bench-pro source/data task count mismatch: readme={task_count}, run_scripts={task_id_count}"
            ));
        }
        return SweBenchProDataset {
            task_count,
            data_state: if has_evaluator && task_id_count == task_count {
                DataState::Ready
            } else {
                DataState::Corrupted
            },
            dataset_dir,
            source_dir,
            task_ids,
            warnings,
        };
    }
    let data_state = if has_parquet || readme.is_file() {
        DataState::Corrupted
    } else {
        DataState::RequiresAuth
    };
    SweBenchProDataset {
        task_count: 0,
        data_state,
        dataset_dir,
        source_dir,
        task_ids,
        warnings,
    }
}

fn first_parquet(dataset_dir: &Path) -> Option<PathBuf> {
    let mut files = std::fs::read_dir(dataset_dir.join("data"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "parquet"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    files.into_iter().next()
}

fn collect_run_script_ids(source_dir: &Path) -> Vec<String> {
    let mut ids = std::fs::read_dir(source_dir.join("run_scripts"))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_dir()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn instance_ids_for_split(
    split: &str,
    dataset: &SweBenchProDataset,
) -> Result<Vec<String>, String> {
    match split {
        "smoke" => dataset
            .task_ids
            .first()
            .cloned()
            .map(|id| vec![id])
            .ok_or_else(|| "swe-bench-pro smoke requires at least one run script".to_string()),
        "full" => {
            if dataset.task_ids.is_empty() {
                return Err("swe-bench-pro full requires run script task ids".to_string());
            }
            Ok(dataset.task_ids.clone())
        }
        other => Err(format!("unknown swe-bench-pro split {other}")),
    }
}

fn swe_bench_pro_task_descriptor(
    split: &str,
    task_id: &str,
    dataset: &SweBenchProDataset,
) -> Result<TaskDescriptor, String> {
    if !dataset
        .source_dir
        .join("run_scripts")
        .join(task_id)
        .is_dir()
    {
        return Err(format!(
            "swe-bench-pro task {task_id} is missing run script"
        ));
    }
    Ok(TaskDescriptor {
        task_id: task_id.to_string(),
        split: split.to_string(),
        estimated_timeout_sec: 7200,
        resource_hint: ResourceHint {
            cpu_cores: 4,
            memory_mb: 8192,
        },
        source_ref: SourceRef {
            benchmark: "swe-bench-pro".to_string(),
            upstream_id: task_id.to_string(),
            checksum: swe_source_ref_checksum(task_id, dataset),
        },
    })
}

fn swe_bench_pro_task(task_id: &str, dataset: &SweBenchProDataset) -> TaskPlan {
    TaskPlan {
        task_id: task_id.to_string(),
        instruction: format!("Solve official SWE-bench Pro instance {task_id}."),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::GitRepo,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: SandboxSpec {
            image: "swe-bench-pro-official".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::Full,
            privileged: false,
            resource_limits: ResourceHint {
                cpu_cores: 4,
                memory_mb: 8192,
            },
        },
        verifier_spec: VerifierSpec {
            command: "swe_bench_pro_eval.py".to_string(),
            working_dir: "workspace".to_string(),
            timeout_sec: 7200,
            expected_exit_codes: vec![0],
            environment_mode: VerifierEnvironment::HostProcess,
            output_parser: "swe_bench_pro_eval_results_json".to_string(),
        },
        artifact_spec: ArtifactSpec {
            base_dir: "workspace".to_string(),
            globs: vec!["**/*".to_string()],
            required_paths: Vec::new(),
            max_size_bytes: 256 * 1024 * 1024,
        },
        patch_spec: Some(PatchSpec {
            diff_path: "patch.diff".to_string(),
            prediction_path: "prediction.jsonl".to_string(),
        }),
        external_runner: Some(ExternalRunnerSpec {
            dataset_path: dataset.dataset_dir.display().to_string(),
            source_path: Some(dataset.source_dir.display().to_string()),
            agent_timeout_sec: None,
        }),
        runtime_binding: Some(swe_bench_pro_runtime_binding(dataset)),
    }
}

fn swe_bench_pro_runtime_binding(dataset: &SweBenchProDataset) -> TaskRuntimeBinding {
    TaskRuntimeBinding {
        authority: built_in_protocol_registry()
            .binding_for_adapter_id(
                &AdapterId::new("harnesslab.swe-bench-pro.runtime")
                    .expect("swe-bench-pro adapter id is valid"),
            )
            .expect("swe-bench-pro protocol binding is registered")
            .authority(),
        dataset_ref: dataset.dataset_dir.display().to_string(),
        task_ref: dataset.source_dir.display().to_string(),
        artifact_contract_id: "artifact.basic.v1".to_string(),
        readiness_contract_id: "readiness.basic.v1".to_string(),
    }
}

fn swe_source_ref_checksum(task_id: &str, dataset: &SweBenchProDataset) -> String {
    let parquet = first_parquet(&dataset.dataset_dir)
        .map(|path| stable_file_checksum(&path))
        .unwrap_or_else(|| "missing-parquet".to_string());
    let evaluator = stable_file_checksum(&dataset.source_dir.join("swe_bench_pro_eval.py"));
    stable_checksum(&format!(
        "swe-bench-pro:{task_id}:parquet={parquet}:evaluator={evaluator}"
    ))
}

fn swe_dataset_snapshot_hash(dataset: &SweBenchProDataset, task_ids: &[String]) -> String {
    let parquet = first_parquet(&dataset.dataset_dir)
        .map(|path| stable_file_checksum(&path))
        .unwrap_or_else(|| "missing-parquet".to_string());
    let evaluator = stable_file_checksum(&dataset.source_dir.join("swe_bench_pro_eval.py"));
    let run_scripts = task_ids
        .iter()
        .map(|task_id| format!("{task_id}:{}", run_script_identity(dataset, task_id)))
        .collect::<Vec<_>>()
        .join("|");
    stable_checksum(&format!(
        "dataset={}:source={}:parquet={parquet}:evaluator={evaluator}:tasks={run_scripts}",
        dataset.dataset_dir.display(),
        dataset.source_dir.display()
    ))
}

fn run_script_identity(dataset: &SweBenchProDataset, task_id: &str) -> String {
    let task_dir = dataset.source_dir.join("run_scripts").join(task_id);
    if !task_dir.is_dir() {
        return "missing".to_string();
    }
    let mut entries = std::fs::read_dir(task_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|file_type| file_type.is_file()))
        .map(|entry| {
            format!(
                "{}:{}",
                entry.file_name().to_string_lossy(),
                stable_file_checksum(&entry.path())
            )
        })
        .collect::<Vec<_>>();
    entries.sort();
    if entries.is_empty() {
        "dir:empty".to_string()
    } else {
        entries.join(",")
    }
}

fn read_num_examples(readme: &Path) -> Option<usize> {
    let content = std::fs::read_to_string(readme).ok()?;
    read_num_examples_from_str(&content)
}

fn read_num_examples_from_str(content: &str) -> Option<usize> {
    content
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("num_examples:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
}

#[cfg(test)]
#[path = "swe_bench_pro_tests.rs"]
mod tests;
