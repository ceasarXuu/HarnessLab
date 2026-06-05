use super::terminal_bench_env::{terminal_bench_agent_env, terminal_bench_input_mode};
use super::terminal_bench_timeout::{
    terminal_bench_no_output_timeout_sec, terminal_bench_process_timeout_sec,
    terminal_bench_timeout_values,
};
use crate::agent_registry::{
    MaterializedAgentProfile, materialize_profile, resolve_profile_capabilities,
};
use crate::runtime_compatibility::BenchmarkRuntimeCompatibility;
use harnesslab_core::{
    AgentKind, FailureClass, FailureCode, InputMode, ProcessRecord, RunAs, TerminationReason,
    default_agent_profile,
};
use std::fs;

#[test]
fn terminal_bench_timeout_values_use_run_override_when_present() {
    assert_eq!(
        terminal_bench_timeout_values(Some(42), 5, 7, None),
        (42, 7, 1849)
    );
}

#[test]
fn terminal_bench_timeout_values_fall_back_to_profile_and_verifier() {
    assert_eq!(
        terminal_bench_timeout_values(None, 5, 7, None),
        (5, 7, 1812)
    );
    assert_eq!(
        terminal_bench_timeout_values(None, 0, 0, None),
        (1, 1, 1802)
    );
}

#[test]
fn terminal_bench_no_output_timeout_defaults_to_setup_watchdog() {
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 2400, None),
        Some(1800)
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(5, 7, 1812, None),
        Some(1800)
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 120, None),
        Some(119)
    );
}

#[test]
fn terminal_bench_no_output_timeout_can_be_overridden_or_disabled() {
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 1200, Some("1")),
        Some(1)
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 1200, Some("off")),
        None
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 1200, Some("0")),
        None
    );
    assert_eq!(
        terminal_bench_no_output_timeout_sec(300, 300, 2400, Some("invalid")),
        Some(1800)
    );
}

#[test]
fn terminal_bench_no_output_activity_patterns_are_setup_scoped() {
    let patterns = super::terminal_bench_runtime::terminal_bench_no_output_activity_patterns();

    assert!(patterns.contains(&"docker compose".to_string()));
    assert!(patterns.contains(&"docker-buildx".to_string()));
    assert!(!patterns.contains(&"docker exec".to_string()));
}

#[test]
fn terminal_bench_process_timeout_can_be_overridden_for_diagnostics() {
    assert_eq!(terminal_bench_process_timeout_sec(960, None), 960);
    assert_eq!(
        terminal_bench_process_timeout_sec(960, Some("invalid")),
        960
    );
    assert_eq!(terminal_bench_process_timeout_sec(960, Some("2")), 2);
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
fn terminal_bench_hard_timeout_maps_to_external_runner_timeout() {
    let failure = super::terminal_bench::terminal_bench_process_failure(&ProcessRecord {
        exit_code: None,
        termination_reason: TerminationReason::Timeout,
        stdout_path: "agent/stdout.log".to_string(),
        stderr_path: "agent/stderr.log".to_string(),
    });

    assert_eq!(failure.class, FailureClass::Execution);
    assert_eq!(failure.code, Some(FailureCode::ExternalRunnerTimeout));
}

#[test]
fn terminal_bench_env_uses_effective_agent_timeout() {
    let profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    let materialized = materialize_profile(&profile).unwrap();

    let compatibility = BenchmarkRuntimeCompatibility::from_profile(&profile);
    let env = terminal_bench_agent_env(&profile, &materialized, 42, &compatibility);

    assert!(env.contains("export HARNESSLAB_AGENT_TIMEOUT_SEC='42'"));
    assert!(env.contains("export HARNESSLAB_AGENT_SETUP_COMMAND=''"));
}

#[test]
fn agt_reg_005_terminal_bench_env_uses_materialized_setup_not_raw_profile() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    profile.setup.commands = vec!["raw-profile-setup".to_string()];
    let materialized = MaterializedAgentProfile {
        setup_script: Some("materialized-only-setup".to_string()),
        setup_summary: "summary".to_string(),
        skills_summary: "skills".to_string(),
        tools_summary: "tools".to_string(),
        hooks_summary: "hooks".to_string(),
        capabilities: resolve_profile_capabilities(&profile),
        run_as: RunAs::Current,
        warnings: Vec::new(),
    };

    let compatibility = BenchmarkRuntimeCompatibility::from_profile(&profile);
    let env = terminal_bench_agent_env(&profile, &materialized, 42, &compatibility);

    assert!(env.contains("export HARNESSLAB_AGENT_SETUP_COMMAND='materialized-only-setup'"));
    assert!(!env.contains("raw-profile-setup"));
}

#[test]
fn terminal_bench_import_agent_official_timeout_adds_cleanup_grace() {
    assert_eq!(
        super::terminal_bench_adapter::terminal_bench_official_agent_timeout(300, true),
        330
    );
    assert_eq!(
        super::terminal_bench_adapter::terminal_bench_official_agent_timeout(300, false),
        300
    );
}

#[test]
fn terminal_bench_tty_mode_maps_to_stdin_for_import_agent() {
    let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
    profile.input_mode = InputMode::Tty;
    let materialized = materialize_profile(&profile).unwrap();

    assert_eq!(terminal_bench_input_mode(&profile), "stdin");
    assert!(
        terminal_bench_agent_env(
            &profile,
            &materialized,
            5,
            &BenchmarkRuntimeCompatibility::from_profile(&profile)
        )
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
    assert_eq!(failure_class, FailureClass::Benchmark);
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

#[test]
fn terminal_bench_result_maps_parse_error_to_agent_output_parse_error() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"parse_error"}]}"#,
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::AgentOutputParseError));
}

#[test]
fn terminal_bench_result_adapter_timeout_log_overrides_parse_error() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"parse_error"}]}"#,
    );
    write_agent_error_log(
        &result_path,
        "agent command timed out; configured_timeout_sec=3; cleanup=root_pid=1 succeeded=True",
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::AgentTimeout));
}

#[test]
fn terminal_bench_result_failed_adapter_cleanup_is_execution_failure() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"parse_error"}]}"#,
    );
    write_agent_error_log(
        &result_path,
        "agent command timed out; configured_timeout_sec=3; cleanup=root_pid=1 succeeded=False",
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Execution);
    assert_eq!(failure_code, Some(FailureCode::AgentCleanupFailed));
}

#[test]
fn terminal_bench_result_failed_adapter_cleanup_overrides_success_score() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":1.0,"n_resolved":1,"n_unresolved":0,"results":[{"task_id":"hello-world","is_resolved":true}]}"#,
    );
    write_agent_cleanup_log(&result_path, "root_pid=1 alive_pids=[2] succeeded=False");

    let (evaluation, _, failure_class, failure_code, score) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(evaluation.raw_score, 0.0);
    assert_eq!(score, 0.0);
    assert_eq!(failure_class, FailureClass::Execution);
    assert_eq!(failure_code, Some(FailureCode::AgentCleanupFailed));
}

#[test]
fn terminal_bench_result_live_child_cleanup_error_is_execution_failure() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"unknown_agent_error"}]}"#,
    );
    write_agent_error_log(
        &result_path,
        "agent command exited but left live child processes: root_pid=1 alive_pids=[2]",
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Execution);
    assert_eq!(failure_code, Some(FailureCode::AgentCleanupFailed));
}

#[test]
fn terminal_bench_result_live_child_cleanup_log_is_execution_failure() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"unknown_agent_error"}]}"#,
    );
    write_agent_cleanup_log(
        &result_path,
        "agent command exited but left live child processes: root_pid=1 alive_pids=[2]",
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Execution);
    assert_eq!(failure_code, Some(FailureCode::AgentCleanupFailed));
}

#[test]
fn terminal_bench_result_adapter_timeout_log_overrides_later_test_timeout() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"test_timeout"}]}"#,
    );
    write_agent_error_log(
        &result_path,
        "agent command timed out; configured_timeout_sec=3; cleanup=root_pid=1 succeeded=True",
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::AgentTimeout));
}

#[test]
fn terminal_bench_result_stale_adapter_timeout_log_keeps_parse_error() {
    let attempt_dir = tempfile::tempdir().unwrap();
    let stale_log_dir = attempt_dir
        .path()
        .join("official/stale-run/task/agent-logs");
    fs::create_dir_all(&stale_log_dir).unwrap();
    fs::write(
        stale_log_dir.join("agent_error.log"),
        "agent command timed out; configured_timeout_sec=3; cleanup=root_pid=1 succeeded=True",
    )
    .unwrap();
    let result_path = write_result(
        attempt_dir.path(),
        r#"{"accuracy":0.0,"results":[{"task_id":"hello-world","is_resolved":false,"failure_mode":"parse_error"}]}"#,
    );

    let (_, _, failure_class, failure_code, _) =
        super::terminal_bench::parse_terminal_bench_result(
            attempt_dir.path(),
            &result_path,
            "hello-world",
        )
        .unwrap();

    assert_eq!(failure_class, FailureClass::Benchmark);
    assert_eq!(failure_code, Some(FailureCode::AgentOutputParseError));
}

fn write_result(root: &std::path::Path, json: &str) -> std::path::PathBuf {
    let result_dir = root.join("official/run-1");
    fs::create_dir_all(&result_dir).unwrap();
    let result_path = result_dir.join("results.json");
    fs::write(&result_path, json).unwrap();
    result_path
}

fn write_agent_error_log(result_path: &std::path::Path, content: &str) {
    let log_dir = result_path.parent().unwrap().join("task/agent-logs");
    fs::create_dir_all(&log_dir).unwrap();
    fs::write(log_dir.join("agent_error.log"), content).unwrap();
}

fn write_agent_cleanup_log(result_path: &std::path::Path, content: &str) {
    let log_dir = result_path.parent().unwrap().join("task/agent-logs");
    fs::create_dir_all(&log_dir).unwrap();
    fs::write(log_dir.join("agent_cleanup.log"), content).unwrap();
}
