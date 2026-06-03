use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{CapabilityPolicy, ProfileValidationError, indexed_field_path, policy_is_default};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDomain {
    Skills,
    Tools,
    Hooks,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum CapabilityEnforcement {
    Enforced,
    Unsupported { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityCatalog {
    pub domain: CapabilityDomain,
    pub available: Vec<String>,
    pub default_enabled: Vec<String>,
    pub enforcement: CapabilityEnforcement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedCapabilityPolicy {
    pub domain: CapabilityDomain,
    pub inherit: bool,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub include_paths: Vec<String>,
    pub available: Vec<String>,
    pub default_enabled: Vec<String>,
    pub candidate_effective: Vec<String>,
    pub effective: Vec<String>,
    pub enforcement: CapabilityEnforcement,
    pub errors: Vec<ProfileValidationError>,
}

impl CapabilityDomain {
    pub fn as_str(self) -> &'static str {
        match self {
            CapabilityDomain::Skills => "skills",
            CapabilityDomain::Tools => "tools",
            CapabilityDomain::Hooks => "hooks",
        }
    }
}

impl CapabilityCatalog {
    pub fn enforced(
        domain: CapabilityDomain,
        available: impl IntoIterator<Item = impl Into<String>>,
        default_enabled: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            domain,
            available: available.into_iter().map(Into::into).collect(),
            default_enabled: default_enabled.into_iter().map(Into::into).collect(),
            enforcement: CapabilityEnforcement::Enforced,
        }
    }

    pub fn unsupported(
        domain: CapabilityDomain,
        available: impl IntoIterator<Item = impl Into<String>>,
        default_enabled: impl IntoIterator<Item = impl Into<String>>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            domain,
            available: available.into_iter().map(Into::into).collect(),
            default_enabled: default_enabled.into_iter().map(Into::into).collect(),
            enforcement: CapabilityEnforcement::Unsupported {
                reason: reason.into(),
            },
        }
    }
}

impl ResolvedCapabilityPolicy {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match &self.enforcement {
            CapabilityEnforcement::Enforced => None,
            CapabilityEnforcement::Unsupported { reason } => Some(reason.as_str()),
        }
    }
}

pub fn resolve_capability_policy(
    policy: &CapabilityPolicy,
    catalog: &CapabilityCatalog,
) -> ResolvedCapabilityPolicy {
    let domain = catalog.domain;
    let field = domain.as_str();
    let available = catalog.available.iter().collect::<BTreeSet<_>>();
    let default_enabled = catalog.default_enabled.iter().collect::<BTreeSet<_>>();
    let deny = policy.deny.iter().collect::<BTreeSet<_>>();
    let mut errors = Vec::new();

    for (index, path) in policy.include_paths.iter().enumerate() {
        if domain != CapabilityDomain::Skills {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "include_paths", index),
                message: format!("{field}.include_paths is not supported"),
                accepted_values: vec!["skills.include_paths".to_string()],
                suggested_fix: format!(
                    "move {path:?} to skills.include_paths or remove it from {field}"
                ),
            });
        }
    }

    for (index, value) in policy.allow.iter().enumerate() {
        if !available.contains(value) {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "allow", index),
                message: format!("unknown {field} allow entry {value:?}"),
                accepted_values: catalog.available.clone(),
                suggested_fix: format!("use one of the known {field} capabilities"),
            });
        }
    }
    for (index, value) in policy.deny.iter().enumerate() {
        if !available.contains(value) {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "deny", index),
                message: format!("unknown {field} deny entry {value:?}"),
                accepted_values: catalog.available.clone(),
                suggested_fix: format!("use one of the known {field} capabilities"),
            });
        }
    }

    for (index, value) in policy.allow.iter().enumerate() {
        if deny.contains(value) {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "allow", index),
                message: format!("allow and deny overlap: {value:?}"),
                accepted_values: vec!["disjoint allow and deny lists".to_string()],
                suggested_fix: "remove duplicate entries from either allow or deny".to_string(),
            });
        }
    }

    if let CapabilityEnforcement::Unsupported { reason } = &catalog.enforcement {
        if !policy_is_default(policy) {
            errors.push(ProfileValidationError {
                field: first_non_default_field(field, policy),
                message: format!("non-default {field} policy is not materializable: {reason}"),
                accepted_values: vec!["default policy".to_string()],
                suggested_fix: format!(
                    "use the default {field} policy or select an agent kind with {field} enforcement"
                ),
            });
        }
    }

    let candidate_effective = if !policy.allow.is_empty() {
        unique(
            policy
                .allow
                .iter()
                .filter(|value| available.contains(*value) && !deny.contains(*value))
                .cloned()
                .collect(),
        )
    } else if policy.inherit {
        unique(
            catalog
                .default_enabled
                .iter()
                .filter(|value| default_enabled.contains(*value) && !deny.contains(*value))
                .cloned()
                .collect(),
        )
    } else {
        Vec::new()
    };
    let effective =
        if errors.is_empty() && matches!(catalog.enforcement, CapabilityEnforcement::Enforced) {
            candidate_effective.clone()
        } else {
            Vec::new()
        };

    ResolvedCapabilityPolicy {
        domain,
        inherit: policy.inherit,
        allow: policy.allow.clone(),
        deny: policy.deny.clone(),
        include_paths: policy.include_paths.clone(),
        available: catalog.available.clone(),
        default_enabled: catalog.default_enabled.clone(),
        candidate_effective,
        effective,
        enforcement: catalog.enforcement.clone(),
        errors,
    }
}

fn first_non_default_field(field: &str, policy: &CapabilityPolicy) -> String {
    if !policy.inherit {
        return format!("{field}.inherit");
    }
    if !policy.allow.is_empty() {
        return indexed_field_path(field, "allow", 0);
    }
    if !policy.deny.is_empty() {
        return indexed_field_path(field, "deny", 0);
    }
    if !policy.include_paths.is_empty() {
        return indexed_field_path(field, "include_paths", 0);
    }
    field.to_string()
}

fn unique(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agt_reg_008_resolver_allow_overrides_inherit_then_applies_deny() {
        let catalog = CapabilityCatalog::enforced(
            CapabilityDomain::Tools,
            ["bash", "read_file", "write_file"],
            ["bash", "read_file"],
        );
        let policy = CapabilityPolicy {
            inherit: true,
            allow: vec!["write_file".to_string(), "bash".to_string()],
            deny: vec!["bash".to_string()],
            include_paths: Vec::new(),
        };

        let resolved = resolve_capability_policy(&policy, &catalog);

        assert_eq!(resolved.candidate_effective, vec!["write_file"]);
        assert!(resolved.effective.is_empty());
        assert!(resolved.errors.iter().any(|error| {
            error.field == "tools.allow[1]" && error.message.contains("allow and deny overlap")
        }));
    }

    #[test]
    fn agt_reg_008_resolver_inherit_false_uses_explicit_allow_as_source() {
        let catalog = CapabilityCatalog::enforced(
            CapabilityDomain::Tools,
            ["bash", "read_file", "write_file"],
            ["bash", "read_file"],
        );
        let policy = CapabilityPolicy {
            inherit: false,
            allow: vec!["bash".to_string()],
            deny: Vec::new(),
            include_paths: Vec::new(),
        };

        let resolved = resolve_capability_policy(&policy, &catalog);

        assert!(resolved.errors.is_empty());
        assert_eq!(resolved.effective, vec!["bash"]);
    }

    #[test]
    fn agt_reg_008_resolver_deduplicates_allow_for_set_semantics() {
        let catalog = CapabilityCatalog::enforced(
            CapabilityDomain::Tools,
            ["bash", "read_file", "write_file"],
            ["bash", "read_file"],
        );
        let policy = CapabilityPolicy {
            inherit: false,
            allow: vec!["bash".to_string(), "bash".to_string()],
            deny: Vec::new(),
            include_paths: Vec::new(),
        };

        let resolved = resolve_capability_policy(&policy, &catalog);

        assert!(resolved.errors.is_empty());
        assert_eq!(resolved.candidate_effective, vec!["bash"]);
        assert_eq!(resolved.effective, vec!["bash"]);
    }

    #[test]
    fn agt_reg_008_resolver_reports_unknown_allow_and_unsupported_materializer() {
        let catalog = CapabilityCatalog::unsupported(
            CapabilityDomain::Tools,
            ["bash", "read_file"],
            ["bash"],
            "custom agents do not expose a verified tools materializer",
        );
        let policy = CapabilityPolicy {
            inherit: true,
            allow: vec!["missing".to_string()],
            deny: Vec::new(),
            include_paths: Vec::new(),
        };

        let resolved = resolve_capability_policy(&policy, &catalog);

        assert!(resolved.errors.iter().any(|error| {
            error.field == "tools.allow[0]" && error.message.contains("unknown tools")
        }));
        assert!(resolved.errors.iter().any(|error| {
            error.field == "tools.allow[0]" && error.message.contains("not materializable")
        }));
    }

    #[test]
    fn agt_reg_008_resolver_rejects_include_paths_outside_skills() {
        let catalog = CapabilityCatalog::enforced(
            CapabilityDomain::Hooks,
            ["pre_tool_use"],
            std::iter::empty::<&str>(),
        );
        let policy = CapabilityPolicy {
            inherit: true,
            allow: Vec::new(),
            deny: Vec::new(),
            include_paths: vec!["./hooks".to_string()],
        };

        let resolved = resolve_capability_policy(&policy, &catalog);

        assert_eq!(resolved.errors[0].field, "hooks.include_paths[0]");
    }
}
