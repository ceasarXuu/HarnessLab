use crate::agent_registry::resolve_profile_capabilities;
use crate::output::DoctorCheck;
use harnesslab_core::{AgentProfile, CapabilityEnforcement, ResolvedCapabilityPolicy};

pub(crate) fn append_capability_policy_checks(
    profile: &AgentProfile,
    checks: &mut Vec<DoctorCheck>,
) {
    let capabilities = resolve_profile_capabilities(profile);
    checks.push(policy_check(profile, "skills", &capabilities.skills));
    checks.push(policy_check(profile, "tools", &capabilities.tools));
    checks.push(policy_check(profile, "hooks", &capabilities.hooks));
}

fn policy_check(
    profile: &AgentProfile,
    domain: &str,
    policy: &ResolvedCapabilityPolicy,
) -> DoctorCheck {
    let errors = &policy.errors;
    let unsupported_reason = policy.unsupported_reason();
    let materializer_verified = matches!(policy.enforcement, CapabilityEnforcement::Enforced);
    let status = if !errors.is_empty() {
        "error"
    } else if materializer_verified {
        "ok"
    } else {
        "warning"
    };
    let message = if !errors.is_empty() {
        format!("Agent {domain} policy has blocking errors")
    } else if materializer_verified {
        format!("Agent {domain} policy resolved")
    } else {
        format!("Agent {domain} policy materializer is not verified")
    };
    let severity = if status == "warning" {
        "warning"
    } else {
        "error"
    };
    let first_error = errors.first();
    DoctorCheck {
        id: format!("agent.{}.{}.policy", profile.name, domain),
        status: status.to_string(),
        severity: severity.to_string(),
        message,
        details: serde_json::json!({
            "domain": domain,
            "inherit": policy.inherit,
            "allow": policy.allow,
            "deny": policy.deny,
            "include_paths": policy.include_paths,
            "available": policy.available,
            "default_enabled": policy.default_enabled,
            "candidate_effective": policy.candidate_effective,
            "effective": policy.effective,
            "enforcement": policy.enforcement,
            "materializer_verified": materializer_verified,
            "unsupported_reason": unsupported_reason,
            "field_path": first_error.map(|error| error.field.as_str()),
            "suggested_fix": first_error.map(|error| error.suggested_fix.as_str()),
            "errors": errors,
        }),
    }
}
