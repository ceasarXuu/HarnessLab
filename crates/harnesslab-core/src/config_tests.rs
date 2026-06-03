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
fn agt_reg_001_profile_deserializes_setup_skills_tools_hooks() {
    let profile: AgentProfile = toml::from_str(
        r#"schema_version = 1
name = "claude-ds"
kind = "claude-code"
display_name = "Claude DS"
command = "claude-ds -p"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 300

[auth]
inherit = true
inherit_env = ["ANTHROPIC_AUTH_TOKEN"]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "builtin"
required_commands = ["claude", "claude-ds"]
run_as = "harnesslab"
commands = []

[skills]
inherit = true
allow = ["skill-a"]
deny = ["skill-b"]
include_paths = ["~/.claude/skills"]

[tools]
inherit = true
allow = []
deny = ["web_search"]

[hooks]
inherit = false
allow = []
deny = []

[usage]
parser = "none"
"#,
    )
    .unwrap();

    assert_eq!(profile.setup.preset, SetupPreset::Builtin);
    assert_eq!(profile.setup.run_as, RunAs::Harnesslab);
    assert_eq!(profile.skills.allow, vec!["skill-a"]);
    assert_eq!(profile.tools.deny, vec!["web_search"]);
    assert!(!profile.hooks.inherit);
}

#[test]
fn agt_reg_001_profile_rejects_setup_and_policy_conflicts() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    profile.setup.preset = SetupPreset::Builtin;
    profile.setup.commands = vec!["echo advanced".to_string()];
    profile.setup.required_commands = vec!["agent | cat".to_string()];
    profile.skills.allow = vec!["skill-a".to_string()];
    profile.skills.deny = vec!["skill-a".to_string()];
    profile.tools.allow = vec!["bash".to_string()];
    profile.tools.deny = vec!["bash".to_string()];
    profile.hooks.allow = vec!["pre".to_string()];
    profile.hooks.deny = vec!["pre".to_string()];

    let report = profile.validation_report();

    assert!(
        report
            .errors
            .iter()
            .any(|error| error.field == "setup.commands")
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.field == "setup.required_commands")
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.field == "skills.allow[0]")
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.field == "tools.allow[0]")
    );
    assert!(
        report
            .errors
            .iter()
            .any(|error| error.field == "hooks.allow[0]")
    );
}

#[test]
fn agt_reg_001_old_profile_shape_gets_defaults() {
    let profile: AgentProfile = toml::from_str(
        r#"schema_version = 1
name = "old"
kind = "custom"
display_name = "Old"
command = "agent"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 60

[auth]
inherit = false
inherit_env = []
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[usage]
parser = "none"
"#,
    )
    .unwrap();

    assert_eq!(profile.setup, SetupConfig::default());
    assert_eq!(profile.skills, CapabilityPolicy::default());
    assert_eq!(profile.tools, CapabilityPolicy::default());
    assert_eq!(profile.hooks, CapabilityPolicy::default());
}

#[test]
fn agt_reg_001_validation_report_covers_field_errors_and_warnings() {
    let mut profile = default_agent_profile("bad/name", AgentKind::Custom, "agent");
    profile.schema_version = 2;
    profile.input_mode = InputMode::Argument;
    profile.timeout_sec = 0;
    profile.auth.mount_docker_socket = true;
    profile.setup.preset = SetupPreset::Builtin;
    profile.setup.commands = vec!["install-agent".to_string()];
    profile.setup.required_commands = vec!["valid-tool".to_string(), "/bad/tool".to_string()];
    profile.skills.allow = vec!["skill/a".to_string(), "dup".to_string()];
    profile.skills.deny = vec!["dup".to_string()];
    profile.tools.allow = vec![" ".to_string()];
    profile.hooks.deny = vec!["hook\\path".to_string()];

    let report = profile.validation_report();
    let fields = report
        .errors
        .iter()
        .map(|error| error.field.as_str())
        .collect::<Vec<_>>();

    assert!(!report.is_valid());
    assert!(fields.contains(&"schema_version"));
    assert!(fields.contains(&"name"));
    assert!(fields.contains(&"command"));
    assert!(fields.contains(&"timeout_sec"));
    assert!(fields.contains(&"setup.commands"));
    assert!(fields.contains(&"setup.required_commands"));
    assert!(fields.contains(&"skills.allow[0]"));
    assert!(fields.contains(&"skills.allow[1]"));
    assert!(fields.contains(&"tools.allow[0]"));
    assert!(fields.contains(&"hooks.deny[0]"));
    assert_eq!(report.warnings[0].field, "auth.mount_docker_socket");
}

#[test]
fn agt_reg_001_validation_report_covers_file_input_and_default_policy() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent run");
    profile.input_mode = InputMode::File;

    let report = profile.validation_report();

    assert!(!report.is_valid());
    assert!(report.errors.iter().any(|error| {
        error.field == "command"
            && error
                .accepted_values
                .iter()
                .any(|value| value == "command containing {{instruction_file}}")
    }));
    profile.command = "agent run {{instruction_file}}".to_string();
    assert!(profile.validation_report().is_valid());
    assert!(crate::policy_is_default(&CapabilityPolicy::default()));
    let custom_policy = CapabilityPolicy {
        inherit: true,
        allow: vec!["skill-a".to_string()],
        deny: Vec::new(),
        include_paths: Vec::new(),
    };
    assert!(!crate::policy_is_default(&custom_policy));
}

#[test]
fn agt_reg_001_validate_covers_field_mapping_and_policy_defaults() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent run");
    profile.input_mode = InputMode::File;
    assert_eq!(profile.validate(), Err(ConfigError::MissingInputVariable));

    profile.command = "agent run {{instruction_file}}".to_string();
    profile.setup.commands = vec!["install-agent".to_string()];
    assert_eq!(
        profile.validate(),
        Err(ConfigError::InvalidField {
            field: "setup.commands".to_string(),
            message: "setup.commands is only valid when setup.preset is custom".to_string(),
            accepted: "see doctor --json details",
        })
    );

    let policy: CapabilityPolicy = toml::from_str(
        r#"allow = ["skill-a"]
deny = []
include_paths = []
"#,
    )
    .unwrap();
    assert!(policy.inherit);

    let mut policy_errors = Vec::new();
    let conflict = CapabilityPolicy {
        inherit: true,
        allow: vec!["same".to_string()],
        deny: vec!["same".to_string()],
        include_paths: Vec::new(),
    };
    crate::validate_policy("capabilities", &conflict, &mut policy_errors);
    assert_eq!(policy_errors[0].field, "capabilities.allow[0]");
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
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "run sk-secret");
    profile.version_command = Some("printf sk-secret".to_string());
    profile.setup.commands = vec!["install sk-secret".to_string()];
    profile.labels.insert(
        "sandbox_setup_command".to_string(),
        "legacy sk-secret".to_string(),
    );

    let snapshot = redacted_profile_snapshot(&profile, &["sk-secret"]);

    assert_eq!(snapshot.command, "run [REDACTED]");
    assert_eq!(
        snapshot.version_command.as_deref(),
        Some("printf [REDACTED]")
    );
    assert_eq!(snapshot.setup.commands, vec!["install [REDACTED]"]);
    assert_eq!(
        snapshot.labels.get("sandbox_setup_command").unwrap(),
        "legacy [REDACTED]"
    );
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
