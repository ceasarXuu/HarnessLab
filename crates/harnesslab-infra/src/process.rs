use crate::{NoOutputActivityEvent, process_group_activity};
use anyhow::{Context, Result};
use harnesslab_core::{ProcessRecord, TerminationReason};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU64, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

#[path = "process_start_gate.rs"]
mod process_start_gate;
use process_start_gate::{ChildStartGate, ChildStartGateFds};

#[path = "process_no_output.rs"]
mod process_no_output;
use process_no_output::NoOutputWatchdog;

#[cfg(unix)]
const MAX_ACTIVE_PROCESS_GROUPS: usize = 4096;

#[cfg(unix)]
static ACTIVE_PROCESS_GROUPS: [AtomicI32; MAX_ACTIVE_PROCESS_GROUPS] =
    [const { AtomicI32::new(0) }; MAX_ACTIVE_PROCESS_GROUPS];

#[cfg(unix)]
static SIGNAL_HANDLERS_INSTALLED: std::sync::Once = std::sync::Once::new();

#[derive(Debug, Clone)]
pub struct ExecSpec {
    pub command: String,
    pub stdin: Option<String>,
    pub working_dir: std::path::PathBuf,
    pub timeout_sec: u64,
    pub no_output_timeout_sec: Option<u64>,
    pub no_output_progress_paths: Vec<std::path::PathBuf>,
    pub no_output_activity_patterns: Vec<String>,
    pub no_output_activity_event: Option<NoOutputActivityEvent>,
    pub env_clear: bool,
    pub env_vars: BTreeMap<String, String>,
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

        let spawned = spawn_child(spec);

        let mut spawned = match spawned {
            Ok(spawned) => spawned,
            Err(error) => {
                fs::write(&spec.stdout_path, "")?;
                fs::write(&spec.stderr_path, error.to_string())?;
                return Ok(record(spec, None, TerminationReason::SpawnError));
            }
        };
        let mut child = spawned.child;
        let process_group = match ActiveProcessGroup::register(&mut child) {
            Ok(process_group) => process_group,
            Err(error) => {
                fs::write(&spec.stdout_path, "")?;
                fs::write(&spec.stderr_path, error.to_string())?;
                return Ok(record(spec, None, TerminationReason::SpawnError));
            }
        };
        if let Err(error) = spawned.start_gate.release() {
            kill_process_tree(&mut child, &process_group);
            let _ = child.wait();
            fs::write(&spec.stdout_path, "")?;
            fs::write(&spec.stderr_path, error.to_string())?;
            return Ok(record(spec, None, TerminationReason::SpawnError));
        }

        if let Some(stdin) = &spec.stdin
            && let Some(mut pipe) = child.stdin.take()
            && let Err(error) = pipe.write_all(stdin.as_bytes())
            && error.kind() != std::io::ErrorKind::BrokenPipe
        {
            kill_process_tree(&mut child, &process_group);
            let _ = child.wait();
            return Err(error.into());
        }
        drop(child.stdin.take());

        let started = Instant::now();
        let last_output_ms = Arc::new(AtomicU64::new(0));
        let mut stdout_thread = Some(stream_child_output(
            child.stdout.take(),
            spec.stdout_path.clone(),
            Arc::clone(&last_output_ms),
            started,
        ));
        let mut stderr_thread = Some(stream_child_output(
            child.stderr.take(),
            spec.stderr_path.clone(),
            Arc::clone(&last_output_ms),
            started,
        ));

        let hard_timeout = Duration::from_secs(spec.timeout_sec.max(1));
        let no_output_timeout = spec
            .no_output_timeout_sec
            .filter(|timeout| *timeout > 0)
            .map(Duration::from_secs);
        let deadline = started + hard_timeout;
        let mut no_output_watchdog =
            NoOutputWatchdog::new(spec.no_output_progress_paths.clone(), started);
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
                kill_process_tree(&mut child, &process_group);
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
            if let Some(timeout) = no_output_timeout {
                let now = Instant::now();
                if let Some(path) = no_output_watchdog.changed_path() {
                    last_output_ms.store(elapsed_ms(started), Ordering::Relaxed);
                    no_output_watchdog.record_progress(
                        spec.no_output_activity_event.as_ref(),
                        path,
                        now,
                    );
                }
                let elapsed_since_start_ms = started.elapsed().as_millis();
                let last_output = u128::from(last_output_ms.load(Ordering::Relaxed));
                let quiet_for_ms = elapsed_since_start_ms.saturating_sub(last_output);
                if quiet_for_ms >= timeout.as_millis() {
                    if !no_output_watchdog.ready_to_probe(now) {
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    no_output_watchdog.mark_probe(now);
                    let activity = process_group_activity(
                        process_group.pgid(),
                        &spec.no_output_activity_patterns,
                    )
                    .unwrap_or(None);
                    if let Some(activity) = &activity
                        && no_output_watchdog.record_activity_or_expired(
                            spec.no_output_activity_event.as_ref(),
                            activity,
                            now,
                            timeout,
                        )
                    {
                        continue;
                    }
                    no_output_watchdog.emit_no_progress(
                        spec.no_output_activity_event.as_ref(),
                        timeout,
                        activity.as_ref(),
                    );
                    kill_process_tree(&mut child, &process_group);
                    child.wait().context("wait for no-progress process kill")?;
                    join_stream_timeout(
                        stdout_thread.take().expect("stdout stream thread"),
                        Duration::from_secs(2),
                    )?;
                    join_stream_timeout(
                        stderr_thread.take().expect("stderr stream thread"),
                        Duration::from_secs(2),
                    )?;
                    return Ok(record(spec, None, TerminationReason::NoProgress));
                } else {
                    no_output_watchdog.clear_activity_deferral();
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

struct SpawnedChild {
    child: Child,
    start_gate: ChildStartGate,
}

fn spawn_child(spec: &ExecSpec) -> std::io::Result<SpawnedChild> {
    install_shutdown_signal_handlers();
    let mut start_gate = ChildStartGate::new()?;
    let command_body = start_gate.wrap_command(&spec.command);
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(command_body)
        .current_dir(&spec.working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if spec.env_clear {
        command.env_clear();
    }
    command.envs(&spec.env_vars);
    configure_child_process_group(&mut command, start_gate.child_fds());
    let child = command.spawn()?;
    start_gate.close_child_end();
    Ok(SpawnedChild { child, start_gate })
}

#[cfg(unix)]
fn configure_child_process_group(command: &mut Command, start_gate: ChildStartGateFds) {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(move || {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            process_start_gate::prepare_child_after_setsid(start_gate);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn configure_child_process_group(_command: &mut Command, _start_gate: ChildStartGateFds) {}

#[cfg(unix)]
struct ActiveProcessGroup {
    pgid: i32,
    slot: usize,
}

#[cfg(unix)]
impl ActiveProcessGroup {
    fn register(child: &mut Child) -> Result<Self> {
        let pgid = child.id() as i32;
        let Some(slot) = reserve_process_group_slot(&ACTIVE_PROCESS_GROUPS, pgid) else {
            kill_process_group(pgid);
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!(
                "active process group registry is full; reduce HarnessLab run concurrency"
            );
        };
        Ok(Self { pgid, slot })
    }

    fn kill(&self) {
        kill_process_group(self.pgid);
    }

    fn pgid(&self) -> i32 {
        self.pgid
    }
}

#[cfg(unix)]
impl Drop for ActiveProcessGroup {
    fn drop(&mut self) {
        let _ = ACTIVE_PROCESS_GROUPS[self.slot].compare_exchange(
            self.pgid,
            0,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }
}

#[cfg(not(unix))]
struct ActiveProcessGroup;

#[cfg(not(unix))]
impl ActiveProcessGroup {
    fn register(_child: &mut Child) -> Result<Self> {
        Ok(Self)
    }

    fn kill(&self) {}

    fn pgid(&self) -> i32 {
        0
    }
}

fn kill_process_tree(child: &mut Child, process_group: &ActiveProcessGroup) {
    process_group.kill();
    let _ = child.kill();
}

#[cfg(unix)]
fn reserve_process_group_slot(groups: &[AtomicI32], pgid: i32) -> Option<usize> {
    groups.iter().position(|group| {
        group
            .compare_exchange(0, pgid, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    })
}

#[cfg(unix)]
fn kill_process_group(pgid: i32) {
    if pgid <= 0 {
        return;
    }
    unsafe {
        let _ = libc::kill(-pgid, libc::SIGKILL);
    }
}

#[cfg(unix)]
fn install_shutdown_signal_handlers() {
    SIGNAL_HANDLERS_INSTALLED.call_once(|| {
        install_signal_handler(libc::SIGINT);
        install_signal_handler(libc::SIGTERM);
    });
}

#[cfg(not(unix))]
fn install_shutdown_signal_handlers() {}

#[cfg(unix)]
fn install_signal_handler(signal: libc::c_int) {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = shutdown_signal_handler as *const () as usize;
        action.sa_flags = 0;
        libc::sigemptyset(&mut action.sa_mask);
        let _ = libc::sigaction(signal, &action, std::ptr::null_mut());
    }
}

#[cfg(unix)]
extern "C" fn shutdown_signal_handler(signal: libc::c_int) {
    for group in &ACTIVE_PROCESS_GROUPS {
        kill_process_group(group.load(Ordering::SeqCst));
    }
    unsafe {
        libc::_exit(128 + signal);
    }
}

fn stream_child_output<R>(
    pipe: Option<R>,
    path: std::path::PathBuf,
    last_output_ms: Arc<AtomicU64>,
    started: Instant,
) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut file = fs::File::create(path)?;
        if let Some(mut pipe) = pipe {
            let mut buffer = [0; 8192];
            loop {
                let read = pipe.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                file.write_all(&buffer[..read])?;
                last_output_ms.store(elapsed_ms(started), Ordering::Relaxed);
            }
        }
        Ok(())
    })
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
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
#[path = "process_tests.rs"]
mod tests;
