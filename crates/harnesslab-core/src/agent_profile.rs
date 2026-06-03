use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{AgentProfile, InputMode, is_valid_profile_name};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetupConfig {
    #[serde(default)]
    pub preset: SetupPreset,
    #[serde(default)]
    pub required_commands: Vec<String>,
    #[serde(default)]
    pub run_as: RunAs,
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SetupPreset {
    None,
    #[default]
    Builtin,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RunAs {
    Root,
    #[default]
    Harnesslab,
    Current,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    #[serde(default = "default_true")]
    pub inherit: bool,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileValidationReport {
    pub errors: Vec<ProfileValidationError>,
    pub warnings: Vec<ProfileValidationWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileValidationError {
    pub field: String,
    pub message: String,
    pub accepted_values: Vec<String>,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileValidationWarning {
    pub code: String,
    pub field: String,
    pub message: String,
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            preset: SetupPreset::Builtin,
            required_commands: Vec::new(),
            run_as: RunAs::Harnesslab,
            commands: Vec::new(),
        }
    }
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self {
            inherit: true,
            allow: Vec::new(),
            deny: Vec::new(),
            include_paths: Vec::new(),
        }
    }
}

impl ProfileValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl AgentProfile {
    pub fn validation_report(&self) -> ProfileValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        if self.schema_version != 1 {
            errors.push(ProfileValidationError {
                field: "schema_version".to_string(),
                message: format!("unsupported schema_version {}", self.schema_version),
                accepted_values: accepted_values(&["1"]),
                suggested_fix: "set schema_version = 1".to_string(),
            });
        }
        if !is_valid_profile_name(&self.name) {
            errors.push(ProfileValidationError {
                field: "name".to_string(),
                message: format!("invalid name {}", self.name),
                accepted_values: accepted_values(&["[a-zA-Z0-9][a-zA-Z0-9._-]*"]),
                suggested_fix: "rename the profile so it starts with an ASCII letter or digit"
                    .to_string(),
            });
        }
        match self.input_mode {
            InputMode::Argument if !self.command.contains("{{instruction}}") => {
                errors.push(ProfileValidationError {
                    field: "command".to_string(),
                    message: "argument input_mode requires {{instruction}}".to_string(),
                    accepted_values: accepted_values(&["command containing {{instruction}}"]),
                    suggested_fix: "add {{instruction}} to command or use input_mode = \"stdin\""
                        .to_string(),
                });
            }
            InputMode::File
                if !self.command.contains("{{instruction}}")
                    && !self.command.contains("{{instruction_file}}") =>
            {
                errors.push(ProfileValidationError {
                    field: "command".to_string(),
                    message: "file input_mode requires {{instruction_file}} or {{instruction}}"
                        .to_string(),
                    accepted_values: accepted_values(&["command containing {{instruction_file}}"]),
                    suggested_fix:
                        "add {{instruction_file}} to command or use input_mode = \"stdin\""
                            .to_string(),
                });
            }
            _ => {}
        }
        if self.timeout_sec == 0 {
            errors.push(ProfileValidationError {
                field: "timeout_sec".to_string(),
                message: "timeout_sec must be positive".to_string(),
                accepted_values: accepted_values(&["positive integer"]),
                suggested_fix: "set timeout_sec to a positive number of seconds".to_string(),
            });
        }
        validate_setup(&self.setup, &mut errors);
        validate_policy("skills", &self.skills, &mut errors);
        validate_policy("tools", &self.tools, &mut errors);
        validate_policy("hooks", &self.hooks, &mut errors);
        if self.auth.mount_docker_socket {
            warnings.push(ProfileValidationWarning {
                code: "docker_socket_requested".to_string(),
                field: "auth.mount_docker_socket".to_string(),
                message: "mount_docker_socket expands container privileges".to_string(),
            });
        }
        ProfileValidationReport { errors, warnings }
    }
}

pub fn validate_setup(setup: &SetupConfig, errors: &mut Vec<ProfileValidationError>) {
    if !matches!(setup.preset, SetupPreset::Custom) && !setup.commands.is_empty() {
        errors.push(ProfileValidationError {
            field: "setup.commands".to_string(),
            message: "setup.commands is only valid when setup.preset is custom".to_string(),
            accepted_values: accepted_values(&["custom"]),
            suggested_fix: "set setup.preset = \"custom\" or remove setup.commands".to_string(),
        });
    }
    for command in &setup.required_commands {
        if !is_valid_command_name(command) {
            errors.push(ProfileValidationError {
                field: "setup.required_commands".to_string(),
                message: format!("invalid command name {command:?}"),
                accepted_values: accepted_values(&["letters", "digits", ".", "_", "+", "-"]),
                suggested_fix: "use bare command names, not shell pipelines or paths".to_string(),
            });
        }
    }
}

pub fn validate_policy(
    field: &'static str,
    policy: &CapabilityPolicy,
    errors: &mut Vec<ProfileValidationError>,
) {
    for (index, value) in policy.allow.iter().enumerate() {
        if !is_valid_capability_name(value) {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "allow", index),
                message: format!("invalid capability name {value:?}"),
                accepted_values: accepted_values(&["non-empty name without path separators"]),
                suggested_fix: "remove empty names and use include_paths for filesystem paths"
                    .to_string(),
            });
        }
    }
    for (index, value) in policy.deny.iter().enumerate() {
        if !is_valid_capability_name(value) {
            errors.push(ProfileValidationError {
                field: indexed_field_path(field, "deny", index),
                message: format!("invalid capability name {value:?}"),
                accepted_values: accepted_values(&["non-empty name without path separators"]),
                suggested_fix: "remove empty names and use include_paths for filesystem paths"
                    .to_string(),
            });
        }
    }
    let allow = policy.allow.iter().collect::<BTreeSet<_>>();
    let deny = policy.deny.iter().collect::<BTreeSet<_>>();
    let duplicates = allow.intersection(&deny).collect::<Vec<_>>();
    if !duplicates.is_empty() {
        let duplicate = duplicates[0];
        let index = policy
            .allow
            .iter()
            .position(|value| value == *duplicate)
            .unwrap_or_default();
        errors.push(ProfileValidationError {
            field: indexed_field_path(field, "allow", index),
            message: format!("allow and deny overlap: {duplicates:?}"),
            accepted_values: accepted_values(&["disjoint allow and deny lists"]),
            suggested_fix: "remove duplicate entries from either allow or deny".to_string(),
        });
    }
}

pub fn policy_is_default(policy: &CapabilityPolicy) -> bool {
    policy.inherit
        && policy.allow.is_empty()
        && policy.deny.is_empty()
        && policy.include_paths.is_empty()
}

fn default_true() -> bool {
    true
}

pub fn is_valid_command_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-'))
}

fn is_valid_capability_name(value: &str) -> bool {
    !value.trim().is_empty() && !value.contains('/') && !value.contains('\\')
}

pub fn accepted_values(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

pub fn indexed_field_path(field: &str, list: &str, index: usize) -> String {
    format!("{field}.{list}[{index}]")
}
