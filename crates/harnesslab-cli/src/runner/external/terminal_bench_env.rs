use crate::agent_registry::MaterializedAgentProfile;
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use harnesslab_core::{AgentProfile, InputMode};

pub(super) fn terminal_bench_agent_env(
    profile: &AgentProfile,
    materialized: &MaterializedAgentProfile,
    agent_timeout: u64,
    compatibility: &BenchmarkRuntimeCompatibility,
) -> String {
    let input_mode = terminal_bench_input_mode(profile);
    let working_dir = format!("{:?}", profile.working_dir).to_ascii_lowercase();
    let timeout = agent_timeout.to_string();
    let setup_command = materialized.setup_script.as_deref().unwrap_or("");
    let run_as = format!("{:?}", materialized.run_as).to_ascii_lowercase();
    let mut exports = [
        ("HARNESSLAB_AGENT_NAME", profile.name.as_str()),
        ("HARNESSLAB_AGENT_COMMAND", profile.command.as_str()),
        ("HARNESSLAB_AGENT_INPUT_MODE", input_mode),
        ("HARNESSLAB_AGENT_WORKING_DIR", working_dir.as_str()),
        ("HARNESSLAB_AGENT_TIMEOUT_SEC", timeout.as_str()),
        ("HARNESSLAB_AGENT_RUN_AS", run_as.as_str()),
        ("HARNESSLAB_AGENT_SETUP_COMMAND", setup_command),
        (
            "HARNESSLAB_AGENT_SETUP_SUMMARY",
            materialized.setup_summary.as_str(),
        ),
        (
            "HARNESSLAB_AGENT_SKILLS_SUMMARY",
            materialized.skills_summary.as_str(),
        ),
        (
            "HARNESSLAB_AGENT_TOOLS_SUMMARY",
            materialized.tools_summary.as_str(),
        ),
        (
            "HARNESSLAB_AGENT_HOOKS_SUMMARY",
            materialized.hooks_summary.as_str(),
        ),
    ]
    .into_iter()
    .map(|(name, value)| format!("export {name}={}", shell_quote(value)))
    .collect::<Vec<_>>();
    for env_name in &profile.auth.inherit_env {
        if let Ok(value) = std::env::var(env_name) {
            exports.push(format!("export {env_name}={}", shell_quote(&value)));
        }
    }
    if let Some(path) = &compatibility.terminal_bench_agent_pythonpath {
        exports.push(format!(
            "export PYTHONPATH={}${{PYTHONPATH:+:$PYTHONPATH}}",
            shell_quote(path)
        ));
    }
    format!("{};", exports.join("; "))
}

pub(super) fn terminal_bench_input_mode(profile: &AgentProfile) -> &'static str {
    match profile.input_mode {
        InputMode::Stdin | InputMode::Tty => "stdin",
        InputMode::Argument => "argument",
        InputMode::File => "file",
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
