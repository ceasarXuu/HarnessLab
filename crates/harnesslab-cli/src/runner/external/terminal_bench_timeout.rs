pub(super) fn terminal_bench_timeout_values(
    run_timeout: Option<u64>,
    profile_timeout: u64,
    verifier_timeout: u64,
) -> (u64, u64, u64) {
    let agent_timeout = run_timeout.unwrap_or(profile_timeout).max(1);
    let test_timeout = run_timeout.unwrap_or(verifier_timeout).max(1);
    let process_timeout = agent_timeout
        .saturating_add(test_timeout)
        .saturating_add(600);
    (agent_timeout, test_timeout, process_timeout)
}

pub(super) fn terminal_bench_no_output_timeout_sec(
    agent_timeout: u64,
    test_timeout: u64,
    process_timeout: u64,
    override_timeout: Option<&str>,
) -> Option<u64> {
    let cap = process_timeout.saturating_sub(1).max(1);
    if let Some(value) = override_timeout
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "disabled" | "none"
        ) {
            return None;
        }
        if let Ok(timeout) = value.parse::<u64>()
            && timeout > 0
        {
            return Some(timeout.min(cap));
        }
    }
    let default_timeout = agent_timeout.max(test_timeout).saturating_add(120).max(300);
    Some(default_timeout.min(cap))
}

pub(super) fn terminal_bench_process_timeout_sec(
    default_timeout: u64,
    override_timeout: Option<&str>,
) -> u64 {
    override_timeout
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_timeout)
}
