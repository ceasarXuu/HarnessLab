use anyhow::{Context, Result};
use harnesslab_core::{ProcessRecord, TerminationReason};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
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

        let child = Command::new("sh")
            .arg("-c")
            .arg(&spec.command)
            .current_dir(&spec.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

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
        {
            use std::io::Write;
            pipe.write_all(stdin.as_bytes())?;
        }

        let deadline = Instant::now() + Duration::from_secs(spec.timeout_sec.max(1));
        loop {
            if let Some(status) = child.try_wait()? {
                let (stdout, stderr) = read_child_output(&mut child)?;
                fs::write(&spec.stdout_path, stdout)?;
                fs::write(&spec.stderr_path, stderr)?;
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
                let _ = child.kill();
                child.wait().context("wait for killed process")?;
                let (stdout, stderr) = read_child_output(&mut child)?;
                fs::write(&spec.stdout_path, stdout)?;
                fs::write(&spec.stderr_path, stderr)?;
                return Ok(record(spec, None, TerminationReason::Timeout));
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn read_child_output(child: &mut std::process::Child) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    if let Some(mut pipe) = child.stdout.take() {
        pipe.read_to_end(&mut stdout)?;
    }
    if let Some(mut pipe) = child.stderr.take() {
        pipe.read_to_end(&mut stderr)?;
    }
    Ok((stdout, stderr))
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
    fn c_run_001_command_detection_helpers_are_stable() {
        assert_eq!(first_command_word("sh -c true"), Some("sh"));
        assert!(command_exists("sh"));
        assert!(command_succeeds("true"));
        assert!(!command_succeeds("false"));
    }
}
