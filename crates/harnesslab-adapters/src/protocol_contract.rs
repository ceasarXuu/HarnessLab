use crate::{
    AdapterBindingDescriptor, ProtocolArtifactDeclaration, ProtocolReportMetadata,
    built_in_protocol_registry,
};
use harnesslab_core::{
    AdapterId, BenchmarkDescriptor, CapabilityId, FailureClass, FailureCode, HealthImpact,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolAdapterDescriptor {
    pub binding: AdapterBindingDescriptor,
    pub descriptor: BenchmarkDescriptor,
    pub data_lifecycle: ProtocolDataLifecycleContract,
    pub runtime_lifecycle: ProtocolRuntimeLifecycleContract,
    pub artifacts: Vec<ProtocolArtifactDeclaration>,
    pub report_metadata: ProtocolReportMetadata,
    pub readiness: Vec<ProtocolReadinessProbe>,
    pub failure_mapping: Vec<ProtocolFailureMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolDataLifecycleContract {
    pub inspect_data: ProtocolOperation,
    pub prepare: ProtocolOperation,
    pub list_tasks: ProtocolOperation,
    pub create_task_plan: ProtocolOperation,
    pub snapshot_task: ProtocolOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolRuntimeLifecycleContract {
    pub preflight: ProtocolOperation,
    pub execute: ProtocolOperation,
    pub cleanup: Option<ProtocolOperation>,
    pub snapshot: ProtocolOperation,
    pub replay_validate: ProtocolOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOperation {
    pub id: &'static str,
    pub capability: CapabilityId,
    pub input_contract: &'static str,
    pub output_contract: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolReadinessProbe {
    pub id: &'static str,
    pub capability: CapabilityId,
    pub phase: &'static str,
    pub severity: &'static str,
    pub status_contract: &'static str,
    pub public_message: &'static str,
    pub remediation: &'static str,
    pub private_detail_contract: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolFailureMapping {
    pub adapter_code: &'static str,
    pub adapter_phase: &'static str,
    pub adapter_subphase: &'static str,
    pub failure_class: FailureClass,
    pub failure_code: FailureCode,
    pub health_impact: HealthImpact,
    pub public_message: &'static str,
    pub private_diagnostics_contract: &'static str,
}

pub fn validate_data_lifecycle_contracts(
    descriptors: &[ProtocolAdapterDescriptor],
) -> Result<(), String> {
    validate_unique_adapter_ids(descriptors)?;
    for descriptor in descriptors {
        validate_binding_descriptor_alignment(descriptor)?;
        validate_data_operation(
            descriptor,
            &descriptor.data_lifecycle.inspect_data,
            "inspect_data",
        )?;
        validate_data_operation(descriptor, &descriptor.data_lifecycle.prepare, "prepare")?;
        validate_data_operation(
            descriptor,
            &descriptor.data_lifecycle.list_tasks,
            "list_tasks",
        )?;
        validate_data_operation(
            descriptor,
            &descriptor.data_lifecycle.create_task_plan,
            "create_task_plan",
        )?;
        validate_data_operation(
            descriptor,
            &descriptor.data_lifecycle.snapshot_task,
            "snapshot_task",
        )?;
    }
    Ok(())
}

pub fn validate_runtime_lifecycle_contracts(
    descriptors: &[ProtocolAdapterDescriptor],
) -> Result<(), String> {
    validate_unique_adapter_ids(descriptors)?;
    for descriptor in descriptors {
        validate_binding_descriptor_alignment(descriptor)?;
        validate_runtime_operation(
            descriptor,
            &descriptor.runtime_lifecycle.preflight,
            "runtime_preflight",
        )?;
        validate_runtime_operation(descriptor, &descriptor.runtime_lifecycle.execute, "execute")?;
        validate_runtime_operation(
            descriptor,
            &descriptor.runtime_lifecycle.snapshot,
            "runtime_snapshot",
        )?;
        validate_runtime_operation(
            descriptor,
            &descriptor.runtime_lifecycle.replay_validate,
            "replay_validate",
        )?;
        if descriptor
            .binding
            .capabilities
            .iter()
            .any(|capability| capability.as_str() == "cleanup.verdict_override")
            && descriptor.runtime_lifecycle.cleanup.is_none()
        {
            return Err(format!(
                "runtime_lifecycle_contract_missing: adapter {} declares cleanup.verdict_override without cleanup operation",
                descriptor.binding.adapter_id
            ));
        }
        validate_readiness_probes(descriptor)?;
        validate_failure_mappings(descriptor)?;
    }
    Ok(())
}

pub(crate) fn validate_unique_adapter_ids(
    descriptors: &[ProtocolAdapterDescriptor],
) -> Result<(), String> {
    let mut adapter_ids = BTreeSet::<AdapterId>::new();
    for descriptor in descriptors {
        if !adapter_ids.insert(descriptor.binding.adapter_id.clone()) {
            return Err(format!(
                "duplicate protocol adapter descriptor {}",
                descriptor.binding.adapter_id
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_binding_descriptor_alignment(
    descriptor: &ProtocolAdapterDescriptor,
) -> Result<(), String> {
    if descriptor.descriptor.name != descriptor.binding.benchmark_id.as_str() {
        return Err(format!(
            "protocol_descriptor_mismatch: adapter {} benchmark descriptor {} does not match binding {}",
            descriptor.binding.adapter_id,
            descriptor.descriptor.name,
            descriptor.binding.benchmark_id
        ));
    }
    if descriptor.descriptor.splits.is_empty() {
        return Err(format!(
            "protocol_descriptor_incomplete: adapter {} has no benchmark splits",
            descriptor.binding.adapter_id
        ));
    }
    built_in_protocol_registry()
        .validate_authority(&descriptor.binding.authority())
        .map_err(|error| format!("protocol_binding_invalid: {error}"))?;
    Ok(())
}

fn validate_data_operation(
    descriptor: &ProtocolAdapterDescriptor,
    operation: &ProtocolOperation,
    expected_id: &str,
) -> Result<(), String> {
    if operation.id != expected_id {
        return Err(format!(
            "data_lifecycle_contract_mismatch: adapter {} expected operation {}, got {}",
            descriptor.binding.adapter_id, expected_id, operation.id
        ));
    }
    require_capability(descriptor, &operation.capability, "data_lifecycle")?;
    require_contract_shape(descriptor, operation, "data_lifecycle")
}

fn validate_runtime_operation(
    descriptor: &ProtocolAdapterDescriptor,
    operation: &ProtocolOperation,
    expected_id: &str,
) -> Result<(), String> {
    if operation.id != expected_id {
        return Err(format!(
            "runtime_lifecycle_contract_mismatch: adapter {} expected operation {}, got {}",
            descriptor.binding.adapter_id, expected_id, operation.id
        ));
    }
    require_capability(descriptor, &operation.capability, "runtime_lifecycle")?;
    require_contract_shape(descriptor, operation, "runtime_lifecycle")
}

fn validate_readiness_probes(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    let probes = descriptor
        .readiness
        .iter()
        .map(|probe| probe.capability.as_str())
        .collect::<BTreeSet<_>>();
    for capability in required_readiness_capabilities(&descriptor.binding.capabilities) {
        if !probes.contains(capability) {
            return Err(format!(
                "readiness_contract_missing: adapter {} capability {} has no readiness probe",
                descriptor.binding.adapter_id, capability
            ));
        }
    }
    for probe in &descriptor.readiness {
        require_capability(descriptor, &probe.capability, "readiness")?;
        if [
            probe.id,
            probe.phase,
            probe.severity,
            probe.status_contract,
            probe.public_message,
            probe.remediation,
            probe.private_detail_contract,
        ]
        .iter()
        .any(|value| value.is_empty())
        {
            return Err(format!(
                "readiness_contract_incomplete: adapter {} has incomplete probe {}",
                descriptor.binding.adapter_id, probe.id
            ));
        }
    }
    Ok(())
}

fn validate_failure_mappings(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    if descriptor.failure_mapping.is_empty() {
        return Err(format!(
            "failure_mapping_missing: adapter {} has no central failure mappings",
            descriptor.binding.adapter_id
        ));
    }
    let mut adapter_codes = BTreeSet::new();
    for mapping in &descriptor.failure_mapping {
        if [
            mapping.adapter_code,
            mapping.adapter_phase,
            mapping.adapter_subphase,
            mapping.public_message,
            mapping.private_diagnostics_contract,
        ]
        .iter()
        .any(|value| value.is_empty())
        {
            return Err(format!(
                "failure_mapping_incomplete: adapter {} has incomplete mapping",
                descriptor.binding.adapter_id
            ));
        }
        if !adapter_codes.insert(mapping.adapter_code) {
            return Err(format!(
                "failure_mapping_duplicate: adapter {} duplicates adapter code {}",
                descriptor.binding.adapter_id, mapping.adapter_code
            ));
        }
        if mapping.failure_class == FailureClass::None {
            return Err(format!(
                "failure_mapping_invalid: adapter {} maps {} to non-failure class",
                descriptor.binding.adapter_id, mapping.adapter_code
            ));
        }
    }
    Ok(())
}

fn required_readiness_capabilities(capabilities: &[CapabilityId]) -> BTreeSet<&str> {
    capability_names(capabilities)
        .into_iter()
        .filter(|capability| {
            matches!(
                *capability,
                "readiness.basic"
                    | "docker.orchestration"
                    | "official.runner"
                    | "patch.evaluator"
                    | "host.agent_execution"
                    | "run_as.readiness"
                    | "cleanup.verdict_override"
            ) || capability.ends_with(".readiness")
        })
        .collect()
}

fn require_capability(
    descriptor: &ProtocolAdapterDescriptor,
    capability: &CapabilityId,
    contract: &str,
) -> Result<(), String> {
    if !descriptor
        .binding
        .capabilities
        .iter()
        .any(|declared| declared == capability)
    {
        return Err(format!(
            "{contract}_capability_missing: adapter {} operation references undeclared capability {}",
            descriptor.binding.adapter_id, capability
        ));
    }
    Ok(())
}

fn require_contract_shape(
    descriptor: &ProtocolAdapterDescriptor,
    operation: &ProtocolOperation,
    contract: &str,
) -> Result<(), String> {
    if operation.input_contract.is_empty() || operation.output_contract.is_empty() {
        return Err(format!(
            "{contract}_contract_incomplete: adapter {} operation {} has empty input/output contract",
            descriptor.binding.adapter_id, operation.id
        ));
    }
    Ok(())
}

fn capability_names(capabilities: &[CapabilityId]) -> BTreeSet<&str> {
    capabilities.iter().map(CapabilityId::as_str).collect()
}
