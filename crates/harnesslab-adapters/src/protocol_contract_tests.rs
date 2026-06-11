use crate::{
    BenchmarkAdapter, SweBenchProAdapter, TerminalBenchAdapter,
    built_in_protocol_adapter_descriptors, validate_data_lifecycle_contracts,
    validate_runtime_lifecycle_contracts,
};
use harnesslab_core::{CapabilityId, FailureClass, FailureCode};

#[test]
fn adapt_protocol_003_data_lifecycle_contract_foundation_is_validated() {
    let descriptors = built_in_protocol_adapter_descriptors();
    validate_data_lifecycle_contracts(&descriptors).unwrap();
    assert_eq!(descriptors.len(), 2);
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
