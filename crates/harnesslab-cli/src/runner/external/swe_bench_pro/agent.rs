use super::SweInstance;
use anyhow::Result;
use harnesslab_core::{AgentProfile, FailureCode, ProcessRecord, TaskPlan, TerminationReason};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) struct SweAgentRun {
    pub(super) process: ProcessRecord,
    pub(super) sandbox_failure: Option<FailureCode>,
}

pub(super) fn run_agent(
    ctx: &super::super::ExternalTaskExecution<'_>,
    task: &TaskPlan,
    workspace: &Path,
    instance: &SweInstance,
) -> Result<SweAgentRun> {
    if ctx
        .profile
        .labels
        .get("swe_bench_pro_agent")
        .map(String::as_str)
        == Some("gold")
    {
        return Ok(SweAgentRun {
            process: apply_gold_patch(
                ctx.attempt_dir,
                workspace,
                instance,
                ctx.profile,
                ctx.report_profile,
            )?,
            sandbox_failure: None,
        });
    }
    let mut sandbox_task = task.clone();
    sandbox_task.sandbox_spec.image = super::docker_image(instance);
    write_sandbox_manifest(ctx, &sandbox_task, workspace)?;
    super::append_swe_event(
        ctx,
        "external_runner_agent_sandbox_starting",
        &format!(
            "swe-bench-pro agent sandbox image {}",
            sandbox_task.sandbox_spec.image
        ),
    )?;
    let agent_run =
        super::super::super::sandbox::run_agent(super::super::super::sandbox::AgentRunRequest {
            spec: ctx.spec,
            profile: ctx.profile,
            report_profile: ctx.report_profile,
            materialized_profile: ctx.materialized_profile,
            report_materialized_profile: ctx.report_materialized_profile,
            task: &sandbox_task,
            attempt: ctx.attempt,
            attempt_dir: ctx.attempt_dir,
            workspace,
        })?;
    if let Some(code) = agent_run.sandbox_failure {
        super::append_swe_event(
            ctx,
            "external_runner_agent_sandbox_failed",
            &format!("swe-bench-pro agent sandbox failed {code:?}"),
        )?;
    } else {
        super::append_swe_event(
            ctx,
            "external_runner_agent_sandbox_completed",
            "swe-bench-pro agent sandbox completed",
        )?;
    }
    Ok(SweAgentRun {
        process: agent_run.process,
        sandbox_failure: agent_run.sandbox_failure,
    })
}

fn write_sandbox_manifest(
    ctx: &super::super::ExternalTaskExecution<'_>,
    task: &TaskPlan,
    workspace: &Path,
) -> Result<()> {
    let docker_request = super::super::super::sandbox::docker_create_request(
        ctx.spec,
        ctx.profile,
        task,
        ctx.attempt,
        workspace,
    );
    let manifest = serde_json::json!({
        "image": task.sandbox_spec.image,
        "network": ctx.spec.execution.network,
        "privileged": task.sandbox_spec.privileged,
        "timeout_sec": ctx.spec.execution.timeout_sec.unwrap_or(ctx.profile.timeout_sec),
        "cpu_cores": task.sandbox_spec.resource_limits.cpu_cores,
        "memory_mb": task.sandbox_spec.resource_limits.memory_mb,
        "input_mode": ctx.profile.input_mode,
        "working_dir": ctx.profile.working_dir,
        "effective_docker_request": {
            "run_id": docker_request.run_id,
            "task_id": docker_request.task_id,
            "attempt": docker_request.attempt,
            "image": docker_request.image,
            "workspace_host_path": docker_request.workspace_host_path,
            "workspace_container_path": docker_request.workspace_container_path,
            "network": docker_request.network,
            "env_vars": docker_request.env_vars,
            "mounts": docker_request.mounts.iter().map(|mount| redact_mount(mount)).collect::<Vec<_>>(),
            "privileged": docker_request.privileged,
            "cpu_cores": docker_request.cpu_cores,
            "memory_mb": docker_request.memory_mb
        }
    });
    fs::write(
        ctx.attempt_dir.join("swe-bench-pro/agent-sandbox.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn redact_mount(mount: &str) -> String {
    let mut parts = mount.splitn(2, ':');
    let _host = parts.next();
    match parts.next() {
        Some(rest) => format!("[HOST_PATH_REDACTED]:{rest}"),
        None => "[MOUNT_REDACTED]".to_string(),
    }
}

fn apply_gold_patch(
    attempt_dir: &Path,
    workspace: &Path,
    instance: &SweInstance,
    runtime_profile: &AgentProfile,
    report_profile: &AgentProfile,
) -> Result<ProcessRecord> {
    let agent_dir = attempt_dir.join("agent");
    fs::create_dir_all(&agent_dir)?;
    super::super::write_external_command_snapshot(
        attempt_dir,
        runtime_profile,
        report_profile,
        "git apply -",
    )?;
    let status = Command::new("git")
        .arg("apply")
        .arg("-")
        .current_dir(workspace)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .expect("git apply stdin")
                .write_all(instance.gold_patch.as_bytes())?;
            child.wait_with_output()
        })?;
    fs::write(agent_dir.join("stdout.log"), status.stdout)?;
    fs::write(agent_dir.join("stderr.log"), status.stderr)?;
    Ok(ProcessRecord {
        exit_code: status.status.code(),
        termination_reason: if status.status.code().is_some() {
            TerminationReason::Completed
        } else {
            TerminationReason::Signaled
        },
        stdout_path: "agent/stdout.log".to_string(),
        stderr_path: "agent/stderr.log".to_string(),
    })
}
