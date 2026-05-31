use anyhow::{Context, Result};
use harnesslab_core::{ProcessRecord, TerminationReason};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ExecSpec {
    pub command: String,
    pub stdin: Option<String>,
    pub working_dir: std::path::PathBuf,
    pub timeout_sec: u64,
    pub stdout_path: std::path::PathBuf,
    pub stderr_path: std::path::PathBuf,
}

pub struct HostProcessExecutor;

impl HostProcessExecutor {
    pub fn exec(spec: &ExecSpec) -> Result<ProcessRecord> {
        if let Some(parent) = spec.stdout_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = spec.stderr_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&spec.working_dir)?;

        let child = spawn_child(spec);

        let mut child = match child {
            Ok(child) => child,
            Err(error) => {
                fs::write(&spec.stdout_path, "")?;
                fs::write(&spec.stderr_path, error.to_string())?;
                return Ok(record(spec, None, TerminationReason::SpawnError));
            }
        };

        if let Some(stdin) = &spec.stdin
            && let Some(mut pipe) = child.stdin.take()
            && let Err(error) = pipe.write_all(stdin.as_bytes())
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            return Err(error.into());
        }
        drop(child.stdin.take());

        let mut stdout_thread = Some(stream_child_output(
            child.stdout.take(),
            spec.stdout_path.clone(),
        ));
        let mut stderr_thread = Some(stream_child_output(
            child.stderr.take(),
            spec.stderr_path.clone(),
        ));

        let deadline = Instant::now() + Duration::from_secs(spec.timeout_sec.max(1));
        loop {
            if let Some(status) = child.try_wait()? {
                join_stream(stdout_thread.take().expect("stdout stream thread"))?;
                join_stream(stderr_thread.take().expect("stderr stream thread"))?;
                return Ok(record(
                    spec,
                    status.code(),
                    if status.code().is_some() {
                        TerminationReason::Completed
                    } else {
                        TerminationReason::Signaled
                    },
                ));
            }
            if Instant::now() >= deadline {
                kill_process_tree(&mut child);
                child.wait().context("wait for killed process")?;
                join_stream_timeout(
                    stdout_thread.take().expect("stdout stream thread"),
                    Duration::from_secs(2),
                )?;
                join_stream_timeout(
                    stderr_thread.take().expect("stderr stream thread"),
                    Duration::from_secs(2),
                )?;
                return Ok(record(spec, None, TerminationReason::Timeout));
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn spawn_child(spec: &ExecSpec) -> std::io::Result<Child> {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(&spec.command)
        .current_dir(&spec.working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_child_process_group(&mut command);
    command.spawn()
}

#[cfg(unix)]
fn configure_child_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn configure_child_process_group(_command: &mut Command) {}

fn kill_process_tree(child: &mut Child) {
    #[cfg(unix)]
    {
        let pgid = -(child.id() as i32);
        unsafe {
            let _ = libc::kill(pgid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
}

fn stream_child_output<R>(
    pipe: Option<R>,
    path: std::path::PathBuf,
) -> thread::JoinHandle<Result<()>>
where
    R: std::io::Read + Send + 'static,
{
    thread::spawn(move || {
        let mut file = fs::File::create(path)?;
        if let Some(mut pipe) = pipe {
            std::io::copy(&mut pipe, &mut file)?;
        }
        Ok(())
    })
}

fn join_stream(handle: thread::JoinHandle<Result<()>>) -> Result<()> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("streaming log writer panicked"))?
}

fn join_stream_timeout(handle: thread::JoinHandle<Result<()>>, timeout: Duration) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = handle
            .join()
            .map_err(|_| anyhow::anyhow!("streaming log writer panicked"))?;
        let _ = tx.send(result);
        Ok::<(), anyhow::Error>(())
    });
    rx.recv_timeout(timeout)
        .map_err(|_| anyhow::anyhow!("streaming log writer did not finish after process kill"))?
}

pub fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", shell_quote(command)))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn command_succeeds(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn first_command_word(command: &str) -> Option<&str> {
    command.split_whitespace().next()
}

fn record(spec: &ExecSpec, exit_code: Option<i32>, reason: TerminationReason) -> ProcessRecord {
    ProcessRecord {
        exit_code,
        termination_reason: reason,
        stdout_path: rel_or_display(&spec.stdout_path),
        stderr_path: rel_or_display(&spec.stderr_path),
    }
}

fn rel_or_display(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), str::to_string)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_sbox_002_host_exec_echo_captures_stdout() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = ExecSpec {
            command: "printf hello".to_string(),
            stdin: None,
            working_dir: tmp.path().join("workspace"),
            timeout_sec: 5,
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
            stdout_path: tmp.path().join("stdout.log"),
            stderr_path: tmp.path().join("stderr.log"),
        };

        let result = HostProcessExecutor::exec(&spec).unwrap();

        assert_eq!(result.termination_reason, TerminationReason::Timeout);
    }

    #[test]
    fn c_sbox_003_timeout_kills_background_pipe_holder() {
        let tmp = tempfile::tempdir().unwrap();
        let spec = ExecSpec {
            command: "sh -c '(sleep 5; printf late) & sleep 10'".to_string(),
            stdin: None,
            working_dir: tmp.path().join("workspace"),
            timeout_sec: 1,
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
            stdout_path: tmp.path().join("stdout.log"),
            stderr_path: tmp.path().join("stderr.log"),
        };

        let result = HostProcessExecutor::exec(&spec).unwrap();

        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn c_run_001_command_detection_helpers_are_stable() {
        assert_eq!(first_command_word("sh -c true"), Some("sh"));
        assert!(command_exists("sh"));
        assert!(command_succeeds("true"));
        assert!(!command_succeeds("false"));
    }
}
