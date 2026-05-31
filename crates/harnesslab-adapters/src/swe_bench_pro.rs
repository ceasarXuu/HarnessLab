use crate::BenchmarkAdapter;
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkIdentity, BenchmarkPlan, BenchmarkSplit,
    BenchmarkStyle, DataState, ExternalRunnerKind, ExternalRunnerSpec, NetworkPolicy, PatchSpec,
    ResourceHint, RunConfigOverrides, SandboxSpec, TaskPlan, VerifierEnvironment, VerifierSpec,
    WorkspaceSpec, WorkspaceType,
};
use std::path::{Path, PathBuf};
use std::process::Command;

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

    fn plan(&self, split: &str) -> Result<BenchmarkPlan, String> {
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
        match split {
            "smoke" => {
                let ids = extract_instance_ids(&dataset.dataset_dir, Some(1))?;
                Ok(plan_from_dataset(self.descriptor(), split, &dataset, ids))
            }
            "full" => {
                let ids = extract_instance_ids(&dataset.dataset_dir, None)?;
                Ok(plan_from_dataset(self.descriptor(), split, &dataset, ids))
            }
            other => Err(format!("unknown swe-bench-pro split {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SweBenchProDataset {
    task_count: usize,
    data_state: DataState,
    dataset_dir: PathBuf,
    source_dir: PathBuf,
}

fn discover_swe_bench_pro(root: &Path) -> SweBenchProDataset {
    let dataset_dir = root.join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
    let dataset_dir = std::fs::canonicalize(&dataset_dir).unwrap_or(dataset_dir);
    let source_dir = root.join("_src/SWE-bench_Pro-os");
    let source_dir = std::fs::canonicalize(&source_dir).unwrap_or(source_dir);
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
    if has_parquet && task_count.is_some_and(|count| count > 0) {
        return SweBenchProDataset {
            task_count: task_count.unwrap(),
            data_state: if has_evaluator {
                DataState::Ready
            } else {
                DataState::Corrupted
            },
            dataset_dir,
            source_dir,
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
    }
}

fn extract_instance_ids(dataset_dir: &Path, limit: Option<usize>) -> Result<Vec<String>, String> {
    let parquet = first_parquet(dataset_dir)
        .ok_or_else(|| "swe-bench-pro parquet data is missing".to_string())?;
    let limit_arg = limit.unwrap_or(0).to_string();
    let script = r#"
import pandas as pd
import sys
path, limit = sys.argv[1], int(sys.argv[2])
df = pd.read_parquet(path, columns=["instance_id"])
ids = [str(v) for v in df["instance_id"].dropna().tolist()]
for value in (ids[:limit] if limit else ids):
    print(value)
"#;
    let output = Command::new("uv")
        .args([
            "run", "--with", "pandas", "--with", "pyarrow", "python", "-c",
        ])
        .arg(script)
        .arg(parquet)
        .arg(limit_arg)
        .output()
        .map_err(|error| format!("failed to execute uv for swe-bench-pro metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to read swe-bench-pro parquet metadata: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let ids = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Err("swe-bench-pro parquet contains no instance_id rows".to_string());
    }
    Ok(ids)
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

fn plan_from_dataset(
    descriptor: BenchmarkDescriptor,
    split: &str,
    dataset: &SweBenchProDataset,
    task_ids: Vec<String>,
) -> BenchmarkPlan {
    let tasks = task_ids
        .into_iter()
        .map(|task_id| swe_bench_pro_task(&task_id, dataset))
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
            timeout_sec: Some(7200),
            network: Some(NetworkPolicy::Full),
        },
        warnings: Vec::new(),
    }
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
            kind: ExternalRunnerKind::SweBenchPro,
            dataset_path: dataset.dataset_dir.display().to_string(),
            source_path: Some(dataset.source_dir.display().to_string()),
        }),
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
mod tests {
    use super::*;

    #[test]
    fn c_bench_006_swe_bench_pro_full_reports_local_data_as_ready() {
        let root = tempfile::tempdir().unwrap();
        let data_dir = root
            .path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();
        std::fs::write(
            root.path()
                .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
            "splits:\n- name: test\n  num_examples: 731\n",
        )
        .unwrap();
        create_source(root.path());

        let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
        let full = descriptor
            .splits
            .iter()
            .find(|split| split.name == "full")
            .unwrap();

        assert_eq!(full.task_count, 731);
        assert_eq!(full.data_state, DataState::Ready);
    }

    #[test]
    fn c_bench_006_swe_bench_pro_full_reports_missing_data_as_requires_auth() {
        let root = tempfile::tempdir().unwrap();

        let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
        let full = descriptor
            .splits
            .iter()
            .find(|split| split.name == "full")
            .unwrap();

        assert_eq!(full.task_count, 0);
        assert_eq!(full.data_state, DataState::RequiresAuth);
    }

    #[test]
    fn c_bench_006_swe_bench_pro_num_examples_parser_handles_comments() {
        assert_eq!(
            read_num_examples_from_str("splits:\n- name: test\n  num_examples: 731 # comment\n"),
            Some(731)
        );
        assert_eq!(
            read_num_examples_from_str("  num_examples:   731  \r\n"),
            Some(731)
        );
        assert_eq!(read_num_examples_from_str("num_examples: many\n"), None);
        assert_eq!(read_num_examples_from_str("splits: []\n"), None);
    }

    #[test]
    fn c_bench_006_swe_bench_pro_parquet_without_readme_is_corrupted() {
        let root = tempfile::tempdir().unwrap();
        let data_dir = root
            .path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "parquet").unwrap();

        let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
        let full = descriptor
            .splits
            .iter()
            .find(|split| split.name == "full")
            .unwrap();

        assert_eq!(full.task_count, 0);
        assert_eq!(full.data_state, DataState::Corrupted);
    }

    #[test]
    fn c_bench_006_swe_bench_pro_readme_without_parquet_is_corrupted() {
        let root = tempfile::tempdir().unwrap();
        let dataset_dir = root.path().join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
        std::fs::create_dir_all(&dataset_dir).unwrap();
        std::fs::write(dataset_dir.join("README.md"), "num_examples: 731\n").unwrap();

        let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
        let full = descriptor
            .splits
            .iter()
            .find(|split| split.name == "full")
            .unwrap();

        assert_eq!(full.task_count, 0);
        assert_eq!(full.data_state, DataState::Corrupted);
    }

    #[test]
    fn c_bench_006_swe_bench_pro_full_plan_errors_match_data_state() {
        let missing = tempfile::tempdir().unwrap();
        let missing_error = SweBenchProAdapter::with_data_root(Some(missing.path()))
            .plan("full")
            .unwrap_err();
        assert!(missing_error.contains("data_state=requires_auth"));

        let present = tempfile::tempdir().unwrap();
        let data_dir = present
            .path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();
        std::fs::write(
            present
                .path()
                .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
            "splits:\n- name: test\n  num_examples: 731\n",
        )
        .unwrap();
        let present_error = SweBenchProAdapter::with_data_root(Some(present.path()))
            .plan("full")
            .unwrap_err();
        assert!(present_error.contains("data_state=corrupted"));
    }

    #[test]
    fn c_bench_006_swe_bench_pro_task_uses_external_runner() {
        let root = tempfile::tempdir().unwrap();
        let dataset = SweBenchProDataset {
            task_count: 1,
            data_state: DataState::Ready,
            dataset_dir: root.path().join("dataset"),
            source_dir: root.path().join("source"),
        };

        let task = swe_bench_pro_task("instance_demo", &dataset);

        assert_eq!(
            task.external_runner.as_ref().unwrap().kind,
            ExternalRunnerKind::SweBenchPro
        );
        assert!(task.patch_spec.is_some());
    }

    fn create_source(root: &Path) {
        let source = root.join("_src/SWE-bench_Pro-os");
        std::fs::create_dir_all(source.join("run_scripts")).unwrap();
        std::fs::write(source.join("swe_bench_pro_eval.py"), "").unwrap();
    }
}
