use super::runtime_anchor::{AnchorProjection, anchor_attempt_snapshot};
use crate::runner::store;
use anyhow::Result;
use harnesslab_core::{ExternalRunnerKind, redact_public_value};
use harnesslab_infra::{atomic_write_json, stable_checksum_bytes, stable_file_checksum};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct ExternalRuntimeSnapshotRequest<'a> {
    pub(super) run_id: &'a str,
    pub(super) attempt_dir: &'a Path,
    pub(super) benchmark: &'a str,
    pub(super) task_id: &'a str,
    pub(super) attempt: u32,
    pub(super) runner_kind: ExternalRunnerKind,
    pub(super) adapter_version: &'a str,
    pub(super) network: harnesslab_core::NetworkPolicy,
    pub(super) timeout_sec: Option<u64>,
    pub(super) profile: &'a harnesslab_core::AgentProfile,
    pub(super) dataset_path: &'a Path,
    pub(super) source_path: Option<&'a Path>,
    pub(super) commands: Vec<RuntimePhaseCommand>,
    pub(super) materials: Vec<RuntimeMaterial>,
    pub(super) public_artifacts: Vec<String>,
    pub(super) extra_redaction_refs: Vec<String>,
}

pub(super) struct RuntimePhaseCommand {
    pub(super) phase: &'static str,
    pub(super) command: String,
    pub(super) working_dir: PathBuf,
    pub(super) timeout_sec: u64,
    pub(super) stdout_path: PathBuf,
    pub(super) stderr_path: PathBuf,
}

pub(super) struct RuntimeMaterial {
    pub(super) name: &'static str,
    pub(super) path: PathBuf,
    pub(super) public_path: Option<String>,
    pub(super) validation_scope: RuntimeMaterialValidationScope,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RuntimeMaterialValidationScope {
    LiveExternal,
    ArchivedAttempt,
}

pub(super) fn write_external_runtime_snapshots(
    request: ExternalRuntimeSnapshotRequest<'_>,
) -> Result<()> {
    let redaction_refs = redaction_refs(&request);
    let secret_refs = redaction_refs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let materials = request
        .materials
        .iter()
        .map(|material| material_snapshot(material, request.attempt_dir))
        .collect::<Vec<_>>();
    let runtime_policy = runtime_policy(&request);
    let private_commands = request
        .commands
        .iter()
        .map(private_command_snapshot)
        .collect::<Vec<_>>();
    let public_commands = request
        .commands
        .iter()
        .map(|command| public_command_snapshot(command, &secret_refs, request.attempt_dir))
        .collect::<Vec<_>>();
    let public_materials = materials
        .iter()
        .map(public_material_snapshot)
        .collect::<Vec<_>>();
    let runtime_fingerprint =
        runtime_fingerprint(&request, &runtime_policy, &private_commands, &materials)?;
    let public_fingerprint = public_fingerprint(
        &request,
        &runtime_policy,
        &public_commands,
        &public_materials,
        &runtime_fingerprint,
    )?;
    let private = PrivateExternalRuntimeSnapshot {
        schema_version: 1,
        visibility: "private".to_string(),
        benchmark: request.benchmark.to_string(),
        task_id: request.task_id.to_string(),
        attempt: request.attempt,
        runner_kind: request.runner_kind,
        adapter_version: request.adapter_version.to_string(),
        runtime_policy: runtime_policy.clone(),
        dataset_path: request.dataset_path.display().to_string(),
        source_path: request.source_path.map(|path| path.display().to_string()),
        commands: private_commands,
        replay_materials: materials.clone(),
        public_artifacts: request.public_artifacts.clone(),
        runtime_fingerprint: runtime_fingerprint.clone(),
        public_fingerprint,
        redaction_basis: private_redaction_basis(),
    };
    let public = PublicExternalRuntimeSnapshot {
        schema_version: 1,
        visibility: "public".to_string(),
        benchmark: request.benchmark.to_string(),
        task_id: request.task_id.to_string(),
        attempt: request.attempt,
        runner_kind: request.runner_kind,
        adapter_version: request.adapter_version.to_string(),
        runtime_policy,
        commands: public_commands,
        runtime_materials: public_materials,
        public_artifacts: request.public_artifacts.clone(),
        runtime_fingerprint,
    };
    let private_path = request.attempt_dir.join("external-runtime.private.json");
    atomic_write_json(&private_path, &private)?;
    restrict_private_snapshot(&private_path)?;
    let public_path = request.attempt_dir.join("external-runtime.public.json");
    atomic_write_json(&public_path, &public)?;
    let private_checksum = stable_file_checksum(&private_path);
    let public_checksum = stable_file_checksum(&public_path);
    anchor_attempt_snapshot(AnchorProjection {
        run_id: request.run_id.to_string(),
        task_id: request.task_id.to_string(),
        attempt: request.attempt,
        attempt_dir: request.attempt_dir.to_path_buf(),
        private_path,
        public_path,
        private_checksum,
        public_checksum,
        runtime_fingerprint: private.runtime_fingerprint.clone(),
        public_fingerprint: private.public_fingerprint.clone(),
    })?;
    Ok(())
}

#[derive(Serialize)]
struct PrivateExternalRuntimeSnapshot {
    schema_version: u32,
    visibility: String,
    benchmark: String,
    task_id: String,
    attempt: u32,
    runner_kind: ExternalRunnerKind,
    adapter_version: String,
    runtime_policy: RuntimePolicySnapshot,
    dataset_path: String,
    source_path: Option<String>,
    commands: Vec<PrivateCommandSnapshot>,
    replay_materials: Vec<MaterialSnapshot>,
    public_artifacts: Vec<String>,
    runtime_fingerprint: String,
    public_fingerprint: String,
    redaction_basis: Vec<String>,
}

#[derive(Serialize)]
struct PublicExternalRuntimeSnapshot {
    schema_version: u32,
    visibility: String,
    benchmark: String,
    task_id: String,
    attempt: u32,
    runner_kind: ExternalRunnerKind,
    adapter_version: String,
    runtime_policy: RuntimePolicySnapshot,
    commands: Vec<PublicCommandSnapshot>,
    runtime_materials: Vec<PublicMaterialSnapshot>,
    public_artifacts: Vec<String>,
    runtime_fingerprint: String,
}

#[derive(Clone, Serialize)]
struct RuntimePolicySnapshot {
    network: harnesslab_core::NetworkPolicy,
    timeout_sec: Option<u64>,
    agent_profile: String,
    agent_kind: String,
}

#[derive(Clone, Serialize)]
struct PrivateCommandSnapshot {
    phase: String,
    command: String,
    working_dir: String,
    timeout_sec: u64,
    stdout_path: String,
    stderr_path: String,
}

#[derive(Serialize)]
struct PublicCommandSnapshot {
    phase: String,
    command: String,
    timeout_sec: u64,
    stdout_path: String,
    stderr_path: String,
}

#[derive(Clone, Serialize)]
struct MaterialSnapshot {
    name: String,
    path: String,
    public_path: Option<String>,
    validation_scope: RuntimeMaterialValidationScope,
    exists: bool,
    size_bytes: Option<u64>,
    checksum: String,
}

#[derive(Serialize)]
struct PublicMaterialSnapshot {
    name: String,
    public_path: Option<String>,
    validation_scope: RuntimeMaterialValidationScope,
    exists: bool,
    size_bytes: Option<u64>,
    checksum: String,
}

fn runtime_policy(request: &ExternalRuntimeSnapshotRequest<'_>) -> RuntimePolicySnapshot {
    RuntimePolicySnapshot {
        network: request.network,
        timeout_sec: request.timeout_sec,
        agent_profile: request.profile.name.clone(),
        agent_kind: format!("{:?}", request.profile.kind),
    }
}

fn private_command_snapshot(command: &RuntimePhaseCommand) -> PrivateCommandSnapshot {
    PrivateCommandSnapshot {
        phase: command.phase.to_string(),
        command: command.command.clone(),
        working_dir: command.working_dir.display().to_string(),
        timeout_sec: command.timeout_sec,
        stdout_path: command.stdout_path.display().to_string(),
        stderr_path: command.stderr_path.display().to_string(),
    }
}

fn public_command_snapshot(
    command: &RuntimePhaseCommand,
    secret_refs: &[&str],
    attempt_dir: &Path,
) -> PublicCommandSnapshot {
    PublicCommandSnapshot {
        phase: command.phase.to_string(),
        command: redact_public_value(&command.command, secret_refs),
        timeout_sec: command.timeout_sec,
        stdout_path: public_path(&command.stdout_path, attempt_dir),
        stderr_path: public_path(&command.stderr_path, attempt_dir),
    }
}

fn material_snapshot(material: &RuntimeMaterial, attempt_dir: &Path) -> MaterialSnapshot {
    let metadata = fs::metadata(&material.path).ok();
    MaterialSnapshot {
        name: material.name.to_string(),
        path: material.path.display().to_string(),
        public_path: material
            .public_path
            .clone()
            .or_else(|| relative_to_attempt(&material.path, attempt_dir)),
        validation_scope: material.validation_scope,
        exists: metadata.is_some(),
        size_bytes: metadata.as_ref().map(std::fs::Metadata::len),
        checksum: stable_file_checksum(&material.path),
    }
}

fn public_material_snapshot(material: &MaterialSnapshot) -> PublicMaterialSnapshot {
    PublicMaterialSnapshot {
        name: material.name.clone(),
        public_path: material.public_path.clone(),
        validation_scope: material.validation_scope,
        exists: material.exists,
        size_bytes: material.size_bytes,
        checksum: material.checksum.clone(),
    }
}

fn public_path(path: &Path, attempt_dir: &Path) -> String {
    relative_to_attempt(path, attempt_dir).unwrap_or_else(|| "[PRIVATE_PATH]".to_string())
}

fn relative_to_attempt(path: &Path, attempt_dir: &Path) -> Option<String> {
    path.strip_prefix(attempt_dir)
        .ok()
        .map(|path| path.display().to_string())
}

fn redaction_refs(request: &ExternalRuntimeSnapshotRequest<'_>) -> Vec<String> {
    let mut refs = store::secret_values(request.profile);
    push_path_refs(&mut refs, &request.attempt_dir.display().to_string());
    push_path_refs(&mut refs, &request.dataset_path.display().to_string());
    if let Some(source_path) = request.source_path {
        push_path_refs(&mut refs, &source_path.display().to_string());
    }
    for value in &request.extra_redaction_refs {
        push_path_refs(&mut refs, value);
    }
    refs
}

fn push_path_refs(refs: &mut Vec<String>, value: &str) {
    push_ref(refs, value.to_string());
    let escaped = value.replace('\'', "'\\''");
    push_ref(refs, escaped.clone());
    push_ref(refs, format!("'{escaped}'"));
}

fn push_ref(refs: &mut Vec<String>, value: String) {
    if !value.is_empty() && !refs.contains(&value) {
        refs.push(value);
    }
}

fn private_redaction_basis() -> Vec<String> {
    vec![
        "runtime profile inherited environment values".to_string(),
        "local dataset/source/attempt paths".to_string(),
        "sensitive token scanner".to_string(),
    ]
}

fn runtime_fingerprint(
    request: &ExternalRuntimeSnapshotRequest<'_>,
    runtime_policy: &RuntimePolicySnapshot,
    commands: &[PrivateCommandSnapshot],
    materials: &[MaterialSnapshot],
) -> Result<String> {
    stable_json_checksum(&serde_json::json!({
        "schema_version": 1,
        "benchmark": request.benchmark,
        "task_id": request.task_id,
        "attempt": request.attempt,
        "runner_kind": request.runner_kind,
        "adapter_version": request.adapter_version,
        "runtime_policy": runtime_policy,
        "dataset_path": request.dataset_path.display().to_string(),
        "source_path": request.source_path.map(|path| path.display().to_string()),
        "commands": commands,
        "replay_materials": materials,
        "public_artifacts": &request.public_artifacts,
    }))
}

fn public_fingerprint(
    request: &ExternalRuntimeSnapshotRequest<'_>,
    runtime_policy: &RuntimePolicySnapshot,
    commands: &[PublicCommandSnapshot],
    materials: &[PublicMaterialSnapshot],
    runtime_fingerprint: &str,
) -> Result<String> {
    stable_json_checksum(&serde_json::json!({
        "schema_version": 1,
        "visibility": "public",
        "benchmark": request.benchmark,
        "task_id": request.task_id,
        "attempt": request.attempt,
        "runner_kind": request.runner_kind,
        "adapter_version": request.adapter_version,
        "runtime_policy": runtime_policy,
        "commands": commands,
        "runtime_materials": materials,
        "public_artifacts": &request.public_artifacts,
        "runtime_fingerprint": runtime_fingerprint,
    }))
}

fn stable_json_checksum(value: &Value) -> Result<String> {
    Ok(stable_checksum_bytes(&serde_json::to_vec(value)?))
}

#[cfg(unix)]
fn restrict_private_snapshot(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_private_snapshot(_path: &Path) -> Result<()> {
    Ok(())
}
