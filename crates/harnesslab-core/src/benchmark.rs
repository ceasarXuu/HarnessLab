use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkDescriptor {
    pub name: String,
    pub style: BenchmarkStyle,
    pub version: String,
    pub homepage: String,
    pub splits: Vec<BenchmarkSplit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStyle {
    Terminal,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkSplit {
    pub name: String,
    pub task_count: usize,
    pub data_state: DataState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataState {
    NotDownloaded,
    Downloading,
    Partial,
    Ready,
    Corrupted,
    RequiresAuth,
    AuthFailed,
    Unsupported,
}

impl std::fmt::Display for DataState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            DataState::NotDownloaded => "not_downloaded",
            DataState::Downloading => "downloading",
            DataState::Partial => "partial",
            DataState::Ready => "ready",
            DataState::Corrupted => "corrupted",
            DataState::RequiresAuth => "requires_auth",
            DataState::AuthFailed => "auth_failed",
            DataState::Unsupported => "unsupported",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreparedBenchmark {
    pub descriptor: BenchmarkDescriptor,
    pub split: String,
    pub data_state: DataState,
    pub prepared_at: String,
    pub task_count: usize,
    pub cache_manifest_path: String,
    #[serde(default)]
    pub source_manifest_path: Option<String>,
    #[serde(default)]
    pub selected_task_ids: Vec<String>,
    #[serde(default)]
    pub data_snapshot_hash: Option<String>,
    pub size_bytes: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkDataState {
    pub descriptor: BenchmarkDescriptor,
    pub cache_manifest_path: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub task_id: String,
    pub split: String,
    pub estimated_timeout_sec: u64,
    pub resource_hint: ResourceHint,
    pub source_ref: SourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceHint {
    pub cpu_cores: usize,
    pub memory_mb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub benchmark: String,
    pub upstream_id: String,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeTaskSnapshot {
    pub benchmark: BenchmarkIdentity,
    pub split: String,
    pub task_id: String,
    pub source_ref: SourceRef,
    pub upstream_metadata_hash: String,
    pub instruction_hash: String,
    pub task_plan_hash: String,
    #[serde(default)]
    pub external_runner: Option<ExternalRunnerSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkPlan {
    pub benchmark: BenchmarkIdentity,
    pub split: String,
    pub prepared_benchmark_ref: String,
    pub tasks: Vec<TaskPlan>,
    pub run_config_overrides: RunConfigOverrides,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunConfigOverrides {
    pub timeout_sec: Option<u64>,
    pub network: Option<crate::NetworkPolicy>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub instruction: String,
    pub workspace_spec: WorkspaceSpec,
    pub sandbox_spec: SandboxSpec,
    pub verifier_spec: VerifierSpec,
    pub artifact_spec: ArtifactSpec,
    pub patch_spec: Option<PatchSpec>,
    #[serde(default)]
    pub external_runner: Option<ExternalRunnerSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalRunnerSpec {
    pub kind: ExternalRunnerKind,
    pub dataset_path: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default)]
    pub agent_timeout_sec: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalRunnerKind {
    TerminalBench,
    SweBenchPro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceSpec {
    pub workspace_type: WorkspaceType,
    pub target_path: String,
    pub clean: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceType {
    Empty,
    GitRepo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxSpec {
    pub image: String,
    pub mounts: Vec<String>,
    pub env_vars: Vec<String>,
    pub network: crate::NetworkPolicy,
    pub privileged: bool,
    pub resource_limits: ResourceHint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierSpec {
    pub command: String,
    pub working_dir: String,
    pub timeout_sec: u64,
    pub expected_exit_codes: Vec<i32>,
    pub environment_mode: VerifierEnvironment,
    pub output_parser: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierEnvironment {
    SameSandbox,
    SeparateSandbox,
    HostProcess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSpec {
    pub base_dir: String,
    pub globs: Vec<String>,
    pub required_paths: Vec<String>,
    pub max_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchSpec {
    pub diff_path: String,
    pub prediction_path: String,
}

pub fn data_state_blocks_run(state: DataState) -> bool {
    !matches!(state, DataState::Ready | DataState::Partial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_bench_001_descriptor_is_serializable() {
        let descriptor = BenchmarkDescriptor {
            name: "fake-terminal".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "fixture".to_string(),
            homepage: "local".to_string(),
            splits: vec![BenchmarkSplit {
                name: "success".to_string(),
                task_count: 1,
                data_state: DataState::Ready,
            }],
        };

        let json = serde_json::to_string(&descriptor).unwrap();

        assert!(json.contains("fake-terminal"));
    }

    #[test]
    fn c_bench_001_data_state_blocking_policy_is_stable() {
        let runnable = [DataState::Ready, DataState::Partial];
        let blocked = [
            DataState::NotDownloaded,
            DataState::Downloading,
            DataState::Corrupted,
            DataState::RequiresAuth,
            DataState::AuthFailed,
            DataState::Unsupported,
        ];

        for state in runnable {
            assert!(!data_state_blocks_run(state), "{state} should be runnable");
        }
        for state in blocked {
            assert!(data_state_blocks_run(state), "{state} should block run");
        }
    }

    #[test]
    fn c_bench_001_data_state_display_matches_json_names() {
        let cases = [
            (DataState::NotDownloaded, "not_downloaded"),
            (DataState::Downloading, "downloading"),
            (DataState::Partial, "partial"),
            (DataState::Ready, "ready"),
            (DataState::Corrupted, "corrupted"),
            (DataState::RequiresAuth, "requires_auth"),
            (DataState::AuthFailed, "auth_failed"),
            (DataState::Unsupported, "unsupported"),
        ];

        for (state, expected) in cases {
            assert_eq!(state.to_string(), expected);
        }
    }
}
