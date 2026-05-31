use super::{terminal_bench_agent_env, terminal_bench_input_mode, terminal_bench_timeout_values};
use harnesslab_core::{AgentKind, InputMode, default_agent_profile};

#[test]
fn terminal_bench_timeout_values_use_run_override_when_present() {
    assert_eq!(terminal_bench_timeout_values(Some(42), 5, 7), (42, 42, 642));
}

#[test]
fn terminal_bench_timeout_values_fall_back_to_profile_and_verifier() {
    assert_eq!(terminal_bench_timeout_values(None, 5, 7), (5, 7, 607));
    assert_eq!(terminal_bench_timeout_values(None, 0, 0), (1, 1, 601));
}

#[test]
fn terminal_bench_env_uses_effective_agent_timeout() {
    let profile = default_agent_profile("custom", AgentKind::Custom, "agent");

    let env = terminal_bench_agent_env(&profile, 42);

    assert!(env.contains("export HARNESSLAB_AGENT_TIMEOUT_SEC='42'"));
}

#[test]
fn terminal_bench_tty_mode_maps_to_stdin_for_import_agent() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    profile.input_mode = InputMode::Tty;

    assert_eq!(terminal_bench_input_mode(&profile), "stdin");
    assert!(
        terminal_bench_agent_env(&profile, 5)
            .contains("export HARNESSLAB_AGENT_INPUT_MODE='stdin'")
    );
}
