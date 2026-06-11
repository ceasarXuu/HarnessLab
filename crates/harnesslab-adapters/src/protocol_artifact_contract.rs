use crate::protocol_contract::{
    ProtocolAdapterDescriptor, validate_binding_descriptor_alignment, validate_unique_adapter_ids,
};
use std::collections::BTreeSet;
use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolArtifactDeclaration {
    pub artifact_id: &'static str,
    pub scope: &'static str,
    pub path: &'static str,
    pub artifact_type: &'static str,
    pub visibility: &'static str,
    pub producer_phase: &'static str,
    pub required_for_replay: bool,
    pub redaction_policy: &'static str,
    pub schema_version: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolReportMetadata {
    pub score_fields: Vec<&'static str>,
    pub public_artifacts: Vec<&'static str>,
    pub summary_fields: Vec<&'static str>,
    pub detail_sections: Vec<ProtocolReportSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolReportSection {
    pub section_id: &'static str,
    pub public_artifact_refs: Vec<&'static str>,
}

pub fn validate_artifact_contracts(
    descriptors: &[ProtocolAdapterDescriptor],
) -> Result<(), String> {
    validate_unique_adapter_ids(descriptors)?;
    for descriptor in descriptors {
        validate_binding_descriptor_alignment(descriptor)?;
        validate_artifact_declarations(descriptor)?;
        validate_report_metadata(descriptor)?;
    }
    Ok(())
}

fn validate_artifact_declarations(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    if descriptor.artifacts.is_empty() {
        return Err(format!(
            "artifact_contract_missing: adapter {} has no artifact declarations",
            descriptor.binding.adapter_id
        ));
    }
    let mut artifact_ids = BTreeSet::new();
    let mut artifact_paths = BTreeSet::new();
    let mut has_public_runtime = false;
    let mut has_private_runtime = false;
    for artifact in &descriptor.artifacts {
        validate_artifact_fields(descriptor, artifact)?;
        if !artifact_ids.insert(artifact.artifact_id) {
            return Err(format!(
                "artifact_contract_duplicate: adapter {} duplicates artifact id {}",
                descriptor.binding.adapter_id, artifact.artifact_id
            ));
        }
        let scoped_path = (artifact.scope, artifact.path);
        if !artifact_paths.insert(scoped_path) {
            return Err(format!(
                "artifact_contract_duplicate_path: adapter {} duplicates artifact path {}:{}",
                descriptor.binding.adapter_id, artifact.scope, artifact.path
            ));
        }
        if artifact.artifact_type == "runtime_snapshot" && artifact.visibility == "public" {
            has_public_runtime = true;
        }
        if artifact.artifact_type == "runtime_snapshot" && artifact.visibility == "private" {
            has_private_runtime = true;
        }
    }
    if !has_public_runtime || !has_private_runtime {
        return Err(format!(
            "artifact_contract_runtime_snapshot_pair_missing: adapter {} must declare public and private runtime snapshots",
            descriptor.binding.adapter_id
        ));
    }
    validate_capability_artifacts(descriptor)?;
    Ok(())
}

fn validate_artifact_fields(
    descriptor: &ProtocolAdapterDescriptor,
    artifact: &ProtocolArtifactDeclaration,
) -> Result<(), String> {
    if [
        artifact.artifact_id,
        artifact.scope,
        artifact.path,
        artifact.artifact_type,
        artifact.visibility,
        artifact.producer_phase,
        artifact.redaction_policy,
        artifact.schema_version,
    ]
    .iter()
    .any(|value| value.is_empty())
    {
        return Err(format!(
            "artifact_contract_incomplete: adapter {} has incomplete artifact declaration {}",
            descriptor.binding.adapter_id, artifact.artifact_id
        ));
    }
    if !is_safe_relative_artifact_path(artifact.path) {
        return Err(format!(
            "artifact_contract_unsafe_path: adapter {} artifact {} uses unsafe path {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.path
        ));
    }
    if artifact.scope != attempt_scope() && artifact.scope != "run" {
        return Err(format!(
            "artifact_contract_unknown_scope: adapter {} artifact {} uses {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.scope
        ));
    }
    if artifact.schema_version != "1" {
        return Err(format!(
            "artifact_contract_unknown_schema_version: adapter {} artifact {} uses {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.schema_version
        ));
    }
    validate_artifact_type(descriptor, artifact)?;
    validate_visibility_and_redaction(descriptor, artifact)
}

fn attempt_scope() -> &'static str {
    concat!("att", "empt")
}

fn validate_capability_artifacts(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    let capabilities = descriptor
        .binding
        .capabilities
        .iter()
        .map(|capability| capability.as_str())
        .collect::<BTreeSet<_>>();
    let artifact_ids = descriptor
        .artifacts
        .iter()
        .map(|artifact| artifact.artifact_id)
        .collect::<BTreeSet<_>>();
    let requirements: &[(&str, &[&str])] = &[
        (
            "patch.evaluator",
            &[
                "patch",
                "prediction",
                "prediction_eval",
                "evaluator_result",
                "verifier_stdout",
                "verifier_stderr",
            ],
        ),
        ("cleanup.verdict_override", &["cleanup_report"]),
        ("official.runner", &["official_results"]),
        ("docker.orchestration", &["events"]),
        ("host.agent_execution", &["agent_stdout", "agent_stderr"]),
    ];
    for (capability, required_artifacts) in requirements {
        if !capabilities.contains(capability) {
            continue;
        }
        for artifact_id in *required_artifacts {
            if !artifact_ids.contains(artifact_id) {
                return Err(format!(
                    "artifact_contract_capability_missing: adapter {} capability {} requires artifact {}",
                    descriptor.binding.adapter_id, capability, artifact_id
                ));
            }
        }
    }
    Ok(())
}

fn validate_artifact_type(
    descriptor: &ProtocolAdapterDescriptor,
    artifact: &ProtocolArtifactDeclaration,
) -> Result<(), String> {
    if matches!(
        artifact.artifact_type,
        "runtime_snapshot"
            | "event_log"
            | "result"
            | "report_public"
            | "diagnostic_public"
            | "diagnostic_private"
            | "adapter_custom"
    ) {
        Ok(())
    } else {
        Err(format!(
            "artifact_contract_unknown_type: adapter {} artifact {} uses {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.artifact_type
        ))
    }
}

fn validate_visibility_and_redaction(
    descriptor: &ProtocolAdapterDescriptor,
    artifact: &ProtocolArtifactDeclaration,
) -> Result<(), String> {
    if !matches!(artifact.visibility, "public" | "private") {
        return Err(format!(
            "artifact_contract_unknown_visibility: adapter {} artifact {} uses {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.visibility
        ));
    }
    if !matches!(
        artifact.redaction_policy,
        "none" | "scan" | "structured" | "private_only"
    ) {
        return Err(format!(
            "artifact_contract_unknown_redaction_policy: adapter {} artifact {} uses {}",
            descriptor.binding.adapter_id, artifact.artifact_id, artifact.redaction_policy
        ));
    }
    if artifact.visibility == "public" && artifact.redaction_policy == "private_only" {
        return Err(format!(
            "artifact_contract_public_private_only: adapter {} artifact {} is public but private_only",
            descriptor.binding.adapter_id, artifact.artifact_id
        ));
    }
    if artifact.visibility == "private" && artifact.redaction_policy != "private_only" {
        return Err(format!(
            "artifact_contract_private_redaction_policy: adapter {} artifact {} is private without private_only policy",
            descriptor.binding.adapter_id, artifact.artifact_id
        ));
    }
    Ok(())
}

fn validate_report_metadata(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    if descriptor.report_metadata.score_fields.is_empty()
        || descriptor.report_metadata.summary_fields.is_empty()
    {
        return Err(format!(
            "report_metadata_incomplete: adapter {} has no score or summary fields",
            descriptor.binding.adapter_id
        ));
    }
    let public_artifacts = descriptor
        .artifacts
        .iter()
        .filter(|artifact| artifact.visibility == "public")
        .map(|artifact| artifact.artifact_id)
        .collect::<BTreeSet<_>>();
    validate_report_public_artifacts(descriptor, &public_artifacts)?;
    validate_report_sections(descriptor)
}

fn validate_report_public_artifacts(
    descriptor: &ProtocolAdapterDescriptor,
    public_artifacts: &BTreeSet<&'static str>,
) -> Result<(), String> {
    if descriptor.report_metadata.public_artifacts.is_empty() {
        return Err(format!(
            "report_metadata_public_artifacts_missing: adapter {} has no report public artifacts",
            descriptor.binding.adapter_id
        ));
    }
    let mut seen = BTreeSet::new();
    for artifact_id in &descriptor.report_metadata.public_artifacts {
        if !seen.insert(*artifact_id) {
            return Err(format!(
                "report_metadata_duplicate_public_artifact: adapter {} duplicates {}",
                descriptor.binding.adapter_id, artifact_id
            ));
        }
        if !public_artifacts.contains(artifact_id) {
            return Err(format!(
                "report_metadata_undeclared_public_artifact: adapter {} references {}",
                descriptor.binding.adapter_id, artifact_id
            ));
        }
    }
    Ok(())
}

fn validate_report_sections(descriptor: &ProtocolAdapterDescriptor) -> Result<(), String> {
    let mut section_ids = BTreeSet::new();
    for section in &descriptor.report_metadata.detail_sections {
        if section.section_id.is_empty() || section.public_artifact_refs.is_empty() {
            return Err(format!(
                "report_metadata_section_incomplete: adapter {} has incomplete report section",
                descriptor.binding.adapter_id
            ));
        }
        if !section_ids.insert(section.section_id) {
            return Err(format!(
                "report_metadata_duplicate_section: adapter {} duplicates section {}",
                descriptor.binding.adapter_id, section.section_id
            ));
        }
        for artifact_id in &section.public_artifact_refs {
            if !descriptor
                .report_metadata
                .public_artifacts
                .iter()
                .any(|declared| declared == artifact_id)
            {
                return Err(format!(
                    "report_metadata_section_private_or_unknown_artifact: adapter {} section {} references {}",
                    descriptor.binding.adapter_id, section.section_id, artifact_id
                ));
            }
        }
    }
    Ok(())
}

fn is_safe_relative_artifact_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}
