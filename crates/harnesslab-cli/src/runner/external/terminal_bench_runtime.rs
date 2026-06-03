use super::ExternalTaskExecution;
use anyhow::{Result, bail};
use harnesslab_infra::{append_event, event};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_DOCKER_PLATFORM: &str = "linux/amd64";
const NATIVE_ARM64_PLATFORM: &str = "linux/arm64";

#[derive(Clone, Copy)]
pub(super) enum QemuCompatMode {
    Amd64MakeJ1,
    NativeArm64CrossX86,
}

impl QemuCompatMode {
    fn label(self) -> &'static str {
        match self {
            Self::Amd64MakeJ1 => "amd64_qemu_make_j1",
            Self::NativeArm64CrossX86 => "native_arm64_cross_x86",
        }
    }
}

pub(super) fn terminal_bench_docker_platform(
    task_id: &str,
    override_platform: Option<&str>,
) -> String {
    terminal_bench_docker_platform_for_host(task_id, override_platform, std::env::consts::ARCH)
}

pub(super) fn terminal_bench_docker_platform_for_host(
    task_id: &str,
    override_platform: Option<&str>,
    host_arch: &str,
) -> String {
    override_platform
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_docker_platform(task_id, host_arch))
        .to_string()
}

pub(super) fn terminal_bench_no_output_activity_patterns() -> Vec<String> {
    [
        "docker compose",
        "docker-compose",
        "docker build",
        "docker buildx",
        "docker-buildx",
        "docker pull",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(super) fn append_runner_config_event(
    ctx: &ExternalTaskExecution<'_>,
    process_timeout_sec: u64,
    no_output_timeout_sec: Option<u64>,
    docker_platform: &str,
) -> Result<()> {
    let no_output_timeout = no_output_timeout_sec
        .map(|timeout| timeout.to_string())
        .unwrap_or_else(|| "disabled".to_string());
    let activity_grace = no_output_timeout.clone();
    let activity_patterns = terminal_bench_no_output_activity_patterns().join(",");
    let progress_paths = "official/terminal-bench/<run-id>/run.log";
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "external_runner_configured",
            &format!(
                "terminal-bench process_timeout_sec={process_timeout_sec} no_output_timeout_sec={no_output_timeout} activity_grace_sec={activity_grace} docker_platform={docker_platform} progress_paths={progress_paths} activity_patterns={activity_patterns}"
            ),
        ),
        &[],
    )
}

pub(super) fn terminal_bench_runtime_dataset(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    docker_platform: &str,
) -> Result<PathBuf> {
    let Some(mode) = qemu_compat_mode(&ctx.task.task_id, docker_platform) else {
        return Ok(dataset_path.to_path_buf());
    };
    let target_root = ctx.attempt_dir.join("runtime/terminal-bench-dataset");
    let runtime_dataset =
        prepare_qemu_task_dataset(dataset_path, &ctx.task.task_id, &target_root, mode)?;
    let runtime_dataset = fs::canonicalize(runtime_dataset)?;
    append_event(
        &ctx.run_dir.join("events.jsonl"),
        &event(
            &ctx.spec.run_id,
            Some(&ctx.task.task_id),
            "terminal_bench_dataset_prepared",
            &format!(
                "compatibility={} source_dataset={} runtime_dataset={}",
                mode.label(),
                dataset_path.display(),
                runtime_dataset.display()
            ),
        ),
        &[],
    )?;
    Ok(runtime_dataset)
}

pub(super) fn prepare_qemu_task_dataset(
    source_dataset_path: &Path,
    task_id: &str,
    target_dataset_path: &Path,
    mode: QemuCompatMode,
) -> Result<PathBuf> {
    let source_task = source_dataset_path.join(task_id);
    if !source_task.is_dir() {
        bail!(
            "terminal-bench task {task_id} is missing under {}",
            source_dataset_path.display()
        );
    }
    let target_task = target_dataset_path.join(task_id);
    copy_dir_recursive(&source_task, &target_task)?;
    patch_qemu_dockerfile(&target_task.join("Dockerfile"), mode)?;
    Ok(target_dataset_path.to_path_buf())
}

fn default_docker_platform(task_id: &str, host_arch: &str) -> &'static str {
    if is_x86_qemu_task(task_id) && host_arch == "aarch64" {
        NATIVE_ARM64_PLATFORM
    } else {
        DEFAULT_DOCKER_PLATFORM
    }
}

fn qemu_compat_mode(task_id: &str, docker_platform: &str) -> Option<QemuCompatMode> {
    if !is_x86_qemu_task(task_id) {
        return None;
    }
    match docker_platform {
        DEFAULT_DOCKER_PLATFORM => Some(QemuCompatMode::Amd64MakeJ1),
        NATIVE_ARM64_PLATFORM => Some(QemuCompatMode::NativeArm64CrossX86),
        _ => None,
    }
}

fn is_x86_qemu_task(task_id: &str) -> bool {
    matches!(task_id, "build-initramfs-qemu" | "build-tcc-qemu")
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn patch_qemu_dockerfile(path: &Path, mode: QemuCompatMode) -> Result<()> {
    let original = fs::read_to_string(path)?;
    let patched = match mode {
        QemuCompatMode::Amd64MakeJ1 => original.replace(
            "RUN cd linux-6.9 && make -j$(nproc)",
            "RUN cd linux-6.9 && make -j1",
        ),
        QemuCompatMode::NativeArm64CrossX86 => patch_native_arm64_cross_x86(&original),
    };
    if patched == original {
        bail!(
            "terminal-bench qemu Dockerfile has no kernel build command to patch: {}",
            path.display()
        );
    }
    fs::write(path, patched)?;
    Ok(())
}

fn patch_native_arm64_cross_x86(original: &str) -> String {
    original
        .replace(
            "RUN apt-get install -y build-essential libncurses-dev bison flex libssl-dev libelf-dev qemu-system bc cpio wget expect",
            "RUN apt-get install -y build-essential libncurses-dev bison flex libssl-dev libelf-dev qemu-system bc cpio wget expect gcc-x86-64-linux-gnu",
        )
        .replace(
            "RUN cd linux-6.9 && make defconfig",
            "RUN cd linux-6.9 && make ARCH=x86_64 CROSS_COMPILE=x86_64-linux-gnu- defconfig",
        )
        .replace(
            "RUN cd linux-6.9 && make olddefconfig",
            "RUN cd linux-6.9 && make ARCH=x86_64 CROSS_COMPILE=x86_64-linux-gnu- olddefconfig",
        )
        .replace(
            "RUN cd linux-6.9 && make -j$(nproc)",
            "RUN cd linux-6.9 && make ARCH=x86_64 CROSS_COMPILE=x86_64-linux-gnu- -j$(nproc)",
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{
        AgentKind, ArtifactSpec, AttemptProvenance, BenchmarkRef, ExecutionConfig,
        ExternalRunnerKind, ExternalRunnerSpec, FailureClass, FailureCode, HealthImpact,
        NetworkPolicy, Outcome, ResourceHint, RunPaths, RunSpec, SandboxSpec, TaskPlan, TaskState,
        VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType, default_agent_profile,
        task_dir_name,
    };
    use std::time::Instant;

    #[test]
    fn docker_platform_is_task_and_host_scoped() {
        assert_eq!(
            terminal_bench_docker_platform_for_host("hello-world", None, "aarch64"),
            "linux/amd64"
        );
        assert_eq!(
            terminal_bench_docker_platform_for_host("build-initramfs-qemu", None, "aarch64"),
            "linux/arm64"
        );
        assert_eq!(
            terminal_bench_docker_platform_for_host(
                "build-initramfs-qemu",
                Some("linux/amd64"),
                "aarch64"
            ),
            "linux/amd64"
        );
    }

    #[test]
    fn terminal_bench_runtime_prepares_qemu_dataset_without_mutating_source() {
        for task_id in ["build-initramfs-qemu", "build-tcc-qemu"] {
            let source = tempfile::tempdir().unwrap();
            let target = tempfile::tempdir().unwrap();
            let task_dir = write_qemu_task(source.path(), task_id, CANONICAL_QEMU_DOCKERFILE);

            let runtime_dataset = prepare_qemu_task_dataset(
                source.path(),
                task_id,
                target.path(),
                QemuCompatMode::NativeArm64CrossX86,
            )
            .unwrap();

            let patched =
                fs::read_to_string(runtime_dataset.join(task_id).join("Dockerfile")).unwrap();
            let original = fs::read_to_string(task_dir.join("Dockerfile")).unwrap();
            assert!(patched.contains("gcc-x86-64-linux-gnu"));
            assert!(
                patched.contains("make ARCH=x86_64 CROSS_COMPILE=x86_64-linux-gnu- -j$(nproc)")
            );
            assert!(original.contains("RUN cd linux-6.9 && make -j$(nproc)"));
            assert!(
                runtime_dataset
                    .join(task_id)
                    .join("tests/test_outputs.py")
                    .exists()
            );
        }
    }

    #[test]
    fn terminal_bench_runtime_prepares_forced_amd64_qemu_dataset() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let task_dir = write_qemu_task(source.path(), "build-tcc-qemu", CANONICAL_QEMU_DOCKERFILE);

        let runtime_dataset = prepare_qemu_task_dataset(
            source.path(),
            "build-tcc-qemu",
            target.path(),
            QemuCompatMode::Amd64MakeJ1,
        )
        .unwrap();

        let patched =
            fs::read_to_string(runtime_dataset.join("build-tcc-qemu/Dockerfile")).unwrap();
        let original = fs::read_to_string(task_dir.join("Dockerfile")).unwrap();
        assert!(patched.contains("RUN cd linux-6.9 && make -j1"));
        assert!(!patched.contains("gcc-x86-64-linux-gnu"));
        assert!(original.contains("RUN cd linux-6.9 && make -j$(nproc)"));
    }

    #[test]
    fn terminal_bench_runtime_prep_failure_is_structured_task_result() {
        let source = tempfile::tempdir().unwrap();
        let run_dir = tempfile::tempdir().unwrap();
        let attempt_dir = run_dir.path().join("tasks/build-initramfs-qemu/attempts/1");
        fs::create_dir_all(&attempt_dir).unwrap();
        write_qemu_task(
            source.path(),
            "build-initramfs-qemu",
            "FROM ubuntu\nRUN echo drifted\n",
        );
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
        profile.labels.insert(
            "terminal_bench_agent_import_path".to_string(),
            "harnesslab_tb_agent.py".to_string(),
        );
        let spec = run_spec(run_dir.path());
        let task = qemu_task(source.path());
        let materialized_profile = crate::agent_registry::materialize_profile(&profile).unwrap();
        let ctx = super::super::ExternalTaskExecution {
            run_dir: run_dir.path(),
            spec: &spec,
            profile: &profile,
            report_profile: &profile,
            materialized_profile: &materialized_profile,
            task: &task,
            attempt: 1,
            provenance: AttemptProvenance::Original,
            attempt_dir: &attempt_dir,
            started: Instant::now(),
        };

        let result = super::super::terminal_bench::execute(&ctx, source.path()).unwrap();

        assert_eq!(result.state, TaskState::Failure);
        assert_eq!(result.outcome, Outcome::Failure);
        assert_eq!(result.failure_class, FailureClass::Execution);
        assert_eq!(
            result.failure_code,
            Some(FailureCode::ExternalRunnerSetupFailed)
        );
        assert_eq!(result.health_impact, HealthImpact::EnvironmentUnhealthy);
        assert!(result.agent.is_none());
        assert!(
            run_dir
                .path()
                .join("tasks")
                .join(task_dir_name("build-initramfs-qemu").unwrap())
                .join("attempts/1/result.json")
                .exists()
        );
        let events = fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
        assert!(events.contains("external_runner_setup_failed"));
        assert!(!events.contains("external_runner_started"));
        assert!(!events.contains("terminal_bench_cleanup"));
    }

    fn run_spec(run_dir: &Path) -> RunSpec {
        RunSpec {
            schema_version: 1,
            run_id: "qemu-setup-test".to_string(),
            created_at: "2026-06-03T00:00:00Z".to_string(),
            agent_profile_ref: "custom".to_string(),
            benchmark: BenchmarkRef {
                name: "terminal-bench".to_string(),
                version: "0.1.1".to_string(),
                split: "full".to_string(),
            },
            execution: ExecutionConfig {
                concurrency: 1,
                attempts: 1,
                network: NetworkPolicy::Full,
                timeout_sec: None,
            },
            paths: RunPaths {
                run_dir: run_dir.display().to_string(),
            },
            replay_source_run_id: None,
        }
    }

    fn qemu_task(dataset_path: &Path) -> TaskPlan {
        TaskPlan {
            task_id: "build-initramfs-qemu".to_string(),
            instruction: "build qemu".to_string(),
            workspace_spec: WorkspaceSpec {
                workspace_type: WorkspaceType::Empty,
                target_path: ".".to_string(),
                clean: true,
            },
            sandbox_spec: SandboxSpec {
                image: "terminal-bench-official".to_string(),
                mounts: Vec::new(),
                env_vars: Vec::new(),
                network: NetworkPolicy::Full,
                privileged: true,
                resource_limits: ResourceHint {
                    cpu_cores: 2,
                    memory_mb: 4096,
                },
            },
            verifier_spec: VerifierSpec {
                command: "tb run".to_string(),
                working_dir: ".".to_string(),
                timeout_sec: 60,
                expected_exit_codes: vec![0],
                environment_mode: VerifierEnvironment::HostProcess,
                output_parser: "terminal_bench".to_string(),
            },
            artifact_spec: ArtifactSpec {
                base_dir: ".".to_string(),
                globs: Vec::new(),
                required_paths: Vec::new(),
                max_size_bytes: 1,
            },
            patch_spec: None,
            external_runner: Some(ExternalRunnerSpec {
                kind: ExternalRunnerKind::TerminalBench,
                dataset_path: dataset_path.display().to_string(),
                source_path: None,
                agent_timeout_sec: Some(360),
            }),
        }
    }

    const CANONICAL_QEMU_DOCKERFILE: &str = "FROM ubuntu\nRUN apt-get install -y build-essential libncurses-dev bison flex libssl-dev libelf-dev qemu-system bc cpio wget expect\nRUN cd linux-6.9 && make defconfig\nRUN cd linux-6.9 && make olddefconfig\nRUN cd linux-6.9 && make -j$(nproc)\n";

    fn write_qemu_task(root: &Path, task_id: &str, dockerfile: &str) -> std::path::PathBuf {
        let task_dir = root.join(task_id);
        fs::create_dir_all(task_dir.join("tests")).unwrap();
        fs::write(task_dir.join("Dockerfile"), dockerfile).unwrap();
        fs::write(
            task_dir.join("tests/test_outputs.py"),
            "def test_ok(): pass\n",
        )
        .unwrap();
        task_dir
    }
}
