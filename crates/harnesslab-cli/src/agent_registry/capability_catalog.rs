use harnesslab_core::{
    AgentKind, CapabilityCatalog, CapabilityDomain, CapabilityEnforcement,
    ResolvedCapabilityPolicy, resolve_capability_policy,
};

use harnesslab_core::{AgentProfile, CapabilityPolicy};

pub(crate) fn resolve_profile_capabilities(profile: &AgentProfile) -> MaterializedCapabilities {
    MaterializedCapabilities {
        skills: resolve_for(profile.kind, CapabilityDomain::Skills, &profile.skills),
        tools: resolve_for(profile.kind, CapabilityDomain::Tools, &profile.tools),
        hooks: resolve_for(profile.kind, CapabilityDomain::Hooks, &profile.hooks),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct MaterializedCapabilities {
    pub skills: ResolvedCapabilityPolicy,
    pub tools: ResolvedCapabilityPolicy,
    pub hooks: ResolvedCapabilityPolicy,
}

impl MaterializedCapabilities {
    pub(crate) fn errors(&self) -> Vec<harnesslab_core::ProfileValidationError> {
        self.skills
            .errors
            .iter()
            .chain(self.tools.errors.iter())
            .chain(self.hooks.errors.iter())
            .cloned()
            .collect()
    }

    pub(crate) fn unsupported_reasons(&self) -> Vec<String> {
        [&self.skills, &self.tools, &self.hooks]
            .into_iter()
            .filter_map(|policy| match &policy.enforcement {
                CapabilityEnforcement::Enforced => None,
                CapabilityEnforcement::Unsupported { reason } => Some(reason.clone()),
            })
            .collect()
    }
}

fn resolve_for(
    kind: AgentKind,
    domain: CapabilityDomain,
    policy: &CapabilityPolicy,
) -> ResolvedCapabilityPolicy {
    let catalog = capability_catalog(kind, domain);
    resolve_capability_policy(policy, &catalog)
}

fn capability_catalog(kind: AgentKind, domain: CapabilityDomain) -> CapabilityCatalog {
    unsupported_catalog(kind, domain)
}

fn unsupported_catalog(kind: AgentKind, domain: CapabilityDomain) -> CapabilityCatalog {
    let reason = format!(
        "kind {:?} has no verified {} materializer",
        kind,
        domain.as_str()
    );
    match domain {
        CapabilityDomain::Skills => CapabilityCatalog::unsupported(
            domain,
            ["code-review", "clear-prd", "test-runner"],
            std::iter::empty::<&str>(),
            reason,
        ),
        CapabilityDomain::Tools => CapabilityCatalog::unsupported(
            domain,
            ["bash", "read_file", "write_file", "web_search"],
            ["bash", "read_file", "write_file"],
            reason,
        ),
        CapabilityDomain::Hooks => CapabilityCatalog::unsupported(
            domain,
            ["pre_tool_use", "post_tool_use"],
            std::iter::empty::<&str>(),
            reason,
        ),
    }
}
