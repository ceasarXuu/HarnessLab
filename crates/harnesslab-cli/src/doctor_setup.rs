use crate::doctor::check_with_details;
use crate::output::DoctorCheck;
use harnesslab_core::{AgentKind, AgentProfile, SetupPreset};
use harnesslab_infra::command_exists;

pub(crate) fn append_required_commands_check(
    profile: &AgentProfile,
    checks: &mut Vec<DoctorCheck>,
) {
    if profile.setup.required_commands.is_empty() {
        return;
    }
    let commands = profile
        .setup
        .required_commands
        .iter()
        .enumerate()
        .map(|(index, command)| required_command_state(profile, index, command))
        .collect::<Vec<_>>();
    let has_error = commands
        .iter()
        .any(|command| command["status"].as_str() == Some("error"));
    checks.push(check_with_details(
        &format!("agent.{}.setup.required_commands", profile.name),
        if has_error { "error" } else { "ok" },
        "error",
        "Agent setup required commands checked",
        serde_json::json!({ "commands": commands }),
    ));
}

fn required_command_state(
    profile: &AgentProfile,
    index: usize,
    command: &str,
) -> serde_json::Value {
    let valid_name = harnesslab_core::is_valid_command_name(command);
    let host_available = valid_name && command_exists(command);
    let provider = required_command_provider(profile, command);
    let status = if !valid_name || (!host_available && provider == "none") {
        "error"
    } else {
        "ok"
    };
    serde_json::json!({
        "field": format!("setup.required_commands[{index}]"),
        "command": command,
        "valid_name": valid_name,
        "host_available": host_available,
        "setup_preset": format!("{:?}", profile.setup.preset).to_ascii_lowercase(),
        "provider": provider,
        "status": status,
        "message": required_command_message(valid_name, host_available, provider),
    })
}

fn required_command_message(
    valid_name: bool,
    host_available: bool,
    provider: &str,
) -> &'static str {
    if !valid_name {
        "invalid command name; use bare command names, not shell pipelines or paths"
    } else if provider == "builtin_setup" {
        "required command is expected to be provided by builtin setup"
    } else if host_available {
        "required command is available on host"
    } else if provider == "custom_setup" {
        "required command availability depends on custom setup inside the sandbox"
    } else {
        "required command is missing and no setup path is expected to provide it"
    }
}

fn required_command_provider(profile: &AgentProfile, command: &str) -> &'static str {
    match profile.setup.preset {
        SetupPreset::None => "none",
        SetupPreset::Custom => "custom_setup",
        SetupPreset::Builtin if builtin_setup_can_provide(profile.kind, command) => "builtin_setup",
        SetupPreset::Builtin => "none",
    }
}

fn builtin_setup_can_provide(kind: AgentKind, command: &str) -> bool {
    matches!(
        (kind, command),
        (AgentKind::Codex, "codex")
            | (AgentKind::ClaudeCode, "claude")
            | (AgentKind::ClaudeCode, "claude-ds")
            | (AgentKind::Opencode, "opencode")
    )
}
