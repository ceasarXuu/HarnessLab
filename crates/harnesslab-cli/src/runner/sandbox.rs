use anyhow::Result;
use harnesslab_core::{
    AgentProfile, FailureCode, InputMode, RunSpec, TaskPlan, TerminationReason,
    effective_auth_mount_specs,
};
use harnesslab_infra::{DockerCliProvider, DockerCreateRequest, ExecSpec, HostProcessExecutor};
use std::fs;
use std::path::Path;

pub(super) struct AgentExecution {
    pub(super) process: harnesslab_core::ProcessRecord,
    pub(super) sandbox_failure: Option<FailureCode>,
}

pub(super) fn run_agent(
    spec: &RunSpec,
    profile: &AgentProfile,
    report_profile: &AgentProfile,
    task: &TaskPlan,
    attempt: u32,
    attempt_dir: &Path,
    workspace: &Path,
) -> Result<AgentExecution> {
    let uses_docker = task_requires_docker(task);
    let command = render_command(profile, task, workspace, uses_docker)?;
    let report_command = render_command(report_profile, task, workspace, uses_docker)?;
    write_agent_command_snapshot(attempt_dir, report_profile, &report_command)?;
    let exec_spec = ExecSpec {
        command,
        stdin: matches!(profile.input_mode, InputMode::Stdin | InputMode::Tty)
            .then(|| task.instruction.clone()),
        working_dir: workspace.to_path_buf(),
        timeout_sec: agent_timeout(spec, profile, task),
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        stdout_path: attempt_dir.join("agent/stdout.log"),
        stderr_path: attempt_dir.join("agent/stderr.log"),
    };
    if !uses_docker {
        return Ok(AgentExecution {
            process: normalize_agent_paths(HostProcessExecutor::exec(&exec_spec)?),
            sandbox_failure: None,
        });
    }
    let request = docker_create_request(spec, profile, task, attempt, workspace);
    let handle = match DockerCliProvider::create(&request) {
        Ok(handle) => handle,
        Err(error) => return sandbox_failure(attempt_dir, error),
    };
    let sandbox = DockerSandboxGuard::new(handle);
    let process = DockerCliProvider::exec(sandbox.handle(), &exec_spec)?;
    Ok(AgentExecution {
        process: normalize_agent_paths(process),
        sandbox_failure: None,
    })
}

fn sandbox_failure(attempt_dir: &Path, error: anyhow::Error) -> Result<AgentExecution> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::write(agent_dir.join("stdout.log"), "")?;
    fs::write(agent_dir.join("stderr.log"), error.to_string())?;
    Ok(AgentExecution {
        process: harnesslab_core::ProcessRecord {
            exit_code: None,
            termination_reason: TerminationReason::SpawnError,
            stdout_path: "agent/stdout.log".to_string(),
            stderr_path: "agent/stderr.log".to_string(),
        },
        sandbox_failure: Some(FailureCode::SandboxCreateFailed),
    })
}

fn normalize_agent_paths(
    mut process: harnesslab_core::ProcessRecord,
) -> harnesslab_core::ProcessRecord {
    process.stdout_path = "agent/stdout.log".to_string();
    process.stderr_path = "agent/stderr.log".to_string();
    process
}

pub(super) fn docker_create_request(
    spec: &RunSpec,
    profile: &AgentProfile,
    task: &TaskPlan,
    attempt: u32,
    workspace: &Path,
) -> DockerCreateRequest {
    DockerCreateRequest {
        run_id: spec.run_id.clone(),
        task_id: task.task_id.clone(),
        attempt,
        image: task.sandbox_spec.image.clone(),
        workspace_host_path: workspace.to_path_buf(),
        workspace_container_path: "/workspace".to_string(),
        network: spec.execution.network,
        env_vars: merged_env_vars(profile, task),
        mounts: merged_mounts(profile, task),
        privileged: task.sandbox_spec.privileged,
        cpu_cores: task.sandbox_spec.resource_limits.cpu_cores,
        memory_mb: task.sandbox_spec.resource_limits.memory_mb,
    }
}

fn write_agent_command_snapshot(
    attempt_dir: &Path,
    profile: &AgentProfile,
    rendered_command: &str,
) -> Result<()> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    fs::write(
        agent_dir.join("command.txt"),
        format!(
            "template={}\nrendered={}\ninput_mode={:?}\n",
            profile.command,
            redacted_rendered_command(profile, rendered_command),
            profile.input_mode
        ),
    )?;
    Ok(())
}

fn redacted_rendered_command(profile: &AgentProfile, rendered_command: &str) -> String {
    if matches!(profile.input_mode, InputMode::Argument) {
        return "[INSTRUCTION_ARGUMENT_REDACTED]".to_string();
    }
    rendered_command.to_string()
}

fn merged_env_vars(profile: &AgentProfile, task: &TaskPlan) -> Vec<String> {
    let mut values = task.sandbox_spec.env_vars.clone();
    if !profile.auth.inherit {
        return values;
    }
    for name in &profile.auth.inherit_env {
        if !values.contains(name) {
            values.push(name.clone());
        }
    }
    values
}

fn merged_mounts(profile: &AgentProfile, task: &TaskPlan) -> Vec<String> {
    let mut mounts = task.sandbox_spec.mounts.clone();
    for auth in effective_auth_mount_specs(profile) {
        if !mounts.contains(&auth.mount) {
            mounts.push(auth.mount);
        }
    }
    mounts
}

pub(super) fn task_requires_docker(task: &TaskPlan) -> bool {
    !matches!(task.sandbox_spec.image.as_str(), "host" | "host-fixture")
}

pub(super) fn render_command(
    profile: &AgentProfile,
    task: &TaskPlan,
    workspace: &Path,
    uses_docker: bool,
) -> Result<String> {
    let command = match profile.input_mode {
        InputMode::Argument => profile
            .command
            .replace("{{instruction}}", &shell_quote(&task.instruction)),
        InputMode::File => {
            let path = workspace.join("instruction.txt");
            fs::create_dir_all(workspace)?;
            fs::write(&path, &task.instruction)?;
            let instruction_path = if uses_docker {
                "/workspace/instruction.txt".to_string()
            } else {
                path.display().to_string()
            };
            profile
                .command
                .replace("{{instruction_file}}", &shell_quote(&instruction_path))
                .replace("{{instruction}}", &shell_quote(&instruction_path))
        }
        InputMode::Stdin | InputMode::Tty => profile.command.clone(),
    };
    if uses_docker {
        Ok(prefix_docker_setup(profile, &command))
    } else {
        Ok(command)
    }
}

fn prefix_docker_setup(profile: &AgentProfile, command: &str) -> String {
    let Some(setup) = docker_setup_command(profile) else {
        return command.to_string();
    };
    format!("{setup}; {command}")
}

fn docker_setup_command(profile: &AgentProfile) -> Option<String> {
    if let Some(command) = profile.labels.get("sandbox_setup_command") {
        return (!command.trim().is_empty()).then(|| command.clone());
    }
    match profile.kind {
        harnesslab_core::AgentKind::Codex => Some(missing_command_installer(
            "codex",
            "npm install -g @openai/codex",
            "codex",
        )),
        harnesslab_core::AgentKind::ClaudeCode => Some(missing_command_installer(
            "claude",
            "npm install -g @anthropic-ai/claude-code",
            "claude-code",
        )),
        harnesslab_core::AgentKind::Opencode => Some(missing_command_installer(
            "opencode",
            "npm install -g opencode-ai",
            "opencode",
        )),
        _ => None,
    }
}

fn missing_command_installer(binary: &str, install: &str, slug: &str) -> String {
    format!(
        "if ! command -v {binary} >/dev/null 2>&1; then if command -v npm >/dev/null 2>&1; then {install} >/tmp/harnesslab-{slug}-install.log 2>&1 || {{ cat /tmp/harnesslab-{slug}-install.log >&2; exit 127; }}; else echo '{binary} CLI missing and npm unavailable' >&2; exit 127; fi; fi"
    )
}

struct DockerSandboxGuard {
    handle: Option<harnesslab_infra::SandboxHandle>,
}

impl DockerSandboxGuard {
    fn new(handle: harnesslab_infra::SandboxHandle) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    fn handle(&self) -> &harnesslab_infra::SandboxHandle {
        self.handle.as_ref().expect("sandbox handle is present")
    }
}

impl Drop for DockerSandboxGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = DockerCliProvider::destroy(&handle);
        }
    }
}

fn agent_timeout(spec: &RunSpec, profile: &AgentProfile, task: &TaskPlan) -> u64 {
    if task.task_id.contains("agent-timeout") {
        1
    } else {
        spec.execution.timeout_sec.unwrap_or(profile.timeout_sec)
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{
        AgentKind, ArtifactSpec, NetworkPolicy, ResourceHint, SandboxSpec, VerifierEnvironment,
        VerifierSpec, WorkspaceSpec, WorkspaceType, default_agent_profile,
    };

    #[test]
    fn sandbox_failure_records_logs_and_failure_code() {
        let tmp = tempfile::tempdir().unwrap();

        let result = sandbox_failure(tmp.path(), anyhow::anyhow!("docker missing")).unwrap();

        assert_eq!(
            result.sandbox_failure,
            Some(FailureCode::SandboxCreateFailed)
        );
        assert_eq!(
            result.process.termination_reason,
            TerminationReason::SpawnError
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join("agent/stderr.log")).unwrap(),
            "docker missing"
        );
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
        assert!(rendered.ends_with("; codex exec -"));
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
        assert_eq!(
            render_command(&pi, &task, tmp.path(), true).unwrap(),
            "pi -"
        );

        let mut custom = default_agent_profile("custom", AgentKind::Custom, "agent");
        custom.labels.insert(
            "sandbox_setup_command".to_string(),
            "install-agent".to_string(),
        );
        assert_eq!(
            render_command(&custom, &task, tmp.path(), true).unwrap(),
            "install-agent; agent"
        );
        custom
            .labels
            .insert("sandbox_setup_command".to_string(), " ".to_string());
        assert_eq!(
            render_command(&custom, &task, tmp.path(), true).unwrap(),
            "agent"
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
}
