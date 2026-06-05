use super::metadata::first_parquet;
use super::{SweInstance, docker_image};
use crate::runner::external::ExternalTaskExecution;
use crate::runner::external::runtime_snapshot::{
    ExternalRuntimeSnapshotRequest, RuntimeMaterial, RuntimeMaterialValidationScope,
    RuntimePhaseCommand, write_external_runtime_snapshots,
};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::Result;
use harnesslab_core::ExternalRunnerKind;
use std::path::Path;

const SWE_BENCH_PRO_RUNTIME_ADAPTER_VERSION: &str = "swe-bench-pro-runtime.v1";

pub(super) fn write_swe_runtime_snapshots(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    source_path: &Path,
    swe_dir: &Path,
    workspace: &Path,
    instance: &SweInstance,
) -> Result<()> {
    let attempt_root = std::fs::canonicalize(ctx.attempt_dir)?;
    write_external_runtime_snapshots(ExternalRuntimeSnapshotRequest {
        run_id: &ctx.spec.run_id,
        attempt_dir: ctx.attempt_dir,
        benchmark: "swe-bench-pro",
        task_id: &ctx.task.task_id,
        attempt: ctx.attempt,
        runner_kind: ExternalRunnerKind::SweBenchPro,
        adapter_version: SWE_BENCH_PRO_RUNTIME_ADAPTER_VERSION,
        network: ctx.spec.execution.network,
        timeout_sec: ctx.spec.execution.timeout_sec.or(Some(7200)),
        profile: ctx.profile,
        dataset_path,
        source_path: Some(source_path),
        commands: swe_runtime_commands(ctx, source_path, swe_dir, &attempt_root, instance)?,
        materials: swe_runtime_materials(ctx, source_path, swe_dir, instance),
        public_artifacts: vec![
            "swe-bench-pro/raw_sample.jsonl".to_string(),
            "swe-bench-pro/instance.json".to_string(),
            "swe-bench-pro/workspace-manifest.json".to_string(),
            "prediction.jsonl".to_string(),
            "prediction.eval.json".to_string(),
            "patch.diff".to_string(),
            "verifier/stdout.log".to_string(),
            "verifier/stderr.log".to_string(),
        ],
        extra_redaction_refs: vec![
            swe_dir.display().to_string(),
            workspace.display().to_string(),
        ],
        private_diagnostics: None,
        public_diagnostics: None,
    })
}

pub(super) fn write_swe_setup_failure_snapshots(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    source_path: Option<&Path>,
    swe_dir: &Path,
    workspace: &Path,
) -> Result<()> {
    write_external_runtime_snapshots(ExternalRuntimeSnapshotRequest {
        run_id: &ctx.spec.run_id,
        attempt_dir: ctx.attempt_dir,
        benchmark: "swe-bench-pro",
        task_id: &ctx.task.task_id,
        attempt: ctx.attempt,
        runner_kind: ExternalRunnerKind::SweBenchPro,
        adapter_version: SWE_BENCH_PRO_RUNTIME_ADAPTER_VERSION,
        network: ctx.spec.execution.network,
        timeout_sec: ctx.spec.execution.timeout_sec.or(Some(7200)),
        profile: ctx.profile,
        dataset_path,
        source_path,
        commands: setup_failure_commands(ctx, dataset_path, source_path, swe_dir, workspace),
        materials: setup_failure_materials(ctx, dataset_path, source_path, swe_dir),
        public_artifacts: vec![
            "swe-bench-pro/raw_sample.jsonl".to_string(),
            "swe-bench-pro/instance.json".to_string(),
            "swe-bench-pro/workspace-manifest.json".to_string(),
            "verifier/stdout.log".to_string(),
            "verifier/stderr.log".to_string(),
        ],
        extra_redaction_refs: vec![
            swe_dir.display().to_string(),
            workspace.display().to_string(),
        ],
        private_diagnostics: None,
        public_diagnostics: None,
    })
}

pub(super) fn metadata_extract_command(
    script_path: &Path,
    parquet: &Path,
    task_id: &str,
    raw_sample: &Path,
    instance_json: &Path,
) -> String {
    format!(
        "unset PYTHONHOME PYTHONPATH PYTHONUSERBASE; export PYTHONNOUSERSITE=1; uv run --with pandas --with pyarrow python {} {} {} {} {}",
        shell_quote(&script_path.display().to_string()),
        shell_quote(&parquet.display().to_string()),
        shell_quote(task_id),
        shell_quote(&raw_sample.display().to_string()),
        shell_quote(&instance_json.display().to_string())
    )
}

pub(super) fn workspace_prepare_command(workspace: &Path, instance: &SweInstance) -> String {
    format!(
        "set -e; {}; image={}; docker pull --platform linux/amd64 \"$image\"; cid=$(docker create --platform linux/amd64 \"$image\"); trap 'docker rm -f \"$cid\" >/dev/null 2>&1 || true' EXIT; docker cp \"$cid:/app/.\" {}; cd {}; git config user.email harnesslab@example.invalid; git config user.name HarnessLab",
        docker_host_prefix(),
        shell_quote(&docker_image(instance)),
        shell_quote(&workspace.display().to_string()),
        shell_quote(&workspace.display().to_string())
    )
}

pub(super) fn evaluator_command(source_path: &Path, swe_dir: &Path, attempt_root: &Path) -> String {
    format!(
        "set -e; {}; unset PYTHONHOME PYTHONPATH PYTHONUSERBASE; export PYTHONNOUSERSITE=1; uv run --with pandas --with tqdm --with docker python {} --raw_sample_path {} --patch_path {} --output_dir {} --scripts_dir {} --dockerhub_username jefzda --use_local_docker --docker_platform linux/amd64 --num_workers 1 --redo",
        docker_host_prefix(),
        shell_quote(
            &source_path
                .join("swe_bench_pro_eval.py")
                .display()
                .to_string()
        ),
        shell_quote(&swe_dir.join("raw_sample.jsonl").display().to_string()),
        shell_quote(
            &attempt_root
                .join("prediction.eval.json")
                .display()
                .to_string()
        ),
        shell_quote(&swe_dir.join("eval").display().to_string()),
        shell_quote(&source_path.join("run_scripts").display().to_string()),
    )
}

fn swe_runtime_commands(
    ctx: &ExternalTaskExecution<'_>,
    source_path: &Path,
    swe_dir: &Path,
    attempt_root: &Path,
    instance: &SweInstance,
) -> Result<Vec<RuntimePhaseCommand>> {
    Ok(vec![
        RuntimePhaseCommand {
            phase: "metadata_extraction",
            command: metadata_extract_command(
                &swe_dir.join("extract_instance.py"),
                &instance.parquet_path,
                &ctx.task.task_id,
                &swe_dir.join("raw_sample.jsonl"),
                &swe_dir.join("instance.json"),
            ),
            working_dir: swe_dir.to_path_buf(),
            timeout_sec: 300,
            stdout_path: swe_dir.join("metadata.stdout.log"),
            stderr_path: swe_dir.join("metadata.stderr.log"),
        },
        RuntimePhaseCommand {
            phase: "workspace_preparation",
            command: workspace_prepare_command(&attempt_root.join("workspace"), instance),
            working_dir: swe_dir.to_path_buf(),
            timeout_sec: 1800,
            stdout_path: swe_dir.join("workspace.stdout.log"),
            stderr_path: swe_dir.join("workspace.stderr.log"),
        },
        RuntimePhaseCommand {
            phase: "agent_execution",
            command: agent_execution_command(ctx, instance),
            working_dir: attempt_root.join("workspace"),
            timeout_sec: ctx
                .spec
                .execution
                .timeout_sec
                .unwrap_or(ctx.profile.timeout_sec),
            stdout_path: attempt_root.join("agent/stdout.log"),
            stderr_path: attempt_root.join("agent/stderr.log"),
        },
        RuntimePhaseCommand {
            phase: "evaluation",
            command: evaluator_command(source_path, swe_dir, attempt_root),
            working_dir: source_path.to_path_buf(),
            timeout_sec: ctx.task.verifier_spec.timeout_sec,
            stdout_path: attempt_root.join("verifier/stdout.log"),
            stderr_path: attempt_root.join("verifier/stderr.log"),
        },
    ])
}

fn setup_failure_commands(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    source_path: Option<&Path>,
    swe_dir: &Path,
    workspace: &Path,
) -> Vec<RuntimePhaseCommand> {
    let parquet = first_parquet(dataset_path);
    let metadata_command = parquet
        .as_ref()
        .map(|path| {
            metadata_extract_command(
                &swe_dir.join("extract_instance.py"),
                path,
                &ctx.task.task_id,
                &swe_dir.join("raw_sample.jsonl"),
                &swe_dir.join("instance.json"),
            )
        })
        .unwrap_or_else(|| {
            "swe-bench-pro metadata extraction blocked: parquet missing".to_string()
        });
    let mut commands = vec![RuntimePhaseCommand {
        phase: "metadata_extraction",
        command: metadata_command,
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 300,
        stdout_path: swe_dir.join("metadata.stdout.log"),
        stderr_path: swe_dir.join("metadata.stderr.log"),
    }];
    commands.push(RuntimePhaseCommand {
        phase: "source_path_validation",
        command: source_path
            .map(|path| format!("source_path={}", shell_quote(&path.display().to_string())))
            .unwrap_or_else(|| "source_path missing".to_string()),
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 0,
        stdout_path: ctx.attempt_dir.join("verifier/stdout.log"),
        stderr_path: ctx.attempt_dir.join("verifier/stderr.log"),
    });
    commands.push(RuntimePhaseCommand {
        phase: "workspace_preparation",
        command: "swe-bench-pro workspace preparation pending metadata".to_string(),
        working_dir: workspace.to_path_buf(),
        timeout_sec: 1800,
        stdout_path: swe_dir.join("workspace.stdout.log"),
        stderr_path: swe_dir.join("workspace.stderr.log"),
    });
    commands
}

fn setup_failure_materials(
    ctx: &ExternalTaskExecution<'_>,
    dataset_path: &Path,
    source_path: Option<&Path>,
    swe_dir: &Path,
) -> Vec<RuntimeMaterial> {
    let parquet =
        first_parquet(dataset_path).unwrap_or_else(|| dataset_path.join("data/[missing].parquet"));
    let mut materials = vec![
        RuntimeMaterial {
            name: "parquet",
            path: parquet,
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "raw_sample",
            path: swe_dir.join("raw_sample.jsonl"),
            public_path: Some("swe-bench-pro/raw_sample.jsonl".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "instance",
            path: swe_dir.join("instance.json"),
            public_path: Some("swe-bench-pro/instance.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
    ];
    if let Some(source_path) = source_path {
        materials.push(RuntimeMaterial {
            name: "evaluator",
            path: source_path.join("swe_bench_pro_eval.py"),
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        });
    }
    materials.push(RuntimeMaterial {
        name: "verifier_stderr",
        path: ctx.attempt_dir.join("verifier/stderr.log"),
        public_path: Some("verifier/stderr.log".to_string()),
        validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
        include_in_public: true,
    });
    materials
}

fn agent_execution_command(ctx: &ExternalTaskExecution<'_>, instance: &SweInstance) -> String {
    if BenchmarkRuntimeCompatibility::from_profile(ctx.profile).swe_bench_pro_uses_gold_agent() {
        "git apply -".to_string()
    } else {
        format!(
            "harnesslab sandbox agent image={} command={}",
            shell_quote(&docker_image(instance)),
            shell_quote(&ctx.profile.command)
        )
    }
}

fn docker_host_prefix() -> &'static str {
    "if [ -z \"${DOCKER_HOST:-}\" ] && [ -S \"$HOME/.colima/default/docker.sock\" ]; then export DOCKER_HOST=\"unix://$HOME/.colima/default/docker.sock\"; fi"
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn swe_runtime_materials(
    ctx: &ExternalTaskExecution<'_>,
    source_path: &Path,
    swe_dir: &Path,
    instance: &SweInstance,
) -> Vec<RuntimeMaterial> {
    let mut materials = vec![
        RuntimeMaterial {
            name: "parquet",
            path: instance.parquet_path.clone(),
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "evaluator",
            path: source_path.join("swe_bench_pro_eval.py"),
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "run_script",
            path: source_path
                .join("run_scripts")
                .join(&ctx.task.task_id)
                .join("run_script.sh"),
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "raw_sample",
            path: swe_dir.join("raw_sample.jsonl"),
            public_path: Some("swe-bench-pro/raw_sample.jsonl".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "instance",
            path: swe_dir.join("instance.json"),
            public_path: Some("swe-bench-pro/instance.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "workspace_manifest",
            path: swe_dir.join("workspace-manifest.json"),
            public_path: Some("swe-bench-pro/workspace-manifest.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
    ];
    let _ = instance;
    materials.extend([
        RuntimeMaterial {
            name: "prediction_jsonl",
            path: ctx.attempt_dir.join("prediction.jsonl"),
            public_path: Some("prediction.jsonl".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "prediction_eval_json",
            path: ctx.attempt_dir.join("prediction.eval.json"),
            public_path: Some("prediction.eval.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "patch_diff",
            path: ctx.attempt_dir.join("patch.diff"),
            public_path: Some("patch.diff".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "eval_results",
            path: swe_dir.join("eval/eval_results.json"),
            public_path: Some("swe-bench-pro/eval/eval_results.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
    ]);
    materials
}
