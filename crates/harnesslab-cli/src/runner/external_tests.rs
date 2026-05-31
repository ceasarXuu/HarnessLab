use super::terminal_bench_env::{terminal_bench_agent_env, terminal_bench_input_mode};
use super::terminal_bench_timeout::{
    terminal_bench_no_output_timeout_sec, terminal_bench_timeout_values,
};
use harnesslab_core::{
    AgentKind, FailureClass, FailureCode, InputMode, ProcessRecord, TerminationReason,
    default_agent_profile,
};
use std::fs;

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
fn terminal_bench_no_output_timeout_bounds_silent_official_runner_stalls() {
    assert_eq!(
        terminal_bench_no_output_timeout_sec(Some(180), 3600, 3600, None),
        240
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(None, 3600, 3600, None),
        3660
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(None, 3600, 3600, Some("1")),
        1
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(None, 3600, 3600, Some("invalid")),
        3660
    );
}

#[test]
fn terminal_bench_no_output_process_maps_to_external_runner_no_progress() {
    let failure = super::terminal_bench::terminal_bench_process_failure(&ProcessRecord {
        exit_code: None,
        termination_reason: TerminationReason::NoProgress,
        stdout_path: "agent/stdout.log".to_string(),
        stderr_path: "agent/stderr.log".to_string(),
    });

    assert_eq!(failure.class, FailureClass::Execution);
    assert_eq!(failure.code, Some(FailureCode::ExternalRunnerNoProgress));
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

#[test]
fn terminal_bench_result_maps_official_agent_timeout() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_dir = attempt_dir.path().join("official/run-1");
    fs::create_dir_all(&result_dir).unwrap();
    let result_path = result_dir.join("results.json");
    fs::write(
        &result_path,
        r#"{"accuracy":0.0,"n_resolved":0,"n_unresolved":1,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"agent_timeout"}]}"#,
    )
    .unwrap();

    let (_, _, failure_class, failure_code, score) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(score, 0.0);
    assert_eq!(failure_class, FailureClass::Execution);
    assert_eq!(failure_code, Some(FailureCode::AgentTimeout));
}

#[test]
fn terminal_bench_result_maps_official_test_timeout() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"test_timeout"}]}"#,
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::VerifierTimeout));
}

#[test]
fn terminal_bench_result_preserves_success_with_stale_failure_mode() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":1.0,"results":[{"task_id":"hello-world","is_resolved":true,"failure_mode":"agent_timeout"}]}"#,
    );

    let (_, _, failure_class, failure_code, score) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(score, 1.0);
    assert_eq!(failure_class, FailureClass::None);
    assert_eq!(failure_code, None);
}

#[test]
fn terminal_bench_result_ignores_other_task_failure_mode() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"other-task","is_resolved":false,"failure_mode":"agent_timeout"},{"task_id":"hello-world","is_resolved":false}]}"#,
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::TestFailed));
}

#[test]
fn terminal_bench_result_unknown_failure_mode_falls_back_to_test_failed() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"unknown_agent_error"}]}"#,
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::TestFailed));
}

fn write_result(root: &std::path::Path, json: &str) -> std::path::PathBuf {
    let result_dir = root.join("official/run-1");
    fs::create_dir_all(&result_dir).unwrap();
    let result_path = result_dir.join("results.json");
    fs::write(&result_path, json).unwrap();
    result_path
}
