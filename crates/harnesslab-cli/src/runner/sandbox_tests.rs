use super::*;
use harnesslab_core::{
    AgentKind, ArtifactSpec, NetworkPolicy, ResourceHint, SandboxSpec, VerifierEnvironment,
    VerifierSpec, WorkspaceSpec, WorkspaceType, default_agent_profile,
};

#[test]
fn sandbox_failure_records_logs_and_failure_code() {
    let tmp = tempfile::tempdir().unwrap();
    let result = sandbox_failure(tmp.path(), anyhow::anyhow!("docker missing")).unwrap();
    assert!(matches!(
        result.sandbox_failure,
        Some(FailureCode::SandboxCreateFailed)
    ));
    assert_eq!(
        result.process.termination_reason,
        TerminationReason::SpawnError
    );
    let stderr = fs::read_to_string(tmp.path().join("agent/stderr.log")).unwrap();
    assert_eq!(stderr, "docker missing");
}

#[test]
fn render_command_covers_stdin_file_and_argument_modes() {
    let tmp = tempfile::tempdir().unwrap();
    let task = task();
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
    assert_eq!(
        render_command(&profile, &task, tmp.path(), false).unwrap(),
        "agent"
    );
    profile.command = "agent {{instruction}}".to_string();
    profile.input_mode = InputMode::Argument;
    assert!(
        render_command(&profile, &task, tmp.path(), false)
            .unwrap()
            .contains("'create file'")
    );
    profile.input_mode = InputMode::File;
    assert!(
        render_command(&profile, &task, tmp.path(), false)
            .unwrap()
            .contains("instruction.txt")
    );
    assert_eq!(
        fs::read_to_string(tmp.path().join("instruction.txt")).unwrap(),
        "create file"
    );
    assert!(
        render_command(&profile, &task, tmp.path(), true)
            .unwrap()
            .contains("'/workspace/instruction.txt'")
    );
}

#[test]
fn command_snapshot_redacts_argument_instruction() {
    let tmp = tempfile::tempdir().unwrap();
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent {{instruction}}");
    profile.input_mode = InputMode::Argument;
    write_agent_command_snapshot(tmp.path(), &profile, "agent 'secret task text'").unwrap();
    let content = fs::read_to_string(tmp.path().join("agent/command.txt")).unwrap();
    assert!(content.contains("[INSTRUCTION_ARGUMENT_REDACTED]"));
    assert!(!content.contains("secret task text"));
}

#[test]
fn docker_command_prefixes_builtin_and_custom_setup() {
    let tmp = tempfile::tempdir().unwrap();
    let task = task();
    let codex = default_agent_profile("codex", AgentKind::Codex, "codex exec -");
    let rendered = render_command(&codex, &task, tmp.path(), true).unwrap();
    assert!(rendered.contains("npm install -g @openai/codex"));
    assert!(rendered.contains("runuser -u harnesslab"));
    assert!(rendered.contains("codex exec -"));
    assert_eq!(
        render_command(&codex, &task, tmp.path(), false).unwrap(),
        "codex exec -"
    );
    let claude = default_agent_profile("claude", AgentKind::ClaudeCode, "claude -p -");
    assert!(
        render_command(&claude, &task, tmp.path(), true)
            .unwrap()
            .contains("npm install -g @anthropic-ai/claude-code")
    );
    let opencode = default_agent_profile("opencode", AgentKind::Opencode, "opencode run -");
    assert!(
        render_command(&opencode, &task, tmp.path(), true)
            .unwrap()
            .contains("npm install -g opencode-ai")
    );
    let pi = default_agent_profile("pi", AgentKind::PiCodingAgent, "pi -");
    assert!(
        render_command(&pi, &task, tmp.path(), true)
            .unwrap()
            .contains("pi -")
    );
    let mut custom = default_agent_profile("custom", AgentKind::Custom, "agent");
    custom.labels.insert(
        "sandbox_setup_command".to_string(),
        "install-agent".to_string(),
    );
    let rendered = render_command(&custom, &task, tmp.path(), true).unwrap();
    assert!(rendered.contains("harnesslab agent setup starting"));
    assert!(rendered.contains("sh -c 'install-agent'"));
    assert!(rendered.contains("harnesslab agent setup failed:"));
    assert!(rendered.contains("agent"));
    custom
        .labels
        .insert("sandbox_setup_command".to_string(), " ".to_string());
    assert!(
        render_command(&custom, &task, tmp.path(), true)
            .unwrap()
            .contains("agent")
    );
}

#[test]
fn docker_request_respects_auth_inherit_and_exclude_paths() {
    let workspace = std::path::PathBuf::from("/tmp/ws");
    let spec = spec(None);
    let task = task();
    let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
    profile.auth.inherit_env = vec!["OPENAI_API_KEY".to_string()];
    profile.auth.include_paths = vec!["~/.codex:/root/.codex:ro".to_string()];
    profile.auth.exclude_paths = vec!["~/.codex".to_string()];

    let excluded = docker_create_request(&spec, &profile, &task, 1, &workspace);
    assert_eq!(excluded.env_vars, vec!["OPENAI_API_KEY"]);
    assert!(excluded.mounts.is_empty());
    profile.auth.inherit = false;
    let disabled = docker_create_request(&spec, &profile, &task, 1, &workspace);
    assert!(disabled.env_vars.is_empty());
    assert!(disabled.mounts.is_empty());
}

#[test]
fn agent_timeout_uses_task_override_marker() {
    let profile = default_agent_profile("fake", AgentKind::Fake, "agent");
    let mut spec = spec(Some(99));
    let mut task = task();
    assert_eq!(agent_timeout(&spec, &profile, &task), 99);
    spec.execution.timeout_sec = None;
    assert_eq!(agent_timeout(&spec, &profile, &task), 3600);
    task.task_id = "agent-timeout-case".to_string();
    assert_eq!(agent_timeout(&spec, &profile, &task), 1);
}

#[test]
fn docker_guard_exposes_handle_and_ignores_destroy_errors_on_drop() {
    let guard = DockerSandboxGuard::new(harnesslab_infra::SandboxHandle {
        container_id: "nonexistent-harnesslab-test-container".to_string(),
        name: "harnesslab-test".to_string(),
        run_id: "run-1".to_string(),
        workspace_container_path: "/workspace".to_string(),
    });
    assert_eq!(
        guard.handle().container_id,
        "nonexistent-harnesslab-test-container"
    );
}

fn task() -> TaskPlan {
    TaskPlan {
        task_id: "task".to_string(),
        instruction: "create file".to_string(),
        workspace_spec: WorkspaceSpec {
            workspace_type: WorkspaceType::Empty,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: SandboxSpec {
            image: "host-fixture".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::None,
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
    }
}

fn spec(timeout_sec: Option<u64>) -> RunSpec {
    RunSpec {
        schema_version: 1,
        run_id: "run-1".to_string(),
        created_at: "2026-05-30T00:00:00Z".to_string(),
        agent_profile_ref: "fake".to_string(),
        benchmark: harnesslab_core::BenchmarkRef {
            name: "fake-terminal".to_string(),
            version: "0".to_string(),
            split: "success".to_string(),
        },
        execution: harnesslab_core::ExecutionConfig {
            concurrency: 4,
            attempts: 1,
            network: NetworkPolicy::Full,
            timeout_sec,
        },
        paths: harnesslab_core::RunPaths {
            run_dir: "/tmp/run".to_string(),
        },
        replay_source_run_id: None,
    }
}
