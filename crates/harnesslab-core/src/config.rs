use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::{CapabilityPolicy, RunAs, SetupConfig, SetupPreset};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub schema_version: u32,
    pub default_concurrency: usize,
    pub default_attempts: u32,
    pub runs_dir: String,
    pub benchmarks_dir: String,
    pub network_default: crate::NetworkPolicy,
    pub usage_default: UsageConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageConfig {
    pub parser: String,
    #[serde(default = "default_usage_source")]
    pub source: String,
    #[serde(default = "default_input_tokens_key")]
    pub input_tokens_key: String,
    #[serde(default = "default_output_tokens_key")]
    pub output_tokens_key: String,
    #[serde(default = "default_total_tokens_key")]
    pub total_tokens_key: String,
    #[serde(default = "default_cost_usd_key")]
    pub cost_usd_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub schema_version: u32,
    pub name: String,
    pub kind: AgentKind,
    pub display_name: String,
    pub command: String,
    pub input_mode: InputMode,
    pub working_dir: WorkingDirMode,
    pub timeout_sec: u64,
    pub version_command: Option<String>,
    pub auth: AuthConfig,
    #[serde(default)]
    pub setup: SetupConfig,
    #[serde(default)]
    pub skills: CapabilityPolicy,
    #[serde(default)]
    pub tools: CapabilityPolicy,
    #[serde(default)]
    pub hooks: CapabilityPolicy,
    pub usage: UsageConfig,
    #[serde(default)]
    pub labels: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    Codex,
    ClaudeCode,
    Opencode,
    PiCodingAgent,
    Fake,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    Argument,
    File,
    Stdin,
    Tty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkingDirMode {
    Workspace,
    RunDir,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub inherit: bool,
    pub inherit_env: Vec<String>,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub mount_ssh_socket: bool,
    pub mount_docker_socket: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthMountSpec {
    pub host: String,
    pub mount: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("unsupported schema_version {0}")]
    UnsupportedSchema(u32),
    #[error("invalid name {0}")]
    InvalidName(String),
    #[error("command must include an input variable for argument/file mode")]
    MissingInputVariable,
    #[error("timeout_sec must be positive")]
    InvalidTimeout,
    #[error("execution defaults must be positive")]
    InvalidDefaults,
    #[error("invalid {field}: {message}; accepted={accepted}")]
    InvalidField {
        field: &'static str,
        message: String,
        accepted: &'static str,
    },
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            default_concurrency: 4,
            default_attempts: 1,
            runs_dir: "~/.harnesslab/runs".to_string(),
            benchmarks_dir: "~/.harnesslab/benchmarks".to_string(),
            network_default: crate::NetworkPolicy::Full,
            usage_default: UsageConfig::default(),
        }
    }
}

impl Default for UsageConfig {
    fn default() -> Self {
        Self {
            parser: "none".to_string(),
            source: default_usage_source(),
            input_tokens_key: default_input_tokens_key(),
            output_tokens_key: default_output_tokens_key(),
            total_tokens_key: default_total_tokens_key(),
            cost_usd_key: default_cost_usd_key(),
        }
    }
}

impl AgentProfile {
    pub fn validate(&self) -> Result<Vec<ValidationWarning>, ConfigError> {
        if self.schema_version != 1 {
            return Err(ConfigError::UnsupportedSchema(self.schema_version));
        }
        if !is_valid_profile_name(&self.name) {
            return Err(ConfigError::InvalidName(self.name.clone()));
        }
        match self.input_mode {
            InputMode::Argument if !self.command.contains("{{instruction}}") => {
                return Err(ConfigError::MissingInputVariable);
            }
            InputMode::File
                if !self.command.contains("{{instruction}}")
                    && !self.command.contains("{{instruction_file}}") =>
            {
                return Err(ConfigError::MissingInputVariable);
            }
            _ => {}
        }
        if self.timeout_sec == 0 {
            return Err(ConfigError::InvalidTimeout);
        }
        let report = self.validation_report();
        if let Some(error) = report.errors.first() {
            return Err(ConfigError::InvalidField {
                field: error.field,
                message: error.message.clone(),
                accepted: "see doctor --json details",
            });
        }
        let warnings = report
            .warnings
            .into_iter()
            .map(|warning| ValidationWarning {
                code: warning.code,
                message: warning.message,
            })
            .collect::<Vec<_>>();
        Ok(warnings)
    }
}

pub fn validate_global_config(config: &GlobalConfig) -> Result<(), ConfigError> {
    if config.schema_version != 1 {
        return Err(ConfigError::UnsupportedSchema(config.schema_version));
    }
    if config.default_attempts == 0 || config.default_concurrency == 0 {
        return Err(ConfigError::InvalidDefaults);
    }
    Ok(())
}

pub fn expand_path(input: &str, home: &Path, base: &Path) -> PathBuf {
    if input == "~" {
        return home.to_path_buf();
    }
    if let Some(stripped) = input.strip_prefix("~/") {
        return home.join(stripped);
    }
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

pub fn parse_auth_mount(value: &str) -> Option<AuthMountSpec> {
    let parts = value.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [host] => {
            let expanded = normalized_auth_host_path(host);
            Some(AuthMountSpec {
                host: expanded.clone(),
                mount: format!("{expanded}:{expanded}:ro"),
            })
        }
        [host, container] => {
            let expanded = normalized_auth_host_path(host);
            Some(AuthMountSpec {
                host: expanded.clone(),
                mount: format!("{expanded}:{container}:ro"),
            })
        }
        [host, container, mode] => {
            let expanded = normalized_auth_host_path(host);
            Some(AuthMountSpec {
                host: expanded.clone(),
                mount: format!("{expanded}:{container}:{mode}"),
            })
        }
        _ => None,
    }
}

pub fn effective_auth_mount_specs(profile: &AgentProfile) -> Vec<AuthMountSpec> {
    if !profile.auth.inherit {
        return Vec::new();
    }
    let excluded_hosts = profile
        .auth
        .exclude_paths
        .iter()
        .map(|path| normalized_auth_host_path(path))
        .collect::<Vec<_>>();
    let mut specs = Vec::new();
    for path in &profile.auth.include_paths {
        if let Some(spec) = parse_auth_mount(path)
            && !excluded_hosts.contains(&spec.host)
            && !specs
                .iter()
                .any(|existing: &AuthMountSpec| existing.mount == spec.mount)
        {
            specs.push(spec);
        }
    }
    specs
}

pub fn normalized_auth_host_path(value: &str) -> String {
    let host = value.split(':').next().unwrap_or(value);
    let home = std::env::var("HOME").unwrap_or_default();
    if host == "~" {
        home
    } else if let Some(rest) = host.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        host.to_string()
    }
}

pub fn redacted_profile_snapshot(profile: &AgentProfile, secrets: &[&str]) -> AgentProfile {
    let mut snapshot = profile.clone();
    snapshot.command = crate::redact_known_secret(&snapshot.command, secrets);
    snapshot
}

pub fn default_agent_profile(name: &str, kind: AgentKind, command: &str) -> AgentProfile {
    AgentProfile {
        schema_version: 1,
        name: name.to_string(),
        kind,
        display_name: name.to_string(),
        command: command.to_string(),
        input_mode: InputMode::Stdin,
        working_dir: WorkingDirMode::Workspace,
        timeout_sec: 3600,
        version_command: default_version_command(kind),
        auth: default_auth_config(kind),
        setup: default_setup_config(kind),
        skills: CapabilityPolicy::default(),
        tools: CapabilityPolicy::default(),
        hooks: CapabilityPolicy::default(),
        usage: UsageConfig::default(),
        labels: default_labels(kind),
    }
}

fn default_setup_config(kind: AgentKind) -> SetupConfig {
    let required_commands = match kind {
        AgentKind::Codex => vec!["codex"],
        AgentKind::ClaudeCode => vec!["claude"],
        AgentKind::Opencode => vec!["opencode"],
        AgentKind::PiCodingAgent => vec!["pi"],
        AgentKind::Fake | AgentKind::Custom => Vec::new(),
    }
    .into_iter()
    .map(str::to_string)
    .collect();
    SetupConfig {
        preset: match kind {
            AgentKind::Fake => SetupPreset::None,
            _ => SetupPreset::Builtin,
        },
        required_commands,
        run_as: match kind {
            AgentKind::Fake => RunAs::Current,
            _ => RunAs::Harnesslab,
        },
        commands: Vec::new(),
    }
}

fn default_labels(kind: AgentKind) -> std::collections::BTreeMap<String, String> {
    let mut labels = std::collections::BTreeMap::new();
    match kind {
        AgentKind::Codex => {
            labels.insert(
                "sandbox_setup_command".to_string(),
                "if ! command -v codex >/dev/null 2>&1; then if command -v npm >/dev/null 2>&1; then npm install -g @openai/codex >/tmp/harnesslab-codex-install.log 2>&1 || { cat /tmp/harnesslab-codex-install.log >&2; exit 127; }; else echo 'codex CLI missing and npm unavailable' >&2; exit 127; fi; fi".to_string(),
            );
        }
        AgentKind::ClaudeCode => {
            labels.insert(
                "sandbox_setup_command".to_string(),
                "if ! command -v claude >/dev/null 2>&1; then if command -v npm >/dev/null 2>&1; then npm install -g @anthropic-ai/claude-code >/tmp/harnesslab-claude-code-install.log 2>&1 || { cat /tmp/harnesslab-claude-code-install.log >&2; exit 127; }; else echo 'claude CLI missing and npm unavailable' >&2; exit 127; fi; fi".to_string(),
            );
        }
        AgentKind::Opencode => {
            labels.insert(
                "sandbox_setup_command".to_string(),
                "if ! command -v opencode >/dev/null 2>&1; then if command -v npm >/dev/null 2>&1; then npm install -g opencode-ai >/tmp/harnesslab-opencode-install.log 2>&1 || { cat /tmp/harnesslab-opencode-install.log >&2; exit 127; }; else echo 'opencode CLI missing and npm unavailable' >&2; exit 127; fi; fi".to_string(),
            );
        }
        AgentKind::Fake | AgentKind::PiCodingAgent | AgentKind::Custom => {}
    }
    labels
}

fn default_version_command(kind: AgentKind) -> Option<String> {
    match kind {
        AgentKind::PiCodingAgent => Some("pi coding --version || pi --version".to_string()),
        _ => None,
    }
}

pub fn default_auth_config(kind: AgentKind) -> AuthConfig {
    let (inherit_env, include_paths): (&[&str], &[&str]) = match kind {
        AgentKind::Codex => (&["OPENAI_API_KEY"], &["~/.codex:/root/.codex:rw"]),
        AgentKind::ClaudeCode => (&["ANTHROPIC_API_KEY"], &["~/.claude:/root/.claude:ro"]),
        AgentKind::Opencode => (
            &[
                "OPENCODE_API_KEY",
                "OPENAI_API_KEY",
                "ANTHROPIC_API_KEY",
                "GOOGLE_API_KEY",
                "GEMINI_API_KEY",
                "OPENROUTER_API_KEY",
            ],
            &[
                "~/.config/opencode:/root/.config/opencode:ro",
                "~/.opencode:/root/.opencode:ro",
            ],
        ),
        AgentKind::PiCodingAgent => (
            &["PI_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY"],
            &["~/.pi:/root/.pi:ro"],
        ),
        AgentKind::Fake | AgentKind::Custom => (&[], &[]),
    };
    AuthConfig {
        inherit: true,
        inherit_env: inherit_env
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        include_paths: include_paths
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        exclude_paths: Vec::new(),
        mount_ssh_socket: false,
        mount_docker_socket: false,
    }
}

fn default_usage_source() -> String {
    "agent_logs".to_string()
}

fn default_input_tokens_key() -> String {
    "input_tokens".to_string()
}

fn default_output_tokens_key() -> String {
    "output_tokens".to_string()
}

fn default_total_tokens_key() -> String {
    "total_tokens".to_string()
}

fn default_cost_usd_key() -> String {
    "cost_usd".to_string()
}

pub fn is_valid_profile_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphanumeric())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
