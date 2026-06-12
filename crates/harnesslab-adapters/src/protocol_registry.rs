use harnesslab_core::{
    AdapterId, AdapterProtocolAuthority, AdapterProtocolVersion, AdapterStability, AdapterVersion,
    BenchmarkId, CapabilityId, SelectedMode,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterBindingDescriptor {
    pub benchmark_id: BenchmarkId,
    pub adapter_id: AdapterId,
    pub protocol_version: AdapterProtocolVersion,
    pub adapter_version: AdapterVersion,
    pub default_mode: SelectedMode,
    pub capabilities: Vec<CapabilityId>,
    pub stability: AdapterStability,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterRegistry {
    bindings: Vec<AdapterBindingDescriptor>,
}

impl AdapterRegistry {
    pub fn new(bindings: Vec<AdapterBindingDescriptor>) -> Result<Self, String> {
        validate_bindings(&bindings)?;
        Ok(Self { bindings })
    }

    pub fn bindings(&self) -> &[AdapterBindingDescriptor] {
        &self.bindings
    }

    pub fn default_binding(
        &self,
        benchmark_id: &BenchmarkId,
        mode: &SelectedMode,
    ) -> Option<&AdapterBindingDescriptor> {
        self.bindings.iter().find(|binding| {
            binding.enabled
                && &binding.benchmark_id == benchmark_id
                && &binding.default_mode == mode
        })
    }

    pub fn binding_for_adapter_id(
        &self,
        adapter_id: &AdapterId,
    ) -> Option<&AdapterBindingDescriptor> {
        self.bindings
            .iter()
            .find(|binding| binding.enabled && &binding.adapter_id == adapter_id)
    }

    pub fn validate_authority(
        &self,
        authority: &AdapterProtocolAuthority,
    ) -> Result<&AdapterBindingDescriptor, String> {
        let binding = self
            .binding_for_adapter_id(&authority.adapter_id)
            .ok_or_else(|| format!("unknown adapter_id {}", authority.adapter_id))?;
        if binding.benchmark_id != authority.benchmark_id {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} registered for benchmark {}, got {}",
                authority.adapter_id, binding.benchmark_id, authority.benchmark_id
            ));
        }
        if binding.default_mode != authority.selected_mode {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} registered for mode {}, got {}",
                authority.adapter_id, binding.default_mode, authority.selected_mode
            ));
        }
        if binding.protocol_version != authority.protocol_version {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} registered for protocol {}, got {}",
                authority.adapter_id, binding.protocol_version, authority.protocol_version
            ));
        }
        if binding.adapter_version != authority.adapter_version {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} registered for version {}, got {}",
                authority.adapter_id, binding.adapter_version, authority.adapter_version
            ));
        }
        if binding.stability != authority.stability {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} registered for stability {:?}, got {:?}",
                authority.adapter_id, binding.stability, authority.stability
            ));
        }
        if capability_set(&binding.capabilities) != capability_set(&authority.capabilities) {
            return Err(format!(
                "protocol_authority_mismatch: adapter {} capability set differs from registry",
                authority.adapter_id
            ));
        }
        let authority_binding = AdapterBindingDescriptor {
            benchmark_id: authority.benchmark_id.clone(),
            adapter_id: authority.adapter_id.clone(),
            protocol_version: authority.protocol_version.clone(),
            adapter_version: authority.adapter_version.clone(),
            default_mode: authority.selected_mode.clone(),
            capabilities: authority.capabilities.clone(),
            stability: authority.stability.clone(),
            enabled: true,
        };
        ensure_mode_capabilities(&authority_binding)?;
        Ok(binding)
    }
}

impl AdapterBindingDescriptor {
    pub fn authority(&self) -> AdapterProtocolAuthority {
        AdapterProtocolAuthority::new(
            self.benchmark_id.clone(),
            self.adapter_id.clone(),
            self.adapter_version.clone(),
            self.default_mode.clone(),
            self.capabilities.clone(),
            self.stability.clone(),
        )
    }
}

pub fn built_in_protocol_registry() -> AdapterRegistry {
    AdapterRegistry::new(vec![
        binding(
            "terminal-bench",
            "harnesslab.terminal-bench.runtime",
            "terminal-bench-runtime.v1",
            "official-runner",
            &[
                "descriptor",
                "data.lifecycle",
                "readiness.basic",
                "artifacts.basic",
                "failure.mapping",
                "replay.authority",
                "report.metadata",
                "official.runner",
                "docker.orchestration",
                "cleanup.verdict_override",
                "host.agent_execution",
                "run_as.readiness",
            ],
            AdapterStability::Stable,
        ),
        binding(
            "swe-bench-pro",
            "harnesslab.swe-bench-pro.runtime",
            "swe-bench-pro-runtime.v1",
            "patch-evaluator",
            &[
                "descriptor",
                "data.lifecycle",
                "readiness.basic",
                "artifacts.basic",
                "failure.mapping",
                "replay.authority",
                "report.metadata",
                "patch.evaluator",
                "host.agent_execution",
            ],
            AdapterStability::ConditionalStableBlocked,
        ),
        binding(
            "deterministic-sample",
            "harnesslab.deterministic-sample.runtime",
            "deterministic-sample-runtime.v1",
            "deterministic-sample",
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
    ])
    .expect("built-in adapter protocol registry must be valid")
}

fn binding(
    benchmark_id: &str,
    adapter_id: &str,
    adapter_version: &str,
    mode: &str,
    capabilities: &[&str],
    stability: AdapterStability,
) -> AdapterBindingDescriptor {
    AdapterBindingDescriptor {
        benchmark_id: BenchmarkId::new(benchmark_id).expect("valid built-in benchmark id"),
        adapter_id: AdapterId::new(adapter_id).expect("valid built-in adapter id"),
        protocol_version: AdapterProtocolVersion::new("1").expect("valid protocol version"),
        adapter_version: AdapterVersion::new(adapter_version).expect("valid adapter version"),
        default_mode: SelectedMode::new(mode).expect("valid selected mode"),
        capabilities: capabilities
            .iter()
            .map(|capability| CapabilityId::new(*capability).expect("valid capability id"))
            .collect(),
        stability,
        enabled: true,
    }
}

fn stable_promotion_evidence_exists(adapter_id: &AdapterId) -> bool {
    matches!(adapter_id.as_str(), "harnesslab.terminal-bench.runtime")
}

fn validate_bindings(bindings: &[AdapterBindingDescriptor]) -> Result<(), String> {
    let mut adapter_ids = BTreeSet::new();
    let mut defaults = BTreeMap::new();
    for binding in bindings {
        if !adapter_ids.insert(binding.adapter_id.clone()) {
            return Err(format!("duplicate adapter_id {}", binding.adapter_id));
        }
        let default_key = (binding.benchmark_id.clone(), binding.default_mode.clone());
        if binding.enabled
            && defaults
                .insert(default_key, binding.adapter_id.clone())
                .is_some()
        {
            return Err(format!(
                "duplicate default adapter for benchmark {} mode {}",
                binding.benchmark_id, binding.default_mode
            ));
        }
        if binding.protocol_version.as_str() != "1" {
            return Err(format!(
                "unsupported protocol version {} for adapter {}",
                binding.protocol_version, binding.adapter_id
            ));
        }
        if binding.stability == AdapterStability::Stable
            && !stable_promotion_evidence_exists(&binding.adapter_id)
        {
            return Err(format!(
                "stable_promotion_evidence_missing: adapter {} cannot be registered stable without evidence",
                binding.adapter_id
            ));
        }
        ensure_mode_capabilities(binding)?;
    }
    Ok(())
}

fn ensure_mode_capabilities(binding: &AdapterBindingDescriptor) -> Result<(), String> {
    let capabilities = capability_names(&binding.capabilities);
    for capability in CORE_CAPABILITIES {
        if !capabilities.contains(capability) {
            return Err(format!(
                "mode_capability_mismatch: adapter {} missing core capability {}",
                binding.adapter_id, capability
            ));
        }
    }
    let required = match binding.default_mode.as_str() {
        "deterministic-sample" => &[][..],
        "official-runner" => &["official.runner"][..],
        "patch-evaluator" => &["patch.evaluator", "host.agent_execution"][..],
        "cleanup-sensitive-runner" => &["cleanup.verdict_override"][..],
        "custom-report" => &["custom.report_panel"][..],
        mode => {
            return Err(format!(
                "unsupported selected mode {mode} for adapter {}",
                binding.adapter_id
            ));
        }
    };
    if binding.default_mode.as_str() == "deterministic-sample"
        && [
            "official.runner",
            "patch.evaluator",
            "cleanup.verdict_override",
            "custom.report_panel",
            "host.agent_execution",
            "docker.orchestration",
            "sandbox.runner",
        ]
        .iter()
        .any(|capability| capabilities.contains(capability))
    {
        return Err(format!(
            "mode_capability_mismatch: adapter {} mode {} has incompatible optional capability",
            binding.adapter_id, binding.default_mode
        ));
    }
    if matches!(
        binding.default_mode.as_str(),
        "official-runner" | "cleanup-sensitive-runner"
    ) && ![
        "host.agent_execution",
        "docker.orchestration",
        "sandbox.runner",
    ]
    .iter()
    .any(|capability| capabilities.contains(capability))
    {
        return Err(format!(
            "mode_capability_mismatch: adapter {} mode {} missing execution capability",
            binding.adapter_id, binding.default_mode
        ));
    }
    for capability in required {
        if !capabilities.contains(capability) {
            return Err(format!(
                "mode_capability_mismatch: adapter {} mode {} missing capability {}",
                binding.adapter_id, binding.default_mode, capability
            ));
        }
    }
    Ok(())
}

fn capability_set(capabilities: &[CapabilityId]) -> BTreeSet<CapabilityId> {
    capabilities.iter().cloned().collect()
}

fn capability_names(capabilities: &[CapabilityId]) -> BTreeSet<&str> {
    capabilities.iter().map(CapabilityId::as_str).collect()
}

const CORE_CAPABILITIES: &[&str] = &[
    "descriptor",
    "data.lifecycle",
    "readiness.basic",
    "artifacts.basic",
    "failure.mapping",
    "replay.authority",
    "report.metadata",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapt_protocol_002_registry_binding_resolution_and_conflicts_are_enforced() {
        let registry = built_in_protocol_registry();
        let terminal = registry
            .default_binding(
                &BenchmarkId::new("terminal-bench").unwrap(),
                &SelectedMode::new("official-runner").unwrap(),
            )
            .unwrap();

        assert_eq!(
            terminal.adapter_id.as_str(),
            "harnesslab.terminal-bench.runtime"
        );
        assert_eq!(registry.bindings().len(), 3);

        let mut duplicate_adapter = registry.bindings().to_vec();
        duplicate_adapter[1].adapter_id = duplicate_adapter[0].adapter_id.clone();
        let error = AdapterRegistry::new(duplicate_adapter).unwrap_err();
        assert!(error.contains("duplicate adapter_id"));

        let mut duplicate_default = registry.bindings().to_vec();
        duplicate_default[1].benchmark_id = duplicate_default[0].benchmark_id.clone();
        duplicate_default[1].default_mode = duplicate_default[0].default_mode.clone();
        let error = AdapterRegistry::new(duplicate_default).unwrap_err();
        assert!(error.contains("duplicate default adapter"));

        let mut missing_capability = registry.bindings().to_vec();
        missing_capability[0]
            .capabilities
            .retain(|capability| capability.as_str() != "official.runner");
        let error = AdapterRegistry::new(missing_capability).unwrap_err();
        assert!(error.contains("mode_capability_mismatch"));

        let mut missing_core = registry.bindings().to_vec();
        missing_core[0]
            .capabilities
            .retain(|capability| capability.as_str() != "descriptor");
        let error = AdapterRegistry::new(missing_core).unwrap_err();
        assert!(error.contains("missing core capability"));

        let mut docker_only_official = registry.bindings().to_vec();
        docker_only_official[0]
            .capabilities
            .retain(|capability| capability.as_str() != "host.agent_execution");
        assert!(AdapterRegistry::new(docker_only_official).is_ok());

        let mut no_execution_capability = registry.bindings().to_vec();
        no_execution_capability[0]
            .capabilities
            .retain(|capability| {
                ![
                    "host.agent_execution",
                    "docker.orchestration",
                    "sandbox.runner",
                ]
                .contains(&capability.as_str())
            });
        let error = AdapterRegistry::new(no_execution_capability).unwrap_err();
        assert!(error.contains("missing execution capability"));

        let mut stable_without_evidence = registry.bindings().to_vec();
        stable_without_evidence[0].stability = AdapterStability::Stable;
        stable_without_evidence[0].adapter_id =
            AdapterId::new("harnesslab.unknown.runtime").unwrap();
        let error = AdapterRegistry::new(stable_without_evidence).unwrap_err();
        assert!(error.contains("stable_promotion_evidence_missing"));

        let mut deterministic_with_official = registry.bindings().to_vec();
        deterministic_with_official[0].default_mode =
            SelectedMode::new("deterministic-sample").unwrap();
        let error = AdapterRegistry::new(deterministic_with_official).unwrap_err();
        assert!(error.contains("incompatible optional capability"));

        let mut deterministic_with_execution = registry.bindings().to_vec();
        deterministic_with_execution[1].default_mode =
            SelectedMode::new("deterministic-sample").unwrap();
        deterministic_with_execution[1]
            .capabilities
            .retain(|capability| capability.as_str() != "patch.evaluator");
        let error = AdapterRegistry::new(deterministic_with_execution).unwrap_err();
        assert!(error.contains("incompatible optional capability"));

        let mut mismatched_authority = registry.bindings()[0].authority();
        mismatched_authority.benchmark_id = BenchmarkId::new("swe-bench-pro").unwrap();
        let error = registry
            .validate_authority(&mismatched_authority)
            .unwrap_err();
        assert!(error.contains("protocol_authority_mismatch"));

        let mut mismatched_version = registry.bindings()[0].authority();
        mismatched_version.adapter_version =
            AdapterVersion::new("terminal-bench-runtime.v0").unwrap();
        let error = registry
            .validate_authority(&mismatched_version)
            .unwrap_err();
        assert!(error.contains("registered for version"));

        let mut mismatched_stability = registry.bindings()[0].authority();
        mismatched_stability.stability = AdapterStability::Legacy;
        let error = registry
            .validate_authority(&mismatched_stability)
            .unwrap_err();
        assert!(error.contains("registered for stability"));

        let mut mismatched_capabilities = registry.bindings()[0].authority();
        mismatched_capabilities
            .capabilities
            .retain(|capability| capability.as_str() != "host.agent_execution");
        let error = registry
            .validate_authority(&mismatched_capabilities)
            .unwrap_err();
        assert!(error.contains("capability set differs"));
    }
}
