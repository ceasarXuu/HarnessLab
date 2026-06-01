use super::*;
use std::fs;
#[cfg(unix)]
use std::sync::atomic::AtomicI32;
use std::time::{Duration, Instant};

#[test]
fn c_sbox_002_host_exec_echo_captures_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "printf hello".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 5,
        no_output_timeout_sec: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert_eq!(fs::read_to_string(spec.stdout_path).unwrap(), "hello");
}

#[test]
fn c_sbox_003_host_exec_timeout_is_structured() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "sleep 2".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 1,
        no_output_timeout_sec: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::Timeout);
}

#[test]
fn c_sbox_003_host_exec_no_output_timeout_is_structured() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "printf started; sleep 5".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 30,
        no_output_timeout_sec: Some(1),
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "no-output timeout should kill silent descendants promptly"
    );
    assert_eq!(fs::read_to_string(spec.stdout_path).unwrap(), "started");
}

#[test]
fn c_sbox_003_timeout_kills_background_pipe_holder() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "sh -c '(sleep 5; printf late) & sleep 10'".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 1,
        no_output_timeout_sec: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::Timeout);
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "timeout should kill pipe-holding descendants promptly"
    );
}

#[test]
fn c_sbox_002_stdin_broken_pipe_is_not_spawn_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "true".to_string(),
        stdin: Some("ignored input".repeat(1024)),
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 5,
        no_output_timeout_sec: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn c_sbox_002_host_exec_preserves_stdin_through_start_gate() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "cat".to_string(),
        stdin: Some("payload through stdin".to_string()),
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 5,
        no_output_timeout_sec: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.exit_code, Some(0));
    assert_eq!(
        fs::read_to_string(spec.stdout_path).unwrap(),
        "payload through stdin"
    );
}

#[test]
fn c_run_001_command_detection_helpers_are_stable() {
    assert_eq!(first_command_word("sh -c true"), Some("sh"));
    assert!(command_exists("sh"));
    assert!(command_succeeds("true"));
    assert!(!command_succeeds("false"));
}

#[cfg(unix)]
#[test]
fn c_sbox_014_process_group_registry_reports_capacity() {
    let groups = [AtomicI32::new(11), AtomicI32::new(12)];

    assert_eq!(reserve_process_group_slot(&groups, 13), None);
}
