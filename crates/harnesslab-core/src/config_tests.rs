use super::*;
use std::path::{Path, PathBuf};

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

    profile.input_mode = InputMode::File;
    profile.command = "agent run {{instruction_file}}".to_string();
    assert!(profile.validate().is_ok());

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
fn cfg_004_auth_mount_parser_matches_runtime_mount_contract() {
    let home = std::env::var("HOME").unwrap_or_default();

    assert_eq!(
        parse_auth_mount("~/.codex:/root/.codex:ro").unwrap(),
        AuthMountSpec {
            host: format!("{home}/.codex"),
            mount: format!("{home}/.codex:/root/.codex:ro"),
        }
    );
    assert_eq!(
        parse_auth_mount("/host/cache:/cache").unwrap(),
        AuthMountSpec {
            host: "/host/cache".to_string(),
            mount: "/host/cache:/cache:ro".to_string(),
        }
    );
    assert_eq!(
        parse_auth_mount("relative").unwrap(),
        AuthMountSpec {
            host: "relative".to_string(),
            mount: "relative:relative:ro".to_string(),
        }
    );
    assert!(parse_auth_mount("a:b:c:d").is_none());
}

#[test]
fn cfg_004_effective_auth_mount_specs_match_runtime_rules() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent run");
    profile.auth.include_paths = vec![
        "/host/active:/root/active:ro".to_string(),
        "/host/excluded:/root/excluded:ro".to_string(),
        "/host/active:/root/active:ro".to_string(),
        "invalid:a:b:c".to_string(),
    ];
    profile.auth.exclude_paths = vec!["/host/excluded".to_string()];

    let specs = effective_auth_mount_specs(&profile);

    assert_eq!(
        specs,
        vec![AuthMountSpec {
            host: "/host/active".to_string(),
            mount: "/host/active:/root/active:ro".to_string(),
        }]
    );

    profile.auth.inherit = false;
    assert!(effective_auth_mount_specs(&profile).is_empty());
}

#[test]
fn cfg_005_profile_snapshot_redacts_command_secret() {
    let profile = default_agent_profile("custom", AgentKind::Custom, "run sk-secret");

    let snapshot = redacted_profile_snapshot(&profile, &["sk-secret"]);

    assert_eq!(snapshot.command, "run [REDACTED]");
}

#[test]
fn agt_005_docker_socket_requested_warns() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent {{instruction}}");
    profile.input_mode = InputMode::Argument;
    profile.auth.mount_docker_socket = true;

    let warnings = profile.validate().unwrap();

    assert_eq!(warnings[0].code, "docker_socket_requested");
}

#[test]
fn agt_006_builtin_profiles_expand_auth_defaults() {
    let codex = default_agent_profile("codex-default", AgentKind::Codex, "codex -");
    assert!(
        codex
            .auth
            .inherit_env
            .contains(&"OPENAI_API_KEY".to_string())
    );
    assert!(
        codex
            .auth
            .include_paths
            .contains(&"~/.codex:/root/.codex:rw".to_string())
    );
    assert!(
        codex
            .labels
            .get("sandbox_setup_command")
            .is_some_and(|value| value.contains("@openai/codex"))
    );

    let pi = default_agent_profile("pi-coding-agent-default", AgentKind::PiCodingAgent, "pi -");
    assert_eq!(
        pi.version_command.as_deref(),
        Some("pi coding --version || pi --version")
    );
    assert!(pi.auth.inherit_env.contains(&"PI_API_KEY".to_string()));
}
