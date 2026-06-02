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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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

#[cfg(unix)]
#[test]
fn c_sbox_003_no_output_activity_pattern_defers_to_hard_timeout() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "printf started; sleep 5".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 2,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: vec!["sleep 5".to_string()],
        no_output_activity_event: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::Timeout);
    assert!(
        started.elapsed() >= Duration::from_secs(2),
        "matching activity should prevent the no-output watchdog from firing first"
    );
    assert_eq!(fs::read_to_string(spec.stdout_path).unwrap(), "started");
}

#[cfg(unix)]
#[test]
fn c_sbox_018_no_output_activity_has_bounded_grace() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "printf started; sleep 8".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 10,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: vec!["sleep 8".to_string()],
        no_output_activity_event: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    assert!(
        started.elapsed() < Duration::from_millis(2_700),
        "matching activity must not defer for more than one extra watchdog window"
    );
}

#[cfg(unix)]
#[test]
fn c_sbox_003_no_output_activity_disappearing_kills_promptly() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "printf started; sleep 2; sleep 10".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 10,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: vec!["sleep 2".to_string()],
        no_output_activity_event: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "activity disappearance should not grant a full extra watchdog window"
    );
}

#[cfg(unix)]
#[test]
fn c_sbox_003_no_output_activity_ignores_shell_text_mentions() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: "sh -c 'printf started; : docker buildx; sleep 5'".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 10,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: vec!["docker buildx".to_string()],
        no_output_activity_event: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    assert!(
        started.elapsed() < Duration::from_secs(4),
        "shell argv text should not be treated as Docker setup activity"
    );
}

#[test]
fn c_sbox_003_no_output_progress_file_defers_to_hard_timeout() {
    let tmp = tempfile::tempdir().unwrap();
    let progress_path = tmp.path().join("run.log");
    let events_path = tmp.path().join("events.jsonl");
    let spec = ExecSpec {
        command: format!(
            "printf started; sleep 1; printf progress >> {}; sleep 5",
            shell_quote(&progress_path.display().to_string())
        ),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 4,
        no_output_timeout_sec: Some(2),
        no_output_progress_paths: vec![progress_path],
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: Some(NoOutputActivityEvent {
            path: events_path.clone(),
            run_id: "run-1".to_string(),
            task_id: Some("task-1".to_string()),
            event_name: "external_runner_activity".to_string(),
            no_progress_event_name: None,
        }),
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::Timeout);
    assert!(
        started.elapsed() >= Duration::from_secs(4),
        "progress file growth should defer no-output until hard timeout"
    );
    let events = fs::read_to_string(events_path).unwrap();
    assert!(events.contains("external_runner_activity"));
    assert!(events.contains("progress file path="));
}

#[cfg(unix)]
#[test]
fn c_sbox_018_progress_growth_resets_activity_grace() {
    let tmp = tempfile::tempdir().unwrap();
    let progress_path = tmp.path().join("run.log");
    let spec = ExecSpec {
        command: format!(
            "printf started; sleep 1; printf progress >> {}; sleep 8",
            shell_quote(&progress_path.display().to_string())
        ),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 10,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: vec![progress_path],
        no_output_activity_patterns: vec!["sleep 8".to_string()],
        no_output_activity_event: None,
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };
    let started = Instant::now();

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    assert!(
        started.elapsed() >= Duration::from_secs(3),
        "progress growth should reset the activity grace before the later stale activity window"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "later stale activity should still receive only one extra watchdog window"
    );
}

#[cfg(unix)]
#[test]
fn c_sbox_019_activity_event_emits_after_output_reset() {
    let tmp = tempfile::tempdir().unwrap();
    let events_path = tmp.path().join("events.jsonl");
    let spec = ExecSpec {
        command: "printf started; sleep 1.8; printf reset; sleep 8".to_string(),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 10,
        no_output_timeout_sec: Some(1),
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: vec!["sleep 1.8".to_string(), "sleep 8".to_string()],
        no_output_activity_event: Some(NoOutputActivityEvent {
            path: events_path.clone(),
            run_id: "run-1".to_string(),
            task_id: Some("task-1".to_string()),
            event_name: "external_runner_activity".to_string(),
            no_progress_event_name: Some("external_runner_no_progress".to_string()),
        }),
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();

    assert_eq!(result.termination_reason, TerminationReason::NoProgress);
    let events = fs::read_to_string(events_path).unwrap();
    assert_eq!(
        events
            .matches("\"event\":\"external_runner_activity\"")
            .count(),
        2,
        "each activity grace window should persist exactly one activity event: {events}"
    );
    assert!(events.contains("\"event\":\"external_runner_no_progress\""));
    assert!(events.contains("pattern=sleep 1.8"));
    assert!(events.contains("pattern=sleep 8"));
    assert_eq!(
        events
            .lines()
            .filter(|line| {
                line.contains("\"event\":\"external_runner_activity\"")
                    && line.contains("pattern=sleep 8")
            })
            .count(),
        1
    );
    assert!(events.find("pattern=sleep 8") < events.find("external_runner_no_progress"));
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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
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
