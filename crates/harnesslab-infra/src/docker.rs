use crate::{ExecSpec, HostProcessExecutor};
use anyhow::{Context, Result, bail};
use harnesslab_core::{NetworkPolicy, ProcessRecord};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResult {
    pub status: String,
    pub message: String,
}

pub struct DockerCliProvider;

trait DockerCommandRunner {
    fn output(&self, args: &[String]) -> Result<DockerCommandOutput>;
}

struct CliDockerRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DockerCommandOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl DockerCommandRunner for CliDockerRunner {
    fn output(&self, args: &[String]) -> Result<DockerCommandOutput> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .context("execute docker CLI")?;
        Ok(DockerCommandOutput {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerCreateRequest {
    pub run_id: String,
    pub task_id: String,
    pub attempt: u32,
    pub image: String,
    pub workspace_host_path: PathBuf,
    pub workspace_container_path: String,
    pub network: NetworkPolicy,
    pub env_vars: Vec<String>,
    pub mounts: Vec<String>,
    pub privileged: bool,
    pub cpu_cores: usize,
    pub memory_mb: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxHandle {
    pub container_id: String,
    pub name: String,
    pub run_id: String,
    pub workspace_container_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupResult {
    pub removed: Vec<String>,
}

impl DockerCliProvider {
    pub fn health_check() -> HealthResult {
        let status = Command::new("docker")
            .arg("info")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(status) if status.success() => HealthResult {
                status: "ok".to_string(),
                message: "Docker daemon reachable".to_string(),
            },
            Ok(_) => HealthResult {
                status: "error".to_string(),
                message: "Docker daemon unavailable".to_string(),
            },
            Err(_) => HealthResult {
                status: "error".to_string(),
                message: "Docker CLI not found".to_string(),
            },
        }
    }

    pub fn create(request: &DockerCreateRequest) -> Result<SandboxHandle> {
        Self::create_with_runner(request, &CliDockerRunner)
    }

    fn create_with_runner(
        request: &DockerCreateRequest,
        runner: &impl DockerCommandRunner,
    ) -> Result<SandboxHandle> {
        let args = Self::create_args(request);
        let output = runner.output(&args).context("create docker sandbox")?;
        if !output.success {
            bail!(
                "docker run failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let container_id = String::from_utf8(output.stdout)
            .context("docker run emitted non-utf8 container id")?
            .trim()
            .to_string();
        if container_id.is_empty() {
            bail!("docker run did not return a container id");
        }
        Ok(SandboxHandle {
            container_id,
            name: container_name(request),
            run_id: request.run_id.clone(),
            workspace_container_path: request.workspace_container_path.clone(),
        })
    }

    pub fn exec(handle: &SandboxHandle, spec: &ExecSpec) -> Result<ProcessRecord> {
        let wrapped = ExecSpec {
            command: docker_shell_command(&Self::exec_args(handle, &spec.command)),
            stdin: spec.stdin.clone(),
            working_dir: spec.working_dir.clone(),
            timeout_sec: spec.timeout_sec,
            stdout_path: spec.stdout_path.clone(),
            stderr_path: spec.stderr_path.clone(),
        };
        HostProcessExecutor::exec(&wrapped)
    }

    pub fn copy_out(
        handle: &SandboxHandle,
        container_path: &str,
        destination: &Path,
    ) -> Result<()> {
        Self::copy_out_with_runner(handle, container_path, destination, &CliDockerRunner)
    }

    fn copy_out_with_runner(
        handle: &SandboxHandle,
        container_path: &str,
        destination: &Path,
        runner: &impl DockerCommandRunner,
    ) -> Result<()> {
        let args = Self::copy_out_args(handle, container_path, &destination.display().to_string());
        let output = runner
            .output(&args)
            .context("copy artifacts out of docker sandbox")?;
        if output.success {
            Ok(())
        } else {
            bail!(
                "docker cp failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    pub fn destroy(handle: &SandboxHandle) -> Result<()> {
        Self::destroy_with_runner(handle, &CliDockerRunner)
    }

    fn destroy_with_runner(
        handle: &SandboxHandle,
        runner: &impl DockerCommandRunner,
    ) -> Result<()> {
        let output = runner
            .output(&Self::destroy_args(handle))
            .context("destroy sandbox")?;
        if output.success {
            Ok(())
        } else {
            bail!(
                "docker rm failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    pub fn cleanup_orphans(run_id: &str) -> Result<CleanupResult> {
        Self::cleanup_orphans_with_runner(run_id, &CliDockerRunner)
    }

    fn cleanup_orphans_with_runner(
        run_id: &str,
        runner: &impl DockerCommandRunner,
    ) -> Result<CleanupResult> {
        let output = runner
            .output(&Self::ps_orphans_args(run_id))
            .context("list orphans")?;
        if !output.success {
            bail!(
                "docker ps failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let stdout =
            String::from_utf8(output.stdout).context("docker ps emitted non-utf8 output")?;
        let mut removed = Vec::new();
        for container_id in stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            let handle = SandboxHandle {
                container_id: container_id.to_string(),
                name: String::new(),
                run_id: run_id.to_string(),
                workspace_container_path: "/workspace".to_string(),
            };
            Self::destroy_with_runner(&handle, runner)?;
            removed.push(container_id.to_string());
        }
        Ok(CleanupResult { removed })
    }

    pub fn create_args(request: &DockerCreateRequest) -> Vec<String> {
        let mut args = vec![
            "run".to_string(),
            "-d".to_string(),
            "--name".to_string(),
            container_name(request),
            "--label".to_string(),
            format!("harnesslab.run_id={}", request.run_id),
            "--label".to_string(),
            format!("harnesslab.task_id={}", request.task_id),
            "--label".to_string(),
            format!("harnesslab.attempt={}", request.attempt),
            "--network".to_string(),
            network_name(request.network).to_string(),
            "-v".to_string(),
            format!(
                "{}:{}",
                request.workspace_host_path.display(),
                request.workspace_container_path
            ),
        ];
        for mount in &request.mounts {
            args.push("-v".to_string());
            args.push(mount.clone());
        }
        for env_var in &request.env_vars {
            args.push("-e".to_string());
            args.push(env_var.clone());
        }
        if request.privileged {
            args.push("--privileged".to_string());
        }
        if request.cpu_cores > 0 {
            args.push("--cpus".to_string());
            args.push(request.cpu_cores.to_string());
        }
        if request.memory_mb > 0 {
            args.push("--memory".to_string());
            args.push(format!("{}m", request.memory_mb));
        }
        args.extend([
            request.image.clone(),
            "sh".to_string(),
            "-lc".to_string(),
            "sleep infinity".to_string(),
        ]);
        args
    }

    pub fn exec_args(handle: &SandboxHandle, command: &str) -> Vec<String> {
        vec![
            "exec".to_string(),
            "-i".to_string(),
            "--workdir".to_string(),
            handle.workspace_container_path.clone(),
            handle.container_id.clone(),
            "sh".to_string(),
            "-lc".to_string(),
            command.to_string(),
        ]
    }

    pub fn copy_out_args(
        handle: &SandboxHandle,
        container_path: &str,
        destination: &str,
    ) -> Vec<String> {
        vec![
            "cp".to_string(),
            format!("{}:{}", handle.container_id, container_path),
            destination.to_string(),
        ]
    }

    pub fn destroy_args(handle: &SandboxHandle) -> Vec<String> {
        vec![
            "rm".to_string(),
            "-f".to_string(),
            handle.container_id.clone(),
        ]
    }

    pub fn ps_orphans_args(run_id: &str) -> Vec<String> {
        vec![
            "ps".to_string(),
            "-aq".to_string(),
            "--filter".to_string(),
            format!("label=harnesslab.run_id={run_id}"),
        ]
    }
}

fn docker_shell_command(args: &[String]) -> String {
    let quoted_args = args
        .iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ");
    format!("docker {quoted_args}")
}

fn container_name(request: &DockerCreateRequest) -> String {
    format!(
        "harnesslab-{}-{}-{}",
        safe_token(&request.run_id),
        safe_token(&request.task_id),
        request.attempt
    )
}

fn network_name(network: NetworkPolicy) -> &'static str {
    match network {
        NetworkPolicy::Full => "bridge",
        NetworkPolicy::None => "none",
    }
}

fn safe_token(value: &str) -> String {
    let mut sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        sanitized.push('x');
    }
    sanitized
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
#[path = "docker_tests.rs"]
mod tests;
