use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn scaffold_adapter(
    benchmark_id: &str,
    adapter_id: &str,
    output_dir: &Path,
) -> Result<()> {
    validate_id(benchmark_id, "benchmark-id")?;
    validate_id(adapter_id, "adapter-id")?;

    fs::create_dir_all(output_dir)
        .with_context(|| format!("create output directory {}", output_dir.display()))?;

    let module_name = benchmark_id.replace("-", "_");
    let module_file = output_dir.join(format!("{module_name}.rs"));
    let content = generate_adapter_module(benchmark_id, adapter_id);
    fs::write(&module_file, content)
        .with_context(|| format!("write scaffold module {}", module_file.display()))?;

    let test_file = output_dir.join(format!("{module_name}_tests.rs"));
    fs::write(&test_file, generate_test_module(benchmark_id, adapter_id))
        .with_context(|| format!("write scaffold test {}", test_file.display()))?;

    let binding_file = output_dir.join("registry_binding.rs");
    fs::write(&binding_file, generate_registry_binding(benchmark_id, adapter_id))
        .with_context(|| format!("write registry binding snippet {}", binding_file.display()))?;

    println!(
        "scaffold ok: {} -> {}",
        benchmark_id,
        output_dir.display()
    );
    println!("next step: copy the binding from registry_binding.rs into protocol_registry.rs");
    Ok(())
}

fn validate_id(value: &str, name: &str) -> Result<()> {
    if value.is_empty() {
        bail!("{name} must not be empty");
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
    {
        bail!(
            "{name} must contain only lowercase ascii letters, digits, hyphens, and dots"
        );
    }
    if value.starts_with('-') || value.ends_with('-') || value.starts_with('.') || value.ends_with('.') {
        bail!("{name} must not start or end with a hyphen or dot");
    }
    Ok(())
}

fn generate_adapter_module(benchmark_id: &str, adapter_id: &str) -> String {
    let adapter_struct = to_camel_case(benchmark_id);
    format!(
        r#"use crate::{{
    BenchmarkAdapter, ProtocolAdapterDescriptor, ProtocolArtifactDeclaration,
    ProtocolDataLifecycleContract, ProtocolFailureMapping, ProtocolOperation,
    ProtocolReadinessProbe, ProtocolReportMetadata,
    ProtocolRuntimeLifecycleContract, built_in_protocol_registry,
    prepared_from_descriptor, stable_checksum,
}};
use harnesslab_core::{{
    AdapterId, ArtifactSpec, BenchmarkDescriptor, BenchmarkSplit, BenchmarkStyle, CapabilityId,
    DataState, FailureClass, FailureCode, HealthImpact, NetworkPolicy, PreparedBenchmark,
    ResourceHint, SandboxSpec, SourceRef, TaskDescriptor, TaskPlan, VerifierEnvironment,
    VerifierSpec, WorkspaceSpec, WorkspaceType,
}};

pub struct {adapter_struct}Adapter;

impl BenchmarkAdapter for {adapter_struct}Adapter {{
    fn descriptor(&self) -> BenchmarkDescriptor {{
        BenchmarkDescriptor {{
            name: "{benchmark_id}".to_string(),
            style: BenchmarkStyle::Terminal,
            version: "scaffold.v1".to_string(),
            homepage: "scaffold".to_string(),
            splits: vec![split("smoke")],
        }}
    }}

    fn prepare(&self, split: &str) -> Result<PreparedBenchmark, String> {{
        if split != "smoke" {{
            return Err(format!("unknown {benchmark_id} split {{split}}"));
        }}
        Ok(prepared_from_descriptor(
            self.descriptor(),
            split,
            format!("fixture://{benchmark_id}/{{split}}"),
            1,
        ))
    }}

    fn list_tasks(&self, prepared: &PreparedBenchmark) -> Result<Vec<TaskDescriptor>, String> {{
        if prepared.split != "smoke" {{
            return Err(format!(
                "unknown {benchmark_id} split {{}}",
                prepared.split
            ));
        }}
        Ok(vec![TaskDescriptor {{
            task_id: "{benchmark_id}-smoke".to_string(),
            split: prepared.split.clone(),
            estimated_timeout_sec: 5,
            resource_hint: ResourceHint {{
                cpu_cores: 1,
                memory_mb: 256,
            }},
            source_ref: SourceRef {{
                benchmark: "{benchmark_id}".to_string(),
                upstream_id: "smoke".to_string(),
                checksum: stable_checksum("{benchmark_id}:smoke"),
            }},
        }}])
    }}

    fn create_task_plan(
        &self,
        _prepared: &PreparedBenchmark,
        task: &TaskDescriptor,
    ) -> Result<TaskPlan, String> {{
        if task.task_id != "{benchmark_id}-smoke" {{
            return Err(format!(
                "unknown {benchmark_id} task {{}}",
                task.task_id
            ));
        }}
        Ok(TaskPlan {{
            task_id: task.task_id.clone(),
            instruction: "Return deterministic result.".to_string(),
            workspace_spec: WorkspaceSpec {{
                workspace_type: WorkspaceType::Empty,
                target_path: "workspace".to_string(),
                clean: true,
            }},
            sandbox_spec: SandboxSpec {{
                image: "host-fixture".to_string(),
                mounts: Vec::new(),
                env_vars: Vec::new(),
                network: NetworkPolicy::None,
                privileged: false,
                resource_limits: ResourceHint {{
                    cpu_cores: 1,
                    memory_mb: 256,
                }},
            }},
            verifier_spec: VerifierSpec {{
                command: "echo ok".to_string(),
                working_dir: "workspace".to_string(),
                timeout_sec: 5,
                expected_exit_codes: vec![0],
                environment_mode: VerifierEnvironment::HostProcess,
                output_parser: "exit_code".to_string(),
            }},
            artifact_spec: ArtifactSpec {{
                base_dir: "workspace".to_string(),
                globs: vec!["**/*".to_string()],
                required_paths: Vec::new(),
                max_size_bytes: 1024 * 1024,
            }},
            patch_spec: None,
            external_runner: None,
            runtime_binding: None,
        }})
    }}

    fn protocol_descriptor(&self) -> Option<ProtocolAdapterDescriptor> {{
        let binding = built_in_protocol_registry()
            .binding_for_adapter_id(
                &AdapterId::new("{adapter_id}")
                    .expect("adapter id must be valid"),
            )
            .expect("protocol adapter binding must exist; register it in protocol_registry.rs")
            .clone();
        Some(ProtocolAdapterDescriptor {{
            binding,
            descriptor: self.descriptor(),
            data_lifecycle: data_lifecycle(),
            runtime_lifecycle: runtime_lifecycle(),
            artifacts: artifacts(),
            report_metadata: report_metadata(),
            readiness: readiness_probes(),
            failure_mapping: failure_mappings(),
        }})
    }}
}}

fn split(name: &str) -> BenchmarkSplit {{
    BenchmarkSplit {{
        name: name.to_string(),
        task_count: 1,
        data_state: DataState::Ready,
    }}
}}

fn data_lifecycle() -> ProtocolDataLifecycleContract {{
    ProtocolDataLifecycleContract {{
        inspect_data: operation("inspect_data", "data.lifecycle", "benchmark.root", "data.state"),
        prepare: operation("prepare", "data.lifecycle", "split", "prepared.benchmark"),
        list_tasks: operation(
            "list_tasks",
            "data.lifecycle",
            "prepared.benchmark",
            "task.list",
        ),
        create_task_plan: operation(
            "create_task_plan",
            "data.lifecycle",
            "task.descriptor",
            "task.plan",
        ),
        snapshot_task: operation(
            "snapshot_task",
            "replay.authority",
            "task.plan",
            "runtime.task.snapshot",
        ),
    }}
}}

fn runtime_lifecycle() -> ProtocolRuntimeLifecycleContract {{
    ProtocolRuntimeLifecycleContract {{
        preflight: operation(
            "runtime_preflight",
            "readiness.basic",
            "task.plan",
            "readiness.report",
        ),
        execute: operation("execute", "descriptor", "task.plan", "task.result"),
        cleanup: None,
        snapshot: operation(
            "runtime_snapshot",
            "artifacts.basic",
            "task.result",
            "runtime.snapshot",
        ),
        replay_validate: operation(
            "replay_validate",
            "replay.authority",
            "runtime.snapshot",
            "replay.decision",
        ),
    }}
}}

fn readiness_probes() -> Vec<ProtocolReadinessProbe> {{
    vec![
        readiness_probe(
            "data_ready",
            "data.lifecycle",
            "prepare",
            "blocking",
            "ready|blocked",
            "benchmark data must be prepared before task planning",
            "prepare benchmark data before task planning",
            "data.state",
        ),
        readiness_probe(
            "runtime_preflight",
            "readiness.basic",
            "preflight",
            "blocking",
            "ready|blocked",
            "adapter runtime preflight must pass before execution",
            "fix adapter-specific profile labels or source material before running",
            "runtime.preflight.private",
        ),
    ]
}}

fn failure_mappings() -> Vec<ProtocolFailureMapping> {{
    vec![failure_mapping(
        "runtime_setup",
        "preflight",
        "setup",
        FailureClass::Benchmark,
        FailureCode::ExternalRunnerSetupFailed,
        HealthImpact::EnvironmentUnhealthy,
        "adapter runtime setup failed before benchmark execution",
        "runtime.setup.private",
    )]
}}

fn artifacts() -> Vec<ProtocolArtifactDeclaration> {{
    vec![
        artifact(
            "external_runtime_public",
            "attempt",
            "external-runtime.public.json",
            "runtime_snapshot",
            "public",
            "runtime_snapshot",
            true,
            "structured",
        ),
        artifact(
            "external_runtime_private",
            "attempt",
            "external-runtime.private.json",
            "runtime_snapshot",
            "private",
            "runtime_snapshot",
            true,
            "private_only",
        ),
        artifact(
            "result",
            "attempt",
            "result.json",
            "result",
            "public",
            "result_parse",
            true,
            "scan",
        ),
        artifact(
            "benchmark_snapshot",
            "run",
            "benchmark.snapshot.json",
            "runtime_snapshot",
            "public",
            "replay",
            true,
            "structured",
        ),
        artifact(
            "task_runtime_snapshot",
            "attempt",
            "task-runtime.snapshot.json",
            "runtime_snapshot",
            "public",
            "replay",
            true,
            "structured",
        ),
    ]
}}

fn report_metadata() -> ProtocolReportMetadata {{
    ProtocolReportMetadata {{
        score_fields: vec!["accuracy"],
        public_artifacts: vec!["external_runtime_public", "result"],
        summary_fields: vec!["state", "failure_class", "failure_code"],
        detail_sections: vec![],
    }}
}}

fn operation(
    id: &'static str,
    capability: &'static str,
    input_contract: &'static str,
    output_contract: &'static str,
) -> ProtocolOperation {{
    ProtocolOperation {{
        id,
        capability: CapabilityId::new(capability).expect("valid capability id"),
        input_contract,
        output_contract,
    }}
}}

fn artifact(
    artifact_id: &'static str,
    scope: &'static str,
    path: &'static str,
    artifact_type: &'static str,
    visibility: &'static str,
    producer_phase: &'static str,
    required_for_replay: bool,
    redaction_policy: &'static str,
) -> ProtocolArtifactDeclaration {{
    ProtocolArtifactDeclaration {{
        artifact_id,
        scope,
        path,
        artifact_type,
        visibility,
        producer_phase,
        required_for_replay,
        redaction_policy,
        schema_version: "1",
    }}
}}

fn readiness_probe(
    id: &'static str,
    capability: &'static str,
    phase: &'static str,
    severity: &'static str,
    status_contract: &'static str,
    public_message: &'static str,
    remediation: &'static str,
    private_detail_contract: &'static str,
) -> ProtocolReadinessProbe {{
    ProtocolReadinessProbe {{
        id,
        capability: CapabilityId::new(capability).expect("valid capability id"),
        phase,
        severity,
        status_contract,
        public_message,
        remediation,
        private_detail_contract,
    }}
}}

fn failure_mapping(
    adapter_code: &'static str,
    adapter_phase: &'static str,
    adapter_subphase: &'static str,
    failure_class: FailureClass,
    failure_code: FailureCode,
    health_impact: HealthImpact,
    public_message: &'static str,
    private_diagnostics_contract: &'static str,
) -> ProtocolFailureMapping {{
    ProtocolFailureMapping {{
        adapter_code,
        adapter_phase,
        adapter_subphase,
        failure_class,
        failure_code,
        health_impact,
        public_message,
        private_diagnostics_contract,
    }}
}}
"#,
        adapter_struct = adapter_struct,
        benchmark_id = benchmark_id,
        adapter_id = adapter_id,
    )
}

fn generate_test_module(benchmark_id: &str, _adapter_id: &str) -> String {
    let adapter_struct = to_camel_case(benchmark_id);
    format!(
        r#"#[cfg(test)]
mod tests {{
    use crate::{{{adapter_struct}Adapter, BenchmarkAdapter}};

    #[test]
    fn scaffold_descriptor_is_present() {{
        let adapter = {adapter_struct}Adapter;
        assert!(adapter.protocol_descriptor().is_some());
    }}
}}
"#,
        adapter_struct = adapter_struct,
    )
}

fn generate_registry_binding(benchmark_id: &str, adapter_id: &str) -> String {
    let mode = benchmark_id;
    format!(
        r#"// Copy this binding into the `bindings!` macro call in protocol_registry.rs,
// then recompile to register the new adapter.

binding(
    "{benchmark_id}",
    "{adapter_id}",
    "{benchmark_id}-runtime.v1",
    "{mode}",
    None,
    &[
        "descriptor",
        "data.lifecycle",
        "readiness.basic",
        "artifacts.basic",
        "failure.mapping",
        "replay.authority",
        "report.metadata",
    ],
    AdapterStability::Experimental,
),
"#
    )
}

fn to_camel_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
            }
        })
        .collect()
}
