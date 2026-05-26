use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

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
            usage_default: UsageConfig {
                parser: "none".to_string(),
            },
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
        if matches!(self.input_mode, InputMode::Argument | InputMode::File)
            && !self.command.contains("{{instruction}}")
        {
            return Err(ConfigError::MissingInputVariable);
        }
        if self.timeout_sec == 0 {
            return Err(ConfigError::InvalidTimeout);
        }
        let mut warnings = Vec::new();
        if self.auth.mount_docker_socket {
            warnings.push(ValidationWarning {
                code: "docker_socket_requested".to_string(),
                message: "mount_docker_socket expands container privileges".to_string(),
            });
        }
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
        version_command: None,
        auth: AuthConfig {
            inherit: true,
            inherit_env: Vec::new(),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            mount_ssh_socket: false,
            mount_docker_socket: false,
        },
        usage: UsageConfig {
            parser: "none".to_string(),
        },
        labels: std::collections::BTreeMap::new(),
    }
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
mod tests {
    use super::*;

    #[test]
    fn cfg_001_valid_global_config_passes() {
        assert_eq!(validate_global_config(&GlobalConfig::default()), Ok(()));
    }

    #[test]
    fn cfg_001_global_config_rejects_schema_and_zero_defaults() {
        let mut config = GlobalConfig {
            schema_version: 2,
            ..GlobalConfig::default()
        };
        assert_eq!(
            validate_global_config(&config),
            Err(ConfigError::UnsupportedSchema(2))
        );

        config = GlobalConfig::default();
        config.default_concurrency = 0;
        assert_eq!(
            validate_global_config(&config),
            Err(ConfigError::InvalidDefaults)
        );
    }

    #[test]
    fn cfg_002_invalid_profile_name_fails() {
        let mut profile = default_agent_profile("-bad", AgentKind::Custom, "agent {{instruction}}");
        profile.input_mode = InputMode::Argument;

        assert_eq!(
            profile.validate(),
            Err(ConfigError::InvalidName("-bad".to_string()))
        );
    }

    #[test]
    fn cfg_002_profile_rejects_unsupported_schema_and_empty_name() {
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
        profile.schema_version = 2;
        assert_eq!(profile.validate(), Err(ConfigError::UnsupportedSchema(2)));

        profile.schema_version = 1;
        profile.name.clear();
        assert_eq!(
            profile.validate(),
            Err(ConfigError::InvalidName(String::new()))
        );
    }

    #[test]
    fn cfg_002_profile_rejects_missing_input_variable_and_timeout() {
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent run");
        profile.input_mode = InputMode::Argument;
        assert_eq!(profile.validate(), Err(ConfigError::MissingInputVariable));

        profile.input_mode = InputMode::Stdin;
        profile.timeout_sec = 0;
        assert_eq!(profile.validate(), Err(ConfigError::InvalidTimeout));
    }

    #[test]
    fn cfg_004_path_expands_home_and_relative_paths() {
        let home = Path::new("/home/test");
        let base = Path::new("/repo");

        assert_eq!(
            expand_path("~/runs", home, base),
            PathBuf::from("/home/test/runs")
        );
        assert_eq!(expand_path("runs", home, base), PathBuf::from("/repo/runs"));
        assert_eq!(expand_path("~", home, base), PathBuf::from("/home/test"));
        assert_eq!(expand_path("/abs", home, base), PathBuf::from("/abs"));
    }

    #[test]
    fn cfg_005_profile_snapshot_redacts_command_secret() {
        let profile = default_agent_profile("custom", AgentKind::Custom, "run sk-secret");

        let snapshot = redacted_profile_snapshot(&profile, &["sk-secret"]);

        assert_eq!(snapshot.command, "run [REDACTED]");
    }

    #[test]
    fn agt_005_docker_socket_requested_warns() {
        let mut profile =
            default_agent_profile("custom", AgentKind::Custom, "agent {{instruction}}");
        profile.input_mode = InputMode::Argument;
        profile.auth.mount_docker_socket = true;

        let warnings = profile.validate().unwrap();

        assert_eq!(warnings[0].code, "docker_socket_requested");
    }
}
