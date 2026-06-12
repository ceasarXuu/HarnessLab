use crate::{
    BenchmarkAdapter, DeterministicSampleAdapter, SweBenchProAdapter, TerminalBenchAdapter,
    built_in_protocol_adapter_descriptors, validate_artifact_contracts,
    validate_data_lifecycle_contracts, validate_runtime_lifecycle_contracts,
};
use harnesslab_core::{CapabilityId, FailureClass, FailureCode};

#[test]
fn adapt_protocol_003_data_lifecycle_contract_foundation_is_validated() {
    let descriptors = built_in_protocol_adapter_descriptors();
    validate_data_lifecycle_contracts(&descriptors).unwrap();
    assert_eq!(descriptors.len(), 3);
    assert_eq!(
        TerminalBenchAdapter::new()
            .protocol_descriptor()
            .unwrap()
            .binding
            .adapter_id,
        descriptors[0].binding.adapter_id
    );
    assert_eq!(
        SweBenchProAdapter::new()
            .protocol_descriptor()
            .unwrap()
            .binding
            .adapter_id,
        descriptors[1].binding.adapter_id
    );
    assert!(descriptors.iter().all(|descriptor| {
        descriptor
            .data_lifecycle
            .snapshot_task
            .output_contract
            .contains("runtime.task.snapshot")
    }));

    let mut missing_capability = descriptors.clone();
    missing_capability[0].data_lifecycle.prepare.capability =
        CapabilityId::new("undeclared.lifecycle").unwrap();
    let error = validate_data_lifecycle_contracts(&missing_capability).unwrap_err();
    assert!(error.contains("data_lifecycle_capability_missing"));

    let mut mismatched_descriptor = descriptors.clone();
    mismatched_descriptor[0].descriptor.name = "swe-bench-pro".to_string();
    let error = validate_data_lifecycle_contracts(&mismatched_descriptor).unwrap_err();
    assert!(error.contains("protocol_descriptor_mismatch"));

    let mut duplicate_adapter = descriptors.clone();
    duplicate_adapter[1].binding.adapter_id = duplicate_adapter[0].binding.adapter_id.clone();
    let error = validate_data_lifecycle_contracts(&duplicate_adapter).unwrap_err();
    assert!(error.contains("duplicate protocol adapter descriptor"));

    let mut empty_splits = descriptors.clone();
    empty_splits[0].descriptor.splits.clear();
    let error = validate_data_lifecycle_contracts(&empty_splits).unwrap_err();
    assert!(error.contains("protocol_descriptor_incomplete"));

    let mut wrong_operation = descriptors.clone();
    wrong_operation[0].data_lifecycle.prepare.id = "load";
    let error = validate_data_lifecycle_contracts(&wrong_operation).unwrap_err();
    assert!(error.contains("data_lifecycle_contract_mismatch"));

    let mut empty_contract = descriptors.clone();
    empty_contract[0].data_lifecycle.prepare.output_contract = "";
    let error = validate_data_lifecycle_contracts(&empty_contract).unwrap_err();
    assert!(error.contains("data_lifecycle_contract_incomplete"));
}

#[test]
fn adapt_protocol_004_runtime_lifecycle_and_failure_taxonomy_are_validated() {
    let descriptors = built_in_protocol_adapter_descriptors();
    validate_runtime_lifecycle_contracts(&descriptors).unwrap();
    let terminal = descriptors
        .iter()
        .find(|descriptor| descriptor.binding.benchmark_id.as_str() == "terminal-bench")
        .unwrap();
    assert!(terminal.runtime_lifecycle.cleanup.is_some());
    let swe = descriptors
        .iter()
        .find(|descriptor| descriptor.binding.benchmark_id.as_str() == "swe-bench-pro")
        .unwrap();
    assert!(
        swe.failure_mapping
            .iter()
            .any(|mapping| mapping.failure_code == FailureCode::NoValidDiff)
    );

    let mut missing_cleanup = descriptors.clone();
    missing_cleanup[0].runtime_lifecycle.cleanup = None;
    let error = validate_runtime_lifecycle_contracts(&missing_cleanup).unwrap_err();
    assert!(error.contains("cleanup.verdict_override without cleanup operation"));

    let mut missing_readiness = descriptors.clone();
    missing_readiness[1]
        .readiness
        .retain(|probe| probe.capability.as_str() != "readiness.basic");
    let error = validate_runtime_lifecycle_contracts(&missing_readiness).unwrap_err();
    assert!(error.contains("readiness_contract_missing"));

    let mut missing_docker_readiness = descriptors.clone();
    missing_docker_readiness[0]
        .readiness
        .retain(|probe| probe.capability.as_str() != "docker.orchestration");
    let error = validate_runtime_lifecycle_contracts(&missing_docker_readiness).unwrap_err();
    assert!(error.contains("docker.orchestration has no readiness probe"));

    for capability in [
        "official.runner",
        "host.agent_execution",
        "run_as.readiness",
        "cleanup.verdict_override",
    ] {
        let mut missing_probe = descriptors.clone();
        missing_probe[0]
            .readiness
            .retain(|probe| probe.capability.as_str() != capability);
        let error = validate_runtime_lifecycle_contracts(&missing_probe).unwrap_err();
        assert!(error.contains(&format!("{capability} has no readiness probe")));
    }

    let mut missing_patch_readiness = descriptors.clone();
    missing_patch_readiness[1]
        .readiness
        .retain(|probe| probe.capability.as_str() != "patch.evaluator");
    let error = validate_runtime_lifecycle_contracts(&missing_patch_readiness).unwrap_err();
    assert!(error.contains("patch.evaluator has no readiness probe"));

    let mut incomplete_readiness = descriptors.clone();
    incomplete_readiness[0].readiness[0].severity = "";
    let error = validate_runtime_lifecycle_contracts(&incomplete_readiness).unwrap_err();
    assert!(error.contains("readiness_contract_incomplete"));

    let mut wrong_runtime_operation = descriptors.clone();
    wrong_runtime_operation[0].runtime_lifecycle.execute.id = "run";
    let error = validate_runtime_lifecycle_contracts(&wrong_runtime_operation).unwrap_err();
    assert!(error.contains("runtime_lifecycle_contract_mismatch"));

    let mut duplicate_mapping = descriptors.clone();
    let duplicate = duplicate_mapping[1].failure_mapping[0].clone();
    duplicate_mapping[1].failure_mapping.push(duplicate);
    let error = validate_runtime_lifecycle_contracts(&duplicate_mapping).unwrap_err();
    assert!(error.contains("failure_mapping_duplicate"));

    let mut incomplete_mapping = descriptors.clone();
    incomplete_mapping[0].failure_mapping[0].adapter_phase = "";
    let error = validate_runtime_lifecycle_contracts(&incomplete_mapping).unwrap_err();
    assert!(error.contains("failure_mapping_incomplete"));

    let mut invalid_mapping = descriptors.clone();
    invalid_mapping[0].failure_mapping[0].failure_class = FailureClass::None;
    let error = validate_runtime_lifecycle_contracts(&invalid_mapping).unwrap_err();
    assert!(error.contains("failure_mapping_invalid"));
}

#[test]
fn adapt_protocol_005_artifact_boundary_and_redaction_contracts_are_validated() {
    let descriptors = built_in_protocol_adapter_descriptors();
    validate_artifact_contracts(&descriptors).unwrap();

    for descriptor in &descriptors {
        assert!(descriptor.artifacts.iter().any(|artifact| {
            artifact.artifact_type == "runtime_snapshot" && artifact.visibility == "public"
        }));
        assert!(descriptor.artifacts.iter().any(|artifact| {
            artifact.artifact_type == "runtime_snapshot" && artifact.visibility == "private"
        }));
        assert!(
            descriptor
                .report_metadata
                .public_artifacts
                .contains(&"external_runtime_public")
        );
        assert!(
            !descriptor
                .report_metadata
                .public_artifacts
                .contains(&"external_runtime_private"),
            "private runtime snapshots must not be report-public for {}",
            descriptor.binding.adapter_id
        );
    }

    let mut missing_artifacts = descriptors.clone();
    missing_artifacts[0].artifacts.clear();
    let error = validate_artifact_contracts(&missing_artifacts).unwrap_err();
    assert!(error.contains("artifact_contract_missing"));

    let mut duplicate_artifact = descriptors.clone();
    let duplicate = duplicate_artifact[0].artifacts[0].clone();
    duplicate_artifact[0].artifacts.push(duplicate);
    let error = validate_artifact_contracts(&duplicate_artifact).unwrap_err();
    assert!(error.contains("artifact_contract_duplicate"));

    let mut unsafe_path = descriptors.clone();
    unsafe_path[0].artifacts[0].path = "../external-runtime.public.json";
    let error = validate_artifact_contracts(&unsafe_path).unwrap_err();
    assert!(error.contains("artifact_contract_unsafe_path"));

    let mut backslash_path = descriptors.clone();
    backslash_path[1].artifacts[0].path = r"nested\\external-runtime.public.json";
    let error = validate_artifact_contracts(&backslash_path).unwrap_err();
    assert!(error.contains("artifact_contract_unsafe_path"));

    let mut duplicate_path = descriptors.clone();
    duplicate_path[0].artifacts[1].path = duplicate_path[0].artifacts[0].path;
    duplicate_path[0].artifacts[1].scope = duplicate_path[0].artifacts[0].scope;
    let error = validate_artifact_contracts(&duplicate_path).unwrap_err();
    assert!(error.contains("artifact_contract_duplicate_path"));

    let mut public_private_only = descriptors.clone();
    public_private_only[0].artifacts[0].redaction_policy = "private_only";
    let error = validate_artifact_contracts(&public_private_only).unwrap_err();
    assert!(error.contains("artifact_contract_public_private_only"));

    let mut private_without_private_only = descriptors.clone();
    let private_index = private_without_private_only[0]
        .artifacts
        .iter()
        .position(|artifact| artifact.visibility == "private")
        .unwrap();
    private_without_private_only[0].artifacts[private_index].redaction_policy = "structured";
    let error = validate_artifact_contracts(&private_without_private_only).unwrap_err();
    assert!(error.contains("artifact_contract_private_redaction_policy"));

    let mut missing_private_runtime = descriptors.clone();
    missing_private_runtime[0]
        .artifacts
        .retain(|artifact| artifact.visibility != "private");
    let error = validate_artifact_contracts(&missing_private_runtime).unwrap_err();
    assert!(error.contains("artifact_contract_runtime_snapshot_pair_missing"));

    let mut missing_capability_artifact = descriptors.clone();
    missing_capability_artifact[1]
        .artifacts
        .retain(|artifact| artifact.artifact_id != "prediction");
    let error = validate_artifact_contracts(&missing_capability_artifact).unwrap_err();
    assert!(error.contains("patch.evaluator requires artifact prediction"));

    let mut unknown_report_artifact = descriptors.clone();
    unknown_report_artifact[0]
        .report_metadata
        .public_artifacts
        .push("external_runtime_private");
    let error = validate_artifact_contracts(&unknown_report_artifact).unwrap_err();
    assert!(error.contains("report_metadata_undeclared_public_artifact"));

    let mut private_section_ref = descriptors.clone();
    private_section_ref[0].report_metadata.detail_sections[0]
        .public_artifact_refs
        .push("external_runtime_private");
    let error = validate_artifact_contracts(&private_section_ref).unwrap_err();
    assert!(error.contains("report_metadata_section_private_or_unknown_artifact"));

    let mut duplicate_report_artifact = descriptors.clone();
    duplicate_report_artifact[0]
        .report_metadata
        .public_artifacts
        .push("external_runtime_public");
    let error = validate_artifact_contracts(&duplicate_report_artifact).unwrap_err();
    assert!(error.contains("report_metadata_duplicate_public_artifact"));

    let mut duplicate_section = descriptors.clone();
    let duplicate = duplicate_section[1].report_metadata.detail_sections[0].clone();
    duplicate_section[1]
        .report_metadata
        .detail_sections
        .push(duplicate);
    let error = validate_artifact_contracts(&duplicate_section).unwrap_err();
    assert!(error.contains("report_metadata_duplicate_section"));
}

#[test]
fn adapt_protocol_009_scaffold_golden_adapter_compiles_and_passes_conformance() {
    let adapter = DeterministicSampleAdapter;
    let descriptor = adapter
        .protocol_descriptor()
        .expect("scaffold golden adapter must expose a protocol descriptor");
    assert_eq!(descriptor.descriptor.name, "deterministic-sample");
    assert_eq!(
        descriptor.binding.adapter_id.as_str(),
        "harnesslab.deterministic-sample.runtime"
    );
    assert_eq!(
        descriptor.binding.default_mode.as_str(),
        "deterministic-sample"
    );

    let descriptors = vec![descriptor.clone()];
    validate_data_lifecycle_contracts(&descriptors).unwrap();
    validate_runtime_lifecycle_contracts(&descriptors).unwrap();
    validate_artifact_contracts(&descriptors).unwrap();

    assert!(descriptor.runtime_lifecycle.cleanup.is_none());
    assert_eq!(descriptor.readiness.len(), 2);
    assert_eq!(descriptor.failure_mapping.len(), 1);
    assert_eq!(descriptor.artifacts.len(), 5);
    assert!(descriptor.artifacts.iter().any(|a| {
        a.artifact_type == "runtime_snapshot" && a.visibility == "public" && a.required_for_replay
    }));
    assert!(descriptor.artifacts.iter().any(|a| {
        a.artifact_type == "runtime_snapshot"
            && a.visibility == "private"
            && a.redaction_policy == "private_only"
    }));

    let mut missing_failure = descriptors.clone();
    missing_failure[0].failure_mapping.clear();
    let error = validate_runtime_lifecycle_contracts(&missing_failure).unwrap_err();
    assert!(error.contains("failure_mapping_missing"));

    let mut missing_readiness = descriptors.clone();
    missing_readiness[0]
        .readiness
        .retain(|probe| probe.capability.as_str() != "readiness.basic");
    let error = validate_runtime_lifecycle_contracts(&missing_readiness).unwrap_err();
    assert!(error.contains("readiness_contract_missing"));

    let mut missing_artifact = descriptors.clone();
    missing_artifact[0].artifacts.clear();
    let error = validate_artifact_contracts(&missing_artifact).unwrap_err();
    assert!(error.contains("artifact_contract_missing"));
}

#[test]
fn adapt_protocol_010_existing_adapters_have_protocol_descriptors_and_protocol_bindings() {
    let registry = crate::built_in_protocol_registry();
    let bindings = registry.bindings();
    assert!(
        bindings.len() >= 2,
        "registry must contain at least Terminal-Bench and SWE-bench Pro bindings"
    );

    let tb = TerminalBenchAdapter::new();
    let tb_descriptor = tb
        .protocol_descriptor()
        .expect("Terminal-Bench must expose a protocol descriptor after migration");
    validate_data_lifecycle_contracts(&[tb_descriptor.clone()]).unwrap();
    validate_runtime_lifecycle_contracts(&[tb_descriptor.clone()]).unwrap();
    validate_artifact_contracts(&[tb_descriptor.clone()]).unwrap();
    assert_eq!(
        tb_descriptor.binding.adapter_id.as_str(),
        "harnesslab.terminal-bench.runtime"
    );

    let swe = SweBenchProAdapter::new();
    let swe_descriptor = swe
        .protocol_descriptor()
        .expect("SWE-bench Pro must expose a protocol descriptor after migration");
    validate_data_lifecycle_contracts(&[swe_descriptor.clone()]).unwrap();
    validate_runtime_lifecycle_contracts(&[swe_descriptor.clone()]).unwrap();
    validate_artifact_contracts(&[swe_descriptor.clone()]).unwrap();
    assert_eq!(
        swe_descriptor.binding.adapter_id.as_str(),
        "harnesslab.swe-bench-pro.runtime"
    );
}

#[test]
fn adapt_protocol_011_third_adapter_horizontal_extension_requires_no_legacy_kind() {
    let registry = crate::built_in_protocol_registry();
    let bindings = registry.bindings();
    assert!(
        bindings.len() >= 3,
        "registry must contain at least three bindings for horizontal extension proof"
    );

    let third = bindings
        .iter()
        .find(|b| b.benchmark_id.as_str() == "deterministic-sample")
        .expect("deterministic-sample binding must exist");
    assert_eq!(
        third.stability,
        harnesslab_core::AdapterStability::Experimental,
        "third adapter must be experimental until it proves stable promotion"
    );

    let adapter = DeterministicSampleAdapter;
    let descriptor = adapter
        .protocol_descriptor()
        .expect("third adapter must expose a protocol descriptor");
    validate_data_lifecycle_contracts(&[descriptor.clone()]).unwrap();
    validate_runtime_lifecycle_contracts(&[descriptor.clone()]).unwrap();
    validate_artifact_contracts(&[descriptor.clone()]).unwrap();
}

#[test]
fn adapt_protocol_012_stable_promotion_evidence_matches_binding_stability() {
    use harnesslab_core::AdapterStability;

    let registry = crate::built_in_protocol_registry();
    let bindings = registry.bindings();

    let tb = bindings
        .iter()
        .find(|b| b.benchmark_id.as_str() == "terminal-bench")
        .expect("terminal-bench binding must exist");
    assert_eq!(
        tb.stability,
        AdapterStability::Stable,
        "Terminal-Bench is stable"
    );

    let swe = bindings
        .iter()
        .find(|b| b.benchmark_id.as_str() == "swe-bench-pro")
        .expect("swe-bench-pro binding must exist");
    assert_eq!(
        swe.stability,
        AdapterStability::ConditionalStableBlocked,
        "SWE-bench Pro is conditional-stable-blocked due to Docker-gated official proof"
    );

    let sample = bindings
        .iter()
        .find(|b| b.benchmark_id.as_str() == "deterministic-sample")
        .expect("deterministic-sample binding must exist");
    assert_eq!(
        sample.stability,
        AdapterStability::Experimental,
        "deterministic-sample is experimental"
    );
}
