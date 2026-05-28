use crate::{BenchmarkAdapter, plan_from_tasks};
use harnesslab_core::{
    ArtifactSpec, BenchmarkDescriptor, BenchmarkPlan, BenchmarkSplit, BenchmarkStyle, DataState,
    NetworkPolicy, PatchSpec, ResourceHint, SandboxSpec, TaskPlan, VerifierEnvironment,
    VerifierSpec, WorkspaceSpec, WorkspaceType,
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
        BenchmarkDescriptor {
            name: "swe-bench-pro".to_string(),
            style: BenchmarkStyle::Patch,
            version: "2026".to_string(),
            homepage: "https://www.swebench.com/".to_string(),
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
                        .map_or(DataState::RequiresAuth, |dataset| dataset.data_state),
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
                    task_id: "swe-bench-pro-smoke".to_string(),
                    instruction: "Modify app.txt from old to swe-bench-pro-smoke.".to_string(),
                    workspace_spec: WorkspaceSpec {
                        workspace_type: WorkspaceType::GitRepo,
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
                        command: "grep -q swe-bench-pro-smoke app.txt".to_string(),
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
                    patch_spec: Some(PatchSpec {
                        diff_path: "patch.diff".to_string(),
                        prediction_path: "prediction.jsonl".to_string(),
                    }),
                }],
            )),
            "full" => {
                let state = self
                    .data_root
                    .as_deref()
                    .map(discover_swe_bench_pro)
                    .map_or(DataState::RequiresAuth, |dataset| dataset.data_state);
                if state == DataState::Unsupported {
                    Err(
                        "swe-bench-pro full data is present, but evaluator execution is not implemented yet"
                            .to_string(),
                    )
                } else if state == DataState::Corrupted {
                    Err("swe-bench-pro full data is present but incomplete or invalid".to_string())
                } else {
                    Err(
                        "swe-bench-pro full data is not available locally; download it first (requires HuggingFace auth)"
                            .to_string(),
                    )
                }
            }
            other => Err(format!("unknown swe-bench-pro split {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SweBenchProDataset {
    task_count: usize,
    data_state: DataState,
}

fn discover_swe_bench_pro(root: &Path) -> SweBenchProDataset {
    let dataset_dir = root.join("swe-bench-pro/ScaleAI__SWE-bench_Pro");
    let data_dir = dataset_dir.join("data");
    let has_parquet = std::fs::read_dir(data_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "parquet"))
        .is_some();
    let readme = dataset_dir.join("README.md");
    let task_count = read_num_examples(&readme);
    if has_parquet && task_count.is_some_and(|count| count > 0) {
        return SweBenchProDataset {
            task_count: task_count.unwrap(),
            data_state: DataState::Unsupported,
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
    fn c_bench_006_swe_bench_pro_full_reports_local_data_as_unsupported() {
        let root = tempfile::tempdir().unwrap();
        let data_dir = root
            .path()
            .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/data");
        std::fs::create_dir_all(&data_dir).unwrap();
        std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();
        std::fs::write(
            root.path()
                .join("swe-bench-pro/ScaleAI__SWE-bench_Pro/README.md"),
            "splits:\n- name: test\n  num_examples: 731\n",
        )
        .unwrap();

        let descriptor = SweBenchProAdapter::with_data_root(Some(root.path())).descriptor();
        let full = descriptor
            .splits
            .iter()
            .find(|split| split.name == "full")
            .unwrap();

        assert_eq!(full.task_count, 731);
        assert_eq!(full.data_state, DataState::Unsupported);
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
        std::fs::write(data_dir.join("test-00000-of-00001.parquet"), "").unwrap();

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
        assert!(missing_error.contains("not available locally"));

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
        assert!(present_error.contains("data is present"));
    }
}
