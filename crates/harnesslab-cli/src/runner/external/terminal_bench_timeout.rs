pub(super) fn terminal_bench_timeout_values(
    run_timeout: Option<u64>,
    profile_timeout: u64,
    verifier_timeout: u64,
) -> (u64, u64, u64) {
    let agent_timeout = run_timeout.unwrap_or(profile_timeout).max(1);
    let test_timeout = run_timeout.unwrap_or(verifier_timeout).max(1);
    let process_timeout = agent_timeout.max(test_timeout).saturating_add(600);
    (agent_timeout, test_timeout, process_timeout)
}
