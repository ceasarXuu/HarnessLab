#[cfg(unix)]
use harnesslab_infra::{ExecSpec, HostProcessExecutor};
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::{Child, Command, ExitStatus};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
#[test]
fn c_sbox_014_sigterm_kills_registered_process_group() {
    let tmp = tempfile::tempdir().unwrap();
    let ready_path = tmp.path().join("ready");
    let pid_path = tmp.path().join("child.pid");
    let mut helper = Command::new(std::env::current_exe().unwrap())
        .arg("c_sbox_014_signal_helper_runs_host_executor")
        .arg("--exact")
        .arg("--nocapture")
        .env("HARNESSLAB_PROCESS_SIGNAL_HELPER", "1")
        .env("HARNESSLAB_PROCESS_SIGNAL_READY", &ready_path)
        .env("HARNESSLAB_PROCESS_SIGNAL_PID", &pid_path)
        .spawn()
        .unwrap();

    wait_for_path(&ready_path, Duration::from_secs(5));
    let process_group = fs::read_to_string(&pid_path)
        .unwrap()
        .trim()
        .parse::<i32>()
        .unwrap();
    assert!(process_group_exists(process_group));

    unsafe {
        libc::kill(helper.id() as i32, libc::SIGTERM);
    }

    let status = wait_for_exit(&mut helper, Duration::from_secs(5));
    assert!(!status.success());
    wait_for_process_group_exit(process_group, Duration::from_secs(5));
}

#[cfg(unix)]
#[test]
fn c_sbox_014_signal_helper_runs_host_executor() {
    if std::env::var("HARNESSLAB_PROCESS_SIGNAL_HELPER").as_deref() != Ok("1") {
        return;
    }
    let ready_path = std::env::var("HARNESSLAB_PROCESS_SIGNAL_READY").unwrap();
    let pid_path = std::env::var("HARNESSLAB_PROCESS_SIGNAL_PID").unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let spec = ExecSpec {
        command: format!(
            "echo $$ > {}; touch {}; while :; do sleep 1; done",
            shell_quote(&pid_path),
            shell_quote(&ready_path)
        ),
        stdin: None,
        working_dir: tmp.path().join("workspace"),
        timeout_sec: 30,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: tmp.path().join("stdout.log"),
        stderr_path: tmp.path().join("stderr.log"),
    };

    let result = HostProcessExecutor::exec(&spec).unwrap();
    panic!(
        "helper should be terminated by the parent signal before executor returns: {:?}",
        result.termination_reason
    );
}

#[cfg(unix)]
fn wait_for_path(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("path did not appear before timeout: {}", path.display());
}

#[cfg(unix)]
fn wait_for_exit(child: &mut Child, timeout: Duration) -> ExitStatus {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let _ = child.kill();
    panic!("helper process did not exit before timeout");
}

#[cfg(unix)]
fn wait_for_process_group_exit(process_group: i32, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_group_exists(process_group) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("process group {process_group} still exists after signal cleanup");
}

#[cfg(unix)]
fn process_group_exists(process_group: i32) -> bool {
    unsafe {
        if libc::kill(-process_group, 0) == 0 {
            return true;
        }
    }
    std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}

#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
