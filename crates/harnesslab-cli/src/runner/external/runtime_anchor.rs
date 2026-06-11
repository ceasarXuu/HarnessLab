use anyhow::Result;
use harnesslab_core::{
    BenchmarkPlan, ExternalRuntimeAttemptSnapshot, RuntimeTaskSnapshot, task_dir_name,
};
use harnesslab_infra::{
    append_event, atomic_write_json, event, read_json, with_exclusive_file_lock,
};
use std::path::{Path, PathBuf};

pub(super) struct AnchorProjection {
    pub(super) run_id: String,
    pub(super) task_id: String,
    pub(super) attempt: u32,
    pub(super) attempt_dir: PathBuf,
    pub(super) private_path: PathBuf,
    pub(super) public_path: PathBuf,
    pub(super) private_checksum: String,
    pub(super) public_checksum: String,
    pub(super) runtime_fingerprint: String,
    pub(super) public_fingerprint: String,
}

pub(super) fn anchor_attempt_snapshot(projection: AnchorProjection) -> Result<()> {
    let paths = AnchorPaths::from_attempt_dir(&projection.attempt_dir, &projection.task_id)?;
    let lock_path = paths
        .run_dir
        .join(".harnesslab-locks/external-runtime-anchor.lock");
    with_exclusive_file_lock(&lock_path, || anchor_locked(&projection, &paths))
}

struct AnchorPaths {
    task_dir: PathBuf,
    run_dir: PathBuf,
    task_runtime_path: PathBuf,
    benchmark_path: PathBuf,
}

impl AnchorPaths {
    fn from_attempt_dir(attempt_dir: &Path, task_id: &str) -> Result<Self> {
        let Some(task_dir) = attempt_dir.parent().and_then(Path::parent) else {
            anyhow::bail!("external runtime snapshot anchor path missing for {task_id}");
        };
        let Some(run_dir) = task_dir.parent().and_then(Path::parent) else {
            anyhow::bail!("external runtime benchmark anchor path missing for {task_id}");
        };
        let task_runtime_path = task_dir.join("task-runtime.snapshot.json");
        if !task_runtime_path.exists() {
            anyhow::bail!(
                "external runtime snapshot anchor missing task runtime snapshot for {task_id}"
            );
        }
        let actual_task_dir_name = task_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if actual_task_dir_name != task_dir_name(task_id)?.as_str() {
            anyhow::bail!("external runtime snapshot anchor path mismatch for {task_id}");
        }
        Ok(Self {
            task_dir: task_dir.to_path_buf(),
            run_dir: run_dir.to_path_buf(),
            task_runtime_path,
            benchmark_path: run_dir.join("benchmark.snapshot.json"),
        })
    }
}

fn anchor_locked(projection: &AnchorProjection, paths: &AnchorPaths) -> Result<()> {
    let mut snapshot: RuntimeTaskSnapshot = read_json(&paths.task_runtime_path)?;
    if snapshot.task_id != projection.task_id {
        anyhow::bail!(
            "external runtime snapshot anchor task mismatch: expected {} got {}",
            projection.task_id,
            snapshot.task_id
        );
    }
    let anchor = ExternalRuntimeAttemptSnapshot {
        attempt: projection.attempt,
        private_path: attempt_relative_path(&projection.private_path, &paths.task_dir),
        public_path: attempt_relative_path(&projection.public_path, &paths.task_dir),
        private_checksum: projection.private_checksum.clone(),
        public_checksum: projection.public_checksum.clone(),
        runtime_fingerprint: projection.runtime_fingerprint.clone(),
        public_fingerprint: projection.public_fingerprint.clone(),
    };
    snapshot
        .external_runtime_attempts
        .retain(|entry| entry.attempt != anchor.attempt);
    snapshot.external_runtime_attempts.push(anchor);
    snapshot
        .external_runtime_attempts
        .sort_by_key(|entry| entry.attempt);
    atomic_write_json(&paths.task_runtime_path, &snapshot)?;
    anchor_benchmark_snapshot(&paths.benchmark_path, &projection.task_id, &snapshot)?;
    append_event(
        &paths.run_dir.join("events.jsonl"),
        &event(
            &projection.run_id,
            Some(&projection.task_id),
            "external_runtime_anchor_projected",
            &format!(
                "attempt={} private={} public={}",
                projection.attempt,
                attempt_relative_path(&projection.private_path, &paths.task_dir),
                attempt_relative_path(&projection.public_path, &paths.task_dir)
            ),
        ),
        &[],
    )?;
    Ok(())
}

fn anchor_benchmark_snapshot(
    benchmark_path: &Path,
    task_id: &str,
    snapshot: &RuntimeTaskSnapshot,
) -> Result<()> {
    let mut plan: BenchmarkPlan = read_json(benchmark_path)?;
    let Some(entry) = plan
        .task_runtime_snapshots
        .iter_mut()
        .find(|entry| entry.task_id == task_id)
    else {
        anyhow::bail!("external runtime benchmark anchor missing task {task_id}");
    };
    *entry = snapshot.clone();
    atomic_write_json(benchmark_path, &plan)?;
    Ok(())
}

fn attempt_relative_path(path: &Path, task_dir: &Path) -> String {
    path.strip_prefix(task_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{
        ArtifactSpec, BenchmarkIdentity, ExternalRunnerKind, ExternalRunnerSpec, NetworkPolicy,
        ResourceHint, RunConfigOverrides, SandboxSpec, SourceRef, TaskPlan, VerifierEnvironment,
        VerifierSpec, WorkspaceSpec, WorkspaceType,
    };
    use harnesslab_infra::{Event, read_json, stable_file_checksum};
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn anchor_001_same_task_concurrent_attempts_keep_all_anchors() {
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("run");
        initialize_run(&run_dir, &["task-a"]);
        let barrier = Arc::new(Barrier::new(2));

        let mut workers = Vec::new();
        for attempt in [1, 2] {
            let projection = projection(&run_dir, "task-a", attempt);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                anchor_attempt_snapshot(projection).unwrap();
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }

        assert_attempts(
            &run_dir.join("tasks/task-a/task-runtime.snapshot.json"),
            &[1, 2],
        );
        assert_benchmark_attempts(&run_dir, "task-a", &[1, 2]);
        assert_projection_events(&run_dir, &[1, 2]);
    }

    #[test]
    fn anchor_002_parallel_task_completion_preserves_other_task_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let run_dir = tmp.path().join("run");
        initialize_run(&run_dir, &["task-a", "task-b"]);
        let barrier = Arc::new(Barrier::new(2));

        let mut workers = Vec::new();
        for task_id in ["task-a", "task-b"] {
            let projection = projection(&run_dir, task_id, 1);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                anchor_attempt_snapshot(projection).unwrap();
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }

        assert_benchmark_attempts(&run_dir, "task-a", &[1]);
        assert_benchmark_attempts(&run_dir, "task-b", &[1]);
    }

    fn initialize_run(run_dir: &Path, task_ids: &[&str]) {
        let snapshots = task_ids
            .iter()
            .map(|task_id| {
                let snapshot = runtime_snapshot(task_id);
                let task_dir = run_dir.join("tasks").join(task_dir_name(task_id).unwrap());
                fs::create_dir_all(&task_dir).unwrap();
                atomic_write_json(&task_dir.join("task-runtime.snapshot.json"), &snapshot).unwrap();
                snapshot
            })
            .collect::<Vec<_>>();
        let tasks = task_ids
            .iter()
            .map(|task_id| task_plan(task_id))
            .collect::<Vec<_>>();
        let plan = BenchmarkPlan {
            benchmark: benchmark_identity(),
            split: "dev".to_string(),
            prepared_benchmark_ref: "prepared".to_string(),
            tasks,
            task_runtime_snapshots: snapshots,
            run_config_overrides: RunConfigOverrides {
                timeout_sec: None,
                network: None,
            },
            warnings: Vec::new(),
        };
        atomic_write_json(&run_dir.join("benchmark.snapshot.json"), &plan).unwrap();
    }

    fn projection(run_dir: &Path, task_id: &str, attempt: u32) -> AnchorProjection {
        let task_dir = run_dir.join("tasks").join(task_dir_name(task_id).unwrap());
        let attempt_dir = task_dir.join("attempts").join(attempt.to_string());
        fs::create_dir_all(&attempt_dir).unwrap();
        let private_path = attempt_dir.join("external-runtime.private.json");
        let public_path = attempt_dir.join("external-runtime.public.json");
        fs::write(&private_path, format!("private {task_id} {attempt}")).unwrap();
        fs::write(&public_path, format!("public {task_id} {attempt}")).unwrap();
        AnchorProjection {
            run_id: "run".to_string(),
            task_id: task_id.to_string(),
            attempt,
            attempt_dir,
            private_checksum: stable_file_checksum(&private_path),
            public_checksum: stable_file_checksum(&public_path),
            private_path,
            public_path,
            runtime_fingerprint: format!("runtime-{task_id}-{attempt}"),
            public_fingerprint: format!("public-{task_id}-{attempt}"),
        }
    }

    fn assert_attempts(path: &Path, expected: &[u32]) {
        let snapshot: RuntimeTaskSnapshot = read_json(path).unwrap();
        let attempts = snapshot
            .external_runtime_attempts
            .iter()
            .map(|entry| entry.attempt)
            .collect::<Vec<_>>();
        assert_eq!(attempts, expected);
    }

    fn assert_benchmark_attempts(run_dir: &Path, task_id: &str, expected: &[u32]) {
        let plan: BenchmarkPlan = read_json(&run_dir.join("benchmark.snapshot.json")).unwrap();
        let snapshot = plan
            .task_runtime_snapshots
            .iter()
            .find(|snapshot| snapshot.task_id == task_id)
            .unwrap();
        let attempts = snapshot
            .external_runtime_attempts
            .iter()
            .map(|entry| entry.attempt)
            .collect::<Vec<_>>();
        assert_eq!(attempts, expected);
    }

    fn assert_projection_events(run_dir: &Path, expected_attempts: &[u32]) {
        let content = fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
        let events = content
            .lines()
            .map(|line| serde_json::from_str::<Event>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), expected_attempts.len());
        for attempt in expected_attempts {
            let expected_private =
                format!("private=attempts/{attempt}/external-runtime.private.json");
            let expected_public = format!("public=attempts/{attempt}/external-runtime.public.json");
            let event = events
                .iter()
                .find(|event| {
                    event.event == "external_runtime_anchor_projected"
                        && event.task_id.as_deref() == Some("task-a")
                        && event.message.contains(&format!("attempt={attempt}"))
                })
                .unwrap_or_else(|| panic!("missing projection event for attempt {attempt}"));
            assert_eq!(event.run_id, "[PRIVATE_RUN_ID]");
            assert!(event.message.contains(&expected_private));
            assert!(event.message.contains(&expected_public));
        }
    }

    fn runtime_snapshot(task_id: &str) -> RuntimeTaskSnapshot {
        RuntimeTaskSnapshot {
            benchmark: benchmark_identity(),
            split: "dev".to_string(),
            task_id: task_id.to_string(),
            source_ref: SourceRef {
                benchmark: "swe-bench-pro".to_string(),
                upstream_id: task_id.to_string(),
                checksum: "source".to_string(),
            },
            upstream_metadata_hash: "metadata".to_string(),
            instruction_hash: "instruction".to_string(),
            task_plan_hash: "task-plan".to_string(),
            external_runner: Some(external_runner()),
            runtime_binding: None,
            external_runtime_attempts: Vec::new(),
        }
    }

    fn task_plan(task_id: &str) -> TaskPlan {
        TaskPlan {
            task_id: task_id.to_string(),
            instruction: "fix".to_string(),
            workspace_spec: WorkspaceSpec {
                workspace_type: WorkspaceType::GitRepo,
                target_path: "workspace".to_string(),
                clean: true,
            },
            sandbox_spec: SandboxSpec {
                image: "ubuntu:latest".to_string(),
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
                command: "true".to_string(),
                working_dir: ".".to_string(),
                timeout_sec: 1,
                expected_exit_codes: vec![0],
                environment_mode: VerifierEnvironment::HostProcess,
                output_parser: "exit_code".to_string(),
            },
            artifact_spec: ArtifactSpec {
                base_dir: ".".to_string(),
                globs: Vec::new(),
                required_paths: Vec::new(),
                max_size_bytes: 1024,
            },
            patch_spec: None,
            external_runner: Some(external_runner()),
            runtime_binding: None,
        }
    }

    fn external_runner() -> ExternalRunnerSpec {
        ExternalRunnerSpec {
            kind: ExternalRunnerKind::SweBenchPro,
            dataset_path: "dataset.parquet".to_string(),
            source_path: Some("source".to_string()),
            agent_timeout_sec: None,
        }
    }

    fn benchmark_identity() -> BenchmarkIdentity {
        BenchmarkIdentity {
            name: "swe-bench-pro".to_string(),
            version: "fixture".to_string(),
        }
    }
}
