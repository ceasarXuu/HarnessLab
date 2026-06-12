use super::runtime_snapshot::{
    ExternalRuntimeSnapshotRequest, RuntimeMaterial, RuntimeMaterialValidationScope,
    RuntimePhaseCommand, write_external_runtime_snapshots,
};
use super::terminal_bench_cleanup::TaskCleanupOutcome;
use super::{ExternalTaskExecution, terminal_bench_runtime::TerminalBenchRuntimeAttempt};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use anyhow::Result;
use harnesslab_core::{FailureClass, FailureCode};
use serde_json::Value;
use std::path::Path;

pub(super) const TERMINAL_BENCH_RUNTIME_ADAPTER_VERSION: &str = "terminal-bench-runtime.v1";

pub(super) fn write_terminal_bench_runtime_snapshots(
    ctx: &ExternalTaskExecution<'_>,
    prepared: &TerminalBenchRuntimeAttempt,
    diagnostics: TerminalBenchSnapshotDiagnostics,
) -> Result<()> {
    let (private_diagnostics, public_diagnostics) = diagnostics.into_values();
    write_external_runtime_snapshots(ExternalRuntimeSnapshotRequest {
        run_id: &ctx.spec.run_id,
        attempt_dir: ctx.attempt_dir,
        benchmark: "terminal-bench",
        task_id: &ctx.task.task_id,
        attempt: ctx.attempt,
        adapter_id: "harnesslab.terminal-bench.runtime",
        protocol_authority: ctx
            .task
            .runtime_binding
            .as_ref()
            .map(|binding| binding.authority.clone()),
        adapter_version: TERMINAL_BENCH_RUNTIME_ADAPTER_VERSION,
        network: ctx.spec.execution.network,
        timeout_sec: Some(prepared.process_timeout_sec),
        profile: ctx.profile,
        dataset_path: &prepared.source_dataset_path,
        source_path: None,
        commands: vec![RuntimePhaseCommand {
            phase: "official_runner",
            command: prepared.command.clone(),
            working_dir: ctx.attempt_dir.to_path_buf(),
            timeout_sec: prepared.process_timeout_sec,
            stdout_path: ctx.attempt_dir.join("agent/stdout.log"),
            stderr_path: ctx.attempt_dir.join("agent/stderr.log"),
        }],
        materials: runtime_materials(ctx, prepared),
        public_artifacts: public_artifacts(prepared),
        extra_redaction_refs: runtime_redaction_refs(ctx, prepared),
        private_diagnostics,
        public_diagnostics,
    })
}

pub(super) enum TerminalBenchSnapshotDiagnostics {
    PreExecution,
    PostExecution { private: Value, public: Value },
}

impl TerminalBenchSnapshotDiagnostics {
    pub(super) fn post_execution(
        pre_task: &TaskCleanupOutcome,
        post_task: &TaskCleanupOutcome,
        official_failure_class: FailureClass,
        official_failure_code: Option<FailureCode>,
        final_failure_class: FailureClass,
        final_failure_code: Option<FailureCode>,
        cleanup_overrides_result: bool,
    ) -> Self {
        let final_verdict_effect = final_verdict_effect(post_task, cleanup_overrides_result);
        Self::PostExecution {
            private: serde_json::json!({
                "stage": "post_execution",
                "cleanup": {
                    "final_verdict_effect": final_verdict_effect,
                    "phases": [private_cleanup_phase(pre_task), private_cleanup_phase(post_task)],
                },
                "official_failure": failure_snapshot(official_failure_class, official_failure_code),
                "final_failure": failure_snapshot(final_failure_class, final_failure_code),
            }),
            public: serde_json::json!({
                "stage": "post_execution",
                "cleanup": {
                    "final_verdict_effect": final_verdict_effect,
                    "phases": [public_cleanup_phase(pre_task), public_cleanup_phase(post_task)],
                },
                "official_failure": failure_snapshot(official_failure_class, official_failure_code),
                "final_failure": failure_snapshot(final_failure_class, final_failure_code),
            }),
        }
    }

    fn into_values(self) -> (Option<Value>, Option<Value>) {
        match self {
            Self::PreExecution => (
                Some(serde_json::json!({ "stage": "pre_execution" })),
                Some(serde_json::json!({ "stage": "pre_execution" })),
            ),
            Self::PostExecution { private, public } => (Some(private), Some(public)),
        }
    }
}

fn private_cleanup_phase(outcome: &TaskCleanupOutcome) -> Value {
    serde_json::json!({
        "phase": outcome.phase,
        "required": outcome.required,
        "token": outcome.token,
        "success": outcome.success,
        "projects": outcome.projects,
        "removed": outcome.removed,
        "containers_removed": outcome.containers_removed,
        "networks_removed": outcome.networks_removed,
        "error": outcome.error,
    })
}

fn public_cleanup_phase(outcome: &TaskCleanupOutcome) -> Value {
    serde_json::json!({
        "phase": outcome.phase,
        "required": outcome.required,
        "success": outcome.success,
        "projects_count": outcome.projects.len(),
        "removed_count": outcome.removed.len(),
        "containers_removed": outcome.containers_removed,
        "networks_removed": outcome.networks_removed,
        "has_error": outcome.error.is_some(),
    })
}

fn failure_snapshot(class: FailureClass, code: Option<FailureCode>) -> Value {
    serde_json::json!({
        "class": class,
        "code": code,
    })
}

fn final_verdict_effect(
    post_task: &TaskCleanupOutcome,
    cleanup_overrides_result: bool,
) -> &'static str {
    if cleanup_overrides_result {
        "cleanup_overrode_result"
    } else if post_task.error.is_some() {
        "cleanup_warning_only"
    } else {
        "none"
    }
}

fn runtime_materials(
    ctx: &ExternalTaskExecution<'_>,
    prepared: &TerminalBenchRuntimeAttempt,
) -> Vec<RuntimeMaterial> {
    vec![
        RuntimeMaterial {
            name: "source_dataset",
            path: prepared.source_dataset_path.clone(),
            public_path: None,
            validation_scope: RuntimeMaterialValidationScope::LiveExternal,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "runtime_dataset",
            path: prepared.runtime_dataset_path.clone(),
            public_path: relative_attempt_path(ctx.attempt_dir, &prepared.runtime_dataset_path),
            validation_scope: runtime_dataset_scope(
                ctx.attempt_dir,
                &prepared.runtime_dataset_path,
            ),
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "command_snapshot",
            path: ctx.attempt_dir.join("agent/command.txt"),
            public_path: Some("agent/command.txt".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: false,
        },
        RuntimeMaterial {
            name: "official_result",
            path: prepared.result_path.clone(),
            public_path: official_result_public_path(prepared),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
        RuntimeMaterial {
            name: "runner_stdout",
            path: ctx.attempt_dir.join("agent/stdout.log"),
            public_path: Some("agent/stdout.log".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: false,
        },
        RuntimeMaterial {
            name: "runner_stderr",
            path: ctx.attempt_dir.join("agent/stderr.log"),
            public_path: Some("agent/stderr.log".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: false,
        },
        RuntimeMaterial {
            name: "cleanup_report",
            path: ctx.attempt_dir.join("cleanup-report.json"),
            public_path: Some("cleanup-report.json".to_string()),
            validation_scope: RuntimeMaterialValidationScope::ArchivedAttempt,
            include_in_public: true,
        },
    ]
}

fn runtime_redaction_refs(
    ctx: &ExternalTaskExecution<'_>,
    prepared: &TerminalBenchRuntimeAttempt,
) -> Vec<String> {
    let compatibility = BenchmarkRuntimeCompatibility::from_profile(ctx.profile);
    let mut refs = vec![
        prepared.runtime_dataset_path.display().to_string(),
        prepared.output_root.display().to_string(),
        ctx.profile.command.clone(),
        ctx.report_profile.command.clone(),
    ];
    if let Some(setup_script) = &ctx.materialized_profile.setup_script {
        refs.push(setup_script.clone());
    }
    if let Some(setup_script) = &ctx.report_materialized_profile.setup_script {
        refs.push(setup_script.clone());
    }
    if let Some(path) = compatibility.terminal_bench_agent_import_path {
        refs.push(path);
    }
    if let Some(path) = compatibility.terminal_bench_agent_pythonpath {
        refs.push(path);
    }
    refs
}

fn public_artifacts(prepared: &TerminalBenchRuntimeAttempt) -> Vec<String> {
    let mut artifacts = vec!["cleanup-report.json".to_string()];
    if let Some(path) = official_result_public_path(prepared) {
        artifacts.push(path);
    }
    artifacts
}

fn official_result_public_path(prepared: &TerminalBenchRuntimeAttempt) -> Option<String> {
    if prepared.result_path.ends_with("results.json") {
        Some("official/terminal-bench/results.json".to_string())
    } else {
        None
    }
}

fn runtime_dataset_scope(
    attempt_dir: &Path,
    runtime_dataset: &Path,
) -> RuntimeMaterialValidationScope {
    if runtime_dataset.starts_with(attempt_dir) {
        RuntimeMaterialValidationScope::ArchivedAttempt
    } else {
        RuntimeMaterialValidationScope::LiveExternal
    }
}

fn relative_attempt_path(attempt_dir: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(attempt_dir)
        .ok()
        .map(|path| path.display().to_string())
}
