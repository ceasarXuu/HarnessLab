pub(crate) mod capability_catalog;
pub(crate) mod materializer;
pub(crate) mod templates;
pub(crate) mod version_probe;

pub(crate) use capability_catalog::resolve_profile_capabilities;
pub(crate) use materializer::{
    MaterializedAgentProfile, materialization_error_to_anyhow, materialize_profile,
    run_as_requires_sandbox, wrap_rendered_command,
};
pub(crate) use templates::{agents_readme, profile_template};
pub(crate) use version_probe::{
    AgentVersionSnapshot, VersionProbeStatus, probe_agent_version, sanitize_probe_text,
};
