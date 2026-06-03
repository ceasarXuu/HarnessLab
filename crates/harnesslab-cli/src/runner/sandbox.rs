use crate::agent_registry::{MaterializedAgentProfile, wrap_rendered_command};
#[cfg(test)]
use crate::agent_registry::{materialization_error_to_anyhow, materialize_profile};
use crate::runner::sandbox_setup;
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

pub(super) struct AgentRunRequest<'a> {
    pub(super) spec: &'a RunSpec,
    pub(super) profile: &'a AgentProfile,
    pub(super) report_profile: &'a AgentProfile,
    pub(super) materialized_profile: &'a MaterializedAgentProfile,
    pub(super) task: &'a TaskPlan,
    pub(super) attempt: u32,
    pub(super) attempt_dir: &'a Path,
    pub(super) workspace: &'a Path,
}

pub(super) fn run_agent(request: AgentRunRequest<'_>) -> Result<AgentExecution> {
    let AgentRunRequest {
        spec,
        profile,
        report_profile,
        materialized_profile,
        task,
        attempt,
        attempt_dir,
        workspace,
    } = request;
    let uses_docker = task_requires_docker(task);
    let command = render_command_with_materialized(
        profile,
        task,
        workspace,
        uses_docker,
        materialized_profile,
    )?;
    let report_command = render_command_with_materialized(
        report_profile,
        task,
        workspace,
        uses_docker,
        materialized_profile,
    )?;
    write_agent_command_snapshot(attempt_dir, report_profile, &report_command)?;
    if uses_docker {
        sandbox_setup::write_snapshot(attempt_dir, materialized_profile)?;
    }
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
    let sandbox_failure = sandbox_setup::failure_code(attempt_dir, materialized_profile, &process);
    Ok(AgentExecution {
        process: normalize_agent_paths(process),
        sandbox_failure,
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

#[cfg(test)]
pub(super) fn render_command(
    profile: &AgentProfile,
    task: &TaskPlan,
    workspace: &Path,
    uses_docker: bool,
) -> Result<String> {
    let materialized = materialize_profile(profile).map_err(materialization_error_to_anyhow)?;
    render_command_with_materialized(profile, task, workspace, uses_docker, &materialized)
}

pub(super) fn render_command_with_materialized(
    profile: &AgentProfile,
    task: &TaskPlan,
    workspace: &Path,
    uses_docker: bool,
    materialized: &MaterializedAgentProfile,
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
        Ok(sandbox_setup::prefix_command(
            materialized,
            &wrap_rendered_command(&command, materialized.run_as),
        ))
    } else {
        Ok(command)
    }
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
#[path = "sandbox_tests.rs"]
mod tests;
