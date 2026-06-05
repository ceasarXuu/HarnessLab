use super::*;
use harnesslab_core::{
    ArtifactSpec, BenchmarkIdentity, BenchmarkRef, ExecutionConfig, ExternalRunnerKind,
    ExternalRunnerSpec, NetworkPolicy, ResourceHint, RunConfigOverrides, RunPaths, SandboxSpec,
    TaskPlan, VerifierEnvironment, VerifierSpec, WorkspaceSpec, WorkspaceType,
};

#[test]
fn cleanup_001_plan_requires_docker_only_for_container_tasks() {
    assert!(!plan_requires_docker(&plan_with_image("host")));
    assert!(!plan_requires_docker(&plan_with_image("host-fixture")));
    assert!(plan_requires_docker(&plan_with_image("ubuntu:24.04")));
}

#[test]
fn cleanup_002_docker_plan_writes_pre_and_post_cleanup_events() {
    let run_dir = tempfile::tempdir().unwrap();
    let spec = run_spec(run_dir.path());
    let plan = plan_with_image("ubuntu:24.04");

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            run_dir.path(),
            &spec,
            &plan,
            ok_cleanup,
            panic_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
    let records = events
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(events.lines().count(), 2);
    assert_eq!(records[0]["event"], "docker_cleanup");
    assert_eq!(
        records[0]["message"],
        "docker cleanup pre_run: removed_count=1 has_error=false"
    );
    assert_eq!(records[1]["event"], "docker_cleanup");
    assert_eq!(
        records[1]["message"],
        "docker cleanup post_run: removed_count=1 has_error=false"
    );
}

#[test]
fn cleanup_003_non_docker_plan_writes_no_events() {
    let run_dir = tempfile::tempdir().unwrap();
    let spec = run_spec(run_dir.path());
    let plan = plan_with_image("host-fixture");

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            run_dir.path(),
            &spec,
            &plan,
            panic_cleanup,
            panic_compose_cleanup,
        );
    }

    assert!(!run_dir.path().join("events.jsonl").exists());
}

#[test]
fn cleanup_004_cleanup_warning_is_recorded() {
    let run_dir = tempfile::tempdir().unwrap();
    let spec = run_spec(run_dir.path());
    let plan = plan_with_image("ubuntu:24.04");

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            run_dir.path(),
            &spec,
            &plan,
            warning_cleanup,
            panic_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.path().join("events.jsonl")).unwrap();
    assert!(events.contains("docker cleanup pre_run: removed_count=0 has_error=true"));
    assert!(events.contains("docker cleanup post_run: removed_count=0 has_error=true"));
    assert!(!events.contains("cleanup unavailable"));
}

#[test]
fn cleanup_005_terminal_bench_pre_run_cleans_completed_sibling_runs() {
    let runs_dir = tempfile::tempdir().unwrap();
    let run_dir = runs_dir.path().join("current-run");
    let old_run = runs_dir.path().join("Old-Terminal-Run");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&old_run).unwrap();
    std::fs::write(
        old_run.join("terminal-bench-compose-projects.json"),
        r#"{"schema_version":1,"projects":["old-project"]}"#,
    )
    .unwrap();
    let spec = run_spec(&run_dir);
    let plan = terminal_bench_plan();

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            &run_dir,
            &spec,
            &plan,
            ok_cleanup,
            ok_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("Old-Terminal-Run"));
    assert!(!events.contains("current-run"));
    assert!(events.contains("projects_count=1"));
    assert_eq!(events.matches("terminal_bench_docker_cleanup").count(), 3);
}

#[test]
fn cleanup_006_terminal_bench_pre_run_ignores_unidentified_siblings() {
    let runs_dir = tempfile::tempdir().unwrap();
    let run_dir = runs_dir.path().join("current-run");
    let old_run = runs_dir.path().join("old-non-terminal-run");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&old_run).unwrap();
    std::fs::write(old_run.join("results.json"), "{}").unwrap();
    let spec = run_spec(&run_dir);
    let plan = terminal_bench_plan();

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            &run_dir,
            &spec,
            &plan,
            ok_cleanup,
            ok_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("run=old-non-terminal-run"));
    assert_eq!(events.matches("terminal_bench_docker_cleanup").count(), 2);
}

#[test]
fn cleanup_007_terminal_bench_pre_run_considers_stale_run_without_snapshot() {
    let runs_dir = tempfile::tempdir().unwrap();
    let run_dir = runs_dir.path().join("current-run");
    let old_run = runs_dir
        .path()
        .join("old-agent-terminal-bench-full-20260601T000000Z");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&old_run).unwrap();
    let spec = run_spec(&run_dir);
    let plan = terminal_bench_plan();

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            &run_dir,
            &spec,
            &plan,
            ok_cleanup,
            ok_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("old-agent-terminal-bench-full-20260601T000000Z"));
    assert!(events.contains("tokens_count=1"));
    assert!(events.contains("projects_count=1"));
    assert_eq!(events.matches("terminal_bench_docker_cleanup").count(), 3);
}

#[test]
fn cleanup_008_terminal_bench_pre_run_uses_stale_run_json_id() {
    let runs_dir = tempfile::tempdir().unwrap();
    let run_dir = runs_dir.path().join("current-run");
    let old_run = runs_dir.path().join("renamed-stale-dir");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&old_run).unwrap();
    let mut old_spec = run_spec(&old_run);
    old_spec.run_id = "Agent.Terminal_Bench-20260602T010203Z".to_string();
    old_spec.benchmark.name = "terminal-bench".to_string();
    harnesslab_infra::atomic_write_json(&old_run.join("run.json"), &old_spec).unwrap();
    let spec = run_spec(&run_dir);
    let plan = terminal_bench_plan();

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            &run_dir,
            &spec,
            &plan,
            ok_cleanup,
            ok_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("renamed-stale-dir"));
    assert!(!events.contains("Agent.Terminal_Bench-20260602T010203Z"));
    assert!(events.contains("terminal-bench docker cleanup pre_run"));
    assert!(events.contains("tokens_count=1"));
}

#[test]
fn cleanup_009_terminal_bench_pre_run_ignores_loose_name_match() {
    let runs_dir = tempfile::tempdir().unwrap();
    let run_dir = runs_dir.path().join("current-run");
    let loose_run = runs_dir.path().join("debug-terminal-bench-notes");
    let pseudo_timestamp_run = runs_dir
        .path()
        .join("debug-terminal-bench-full-20260602TnotesZ");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::create_dir_all(&loose_run).unwrap();
    std::fs::create_dir_all(&pseudo_timestamp_run).unwrap();
    let spec = run_spec(&run_dir);
    let plan = terminal_bench_plan();

    {
        let _cleanup = RunSandboxCleanup::start_with_cleanup(
            &run_dir,
            &spec,
            &plan,
            ok_cleanup,
            ok_compose_cleanup,
        );
    }

    let events = std::fs::read_to_string(run_dir.join("events.jsonl")).unwrap();
    assert!(!events.contains("run=debug-terminal-bench-notes"));
    assert!(!events.contains("run=debug-terminal-bench-full-20260602TnotesZ"));
    assert_eq!(events.matches("terminal_bench_docker_cleanup").count(), 2);
}

fn run_spec(run_dir: &Path) -> RunSpec {
    RunSpec {
        schema_version: 1,
        run_id: "cleanup-test".to_string(),
        created_at: "2026-05-27T00:00:00Z".to_string(),
        agent_profile_ref: "fake".to_string(),
        benchmark: BenchmarkRef {
            name: "fake".to_string(),
            version: "fixture".to_string(),
            split: "smoke".to_string(),
        },
        execution: ExecutionConfig {
            concurrency: 1,
            attempts: 1,
            network: NetworkPolicy::None,
            timeout_sec: None,
        },
        paths: RunPaths {
            run_dir: run_dir.display().to_string(),
        },
        replay_source_run_id: None,
    }
}

fn ok_cleanup(run_id: &str) -> Result<CleanupResult, String> {
    assert_eq!(run_id, "cleanup-test");
    Ok(CleanupResult {
        removed: vec!["container-1".to_string()],
    })
}

fn panic_cleanup(_run_id: &str) -> Result<CleanupResult, String> {
    panic!("cleanup must not run for host plans")
}

fn panic_compose_cleanup(
    _target: &super::super::external::RuntimeCleanupTarget,
) -> Result<super::super::external::RuntimeCleanupReport, String> {
    panic!("compose cleanup must not run")
}

fn ok_compose_cleanup(
    target: &super::super::external::RuntimeCleanupTarget,
) -> Result<super::super::external::RuntimeCleanupReport, String> {
    Ok(super::super::external::RuntimeCleanupReport {
        removed: vec![format!("network:{}", target.run_dir.display())],
        tokens: vec![target.scan_run_id.clone()],
        projects: vec![format!("project-{}", target.scan_run_id)],
        snapshot_projects: 0,
        matched_projects: 1,
    })
}

fn warning_cleanup(run_id: &str) -> Result<CleanupResult, String> {
    assert_eq!(run_id, "cleanup-test");
    Err("cleanup unavailable".to_string())
}

fn plan_with_image(image: &str) -> BenchmarkPlan {
    BenchmarkPlan {
        benchmark: BenchmarkIdentity {
            name: "fake".to_string(),
            version: "fixture".to_string(),
        },
        split: "smoke".to_string(),
        prepared_benchmark_ref: "fixture".to_string(),
        tasks: vec![TaskPlan {
            task_id: "task".to_string(),
            instruction: "instruction".to_string(),
            workspace_spec: WorkspaceSpec {
                workspace_type: WorkspaceType::Empty,
                target_path: "workspace".to_string(),
                clean: true,
            },
            sandbox_spec: SandboxSpec {
                image: image.to_string(),
                mounts: Vec::new(),
                env_vars: Vec::new(),
                network: harnesslab_core::NetworkPolicy::None,
                privileged: false,
                resource_limits: ResourceHint {
                    cpu_cores: 1,
                    memory_mb: 128,
                },
            },
            verifier_spec: VerifierSpec {
                command: "true".to_string(),
                working_dir: "workspace".to_string(),
                timeout_sec: 1,
                expected_exit_codes: vec![0],
                environment_mode: VerifierEnvironment::HostProcess,
                output_parser: "exit_code".to_string(),
            },
            artifact_spec: ArtifactSpec {
                base_dir: "workspace".to_string(),
                globs: Vec::new(),
                required_paths: Vec::new(),
                max_size_bytes: 1,
            },
            patch_spec: None,
            external_runner: None,
        }],
        task_runtime_snapshots: Vec::new(),
        run_config_overrides: RunConfigOverrides {
            timeout_sec: None,
            network: None,
        },
        warnings: Vec::new(),
    }
}

fn terminal_bench_plan() -> BenchmarkPlan {
    let mut plan = plan_with_image("terminal-bench-official");
    plan.tasks[0].external_runner = Some(ExternalRunnerSpec {
        kind: ExternalRunnerKind::TerminalBench,
        dataset_path: "/tmp/terminal-bench".to_string(),
        source_path: None,
        agent_timeout_sec: None,
    });
    plan
}
