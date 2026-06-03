pub(crate) mod materializer;
pub(crate) mod templates;

pub(crate) use materializer::{
    MaterializedAgentProfile, materialization_error_to_anyhow, materialize_profile,
    wrap_rendered_command,
};
pub(crate) use templates::{agents_readme, profile_template};
