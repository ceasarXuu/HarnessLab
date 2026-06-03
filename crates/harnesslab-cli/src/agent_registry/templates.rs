use harnesslab_core::{AgentKind, default_agent_profile};

pub(crate) fn profile_template(name: &str, kind: AgentKind, command: &str) -> String {
    let profile = default_agent_profile(name, kind, command);
    let kind_value = kind_value(kind);
    let auth_env = quoted_array(&profile.auth.inherit_env);
    let auth_paths = quoted_array(&profile.auth.include_paths);
    let required = quoted_array(&profile.setup.required_commands);
    let version = profile
        .version_command
        .as_ref()
        .map(|command| format!("version_command = \"{}\"\n", escape_toml(command)))
        .unwrap_or_default();
    format!(
        r#"# HarnessLab agent profile.
# Edit this file, then run: harnesslab doctor
schema_version = 1
name = "{name}"
kind = "{kind_value}"
display_name = "{name}"
command = "{command}"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 3600
{version}
[auth]
inherit = true
inherit_env = [{auth_env}]
include_paths = [{auth_paths}]
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "{preset}"
required_commands = [{required}]
run_as = "{run_as}"
commands = []

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = []

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"
source = "agent_logs"
input_tokens_key = "input_tokens"
output_tokens_key = "output_tokens"
total_tokens_key = "total_tokens"
cost_usd_key = "cost_usd"

[labels]
model = "user-configured"
"#,
        preset = format!("{:?}", profile.setup.preset).to_ascii_lowercase(),
        run_as = format!("{:?}", profile.setup.run_as).to_ascii_lowercase(),
        command = escape_toml(command),
    )
}

pub(crate) fn agents_readme() -> &'static str {
    "# HarnessLab Agent Profiles\n\nEdit `*.toml` files in this directory, then run `harnesslab doctor`.\n\nUseful commands:\n\n```bash\nharnesslab agent list\nharnesslab agent schema --json\nharnesslab doctor --json\n```\n\nUse `[skills]`, `[tools]`, and `[hooks]` allow/deny lists only when HarnessLab can materialize them for that agent kind; otherwise doctor will block the run.\n"
}

fn kind_value(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Codex => "codex",
        AgentKind::ClaudeCode => "claude-code",
        AgentKind::Opencode => "opencode",
        AgentKind::PiCodingAgent => "pi-coding-agent",
        AgentKind::Fake => "fake",
        AgentKind::Custom => "custom",
    }
}

fn quoted_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", escape_toml(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
