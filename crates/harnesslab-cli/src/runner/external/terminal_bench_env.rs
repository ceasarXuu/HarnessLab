use harnesslab_core::{AgentProfile, InputMode};

pub(super) fn terminal_bench_agent_env(profile: &AgentProfile, agent_timeout: u64) -> String {
    let input_mode = terminal_bench_input_mode(profile);
    let working_dir = format!("{:?}", profile.working_dir).to_ascii_lowercase();
    let timeout = agent_timeout.to_string();
    let mut exports = [
        ("HARNESSLAB_AGENT_NAME", profile.name.as_str()),
        ("HARNESSLAB_AGENT_COMMAND", profile.command.as_str()),
        ("HARNESSLAB_AGENT_INPUT_MODE", input_mode),
        ("HARNESSLAB_AGENT_WORKING_DIR", working_dir.as_str()),
        ("HARNESSLAB_AGENT_TIMEOUT_SEC", timeout.as_str()),
    ]
    .into_iter()
    .map(|(name, value)| format!("export {name}={}", shell_quote(value)))
    .collect::<Vec<_>>();
    if let Some(path) = profile.labels.get("terminal_bench_agent_pythonpath")
        && !path.trim().is_empty()
    {
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
