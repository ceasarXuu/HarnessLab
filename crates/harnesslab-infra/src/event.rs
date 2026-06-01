use anyhow::{Context, Result, bail};
use harnesslab_core::redact_known_secret;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub schema_version: u32,
    pub timestamp: String,
    pub run_id: String,
    pub task_id: Option<String>,
    pub event: String,
    pub message: String,
}

pub fn append_event(path: &Path, event: &Event, secrets: &[&str]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut redacted = event.clone();
    redacted.message = redact_known_secret(&redacted.message, secrets);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let _guard = FileLockGuard::lock(&file)?;
    let line = serde_json::to_string(&redacted)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

pub fn event(run_id: &str, task_id: Option<&str>, name: &str, message: &str) -> Event {
    Event {
        schema_version: 1,
        timestamp: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
        run_id: run_id.to_string(),
        task_id: task_id.map(str::to_string),
        event: name.to_string(),
        message: message.to_string(),
    }
}

pub fn validate_event_log(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("event log integrity check failed: {}", path.display()))?;
    if content.is_empty() {
        bail!(
            "event log integrity check failed: {} is empty",
            path.display()
        );
    }
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            bail!(
                "event log integrity check failed: {}:{} is blank",
                path.display(),
                line_number
            );
        }
        serde_json::from_str::<Event>(line).with_context(|| {
            format!(
                "event log integrity check failed: {}:{}",
                path.display(),
                line_number
            )
        })?;
    }
    Ok(())
}

#[cfg(unix)]
struct FileLockGuard {
    fd: RawFd,
}

#[cfg(unix)]
impl FileLockGuard {
    fn lock(file: &fs::File) -> Result<Self> {
        let fd = file.as_raw_fd();
        let result = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self { fd })
    }
}

#[cfg(unix)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.fd, libc::LOCK_UN) };
    }
}

#[cfg(not(unix))]
struct FileLockGuard;

#[cfg(not(unix))]
impl FileLockGuard {
    fn lock(_file: &fs::File) -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::process::Command;

    #[test]
    fn log_003_events_are_redacted() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("events.jsonl");
        append_event(
            &path,
            &event("run", Some("task"), "agent_started", "token sk-secret"),
            &["sk-secret"],
        )
        .unwrap();

        let content = std::fs::read_to_string(path).unwrap();

        assert!(content.contains("[REDACTED]"));
        assert!(!content.contains("sk-secret"));
    }

    #[test]
    fn log_006_event_log_integrity_rejects_malformed_line() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("events.jsonl");
        append_event(
            &path,
            &event("run", None, "run_started", "run started"),
            &[],
        )
        .unwrap();
        fs::write(
            &path,
            format!("{}{{bad}}\n", fs::read_to_string(&path).unwrap()),
        )
        .unwrap();

        let error = validate_event_log(&path).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("event log integrity check failed")
        );
        assert!(error.to_string().contains("events.jsonl:2"));
    }

    #[test]
    fn log_005_concurrent_process_appends_preserve_jsonl() {
        if std::env::var("HARNESSLAB_EVENT_APPEND_HELPER").as_deref() == Ok("1") {
            return;
        }
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("events.jsonl");
        let workers = 8;
        let per_worker = 40;

        let mut children = Vec::new();
        for worker in 0..workers {
            children.push(
                Command::new(std::env::current_exe().unwrap())
                    .arg("--exact")
                    .arg("event::tests::log_005_event_append_helper")
                    .env("HARNESSLAB_EVENT_APPEND_HELPER", "1")
                    .env("HARNESSLAB_EVENT_APPEND_PATH", &path)
                    .env("HARNESSLAB_EVENT_APPEND_WORKER", worker.to_string())
                    .env("HARNESSLAB_EVENT_APPEND_COUNT", per_worker.to_string())
                    .spawn()
                    .unwrap(),
            );
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }

        let content = fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), workers * per_worker);
        let mut seen = BTreeSet::new();
        for line in lines {
            let parsed: Event = serde_json::from_str(line).unwrap();
            assert_eq!(parsed.event, "concurrent_append");
            seen.insert((parsed.task_id.unwrap(), parsed.message));
        }

        for worker in 0..workers {
            for index in 0..per_worker {
                assert!(
                    seen.contains(&(worker.to_string(), format!("worker={worker} index={index}")))
                );
            }
        }
    }

    #[test]
    fn log_005_event_append_helper() {
        if std::env::var("HARNESSLAB_EVENT_APPEND_HELPER").as_deref() != Ok("1") {
            return;
        }
        let path = std::path::PathBuf::from(std::env::var("HARNESSLAB_EVENT_APPEND_PATH").unwrap());
        let worker = std::env::var("HARNESSLAB_EVENT_APPEND_WORKER").unwrap();
        let count: usize = std::env::var("HARNESSLAB_EVENT_APPEND_COUNT")
            .unwrap()
            .parse()
            .unwrap();

        for index in 0..count {
            append_event(
                &path,
                &event(
                    "run",
                    Some(&worker),
                    "concurrent_append",
                    &format!("worker={worker} index={index}"),
                ),
                &[],
            )
            .unwrap();
        }
    }

    #[test]
    fn meta_001_selected_failure_outputs_assertion_context() {
        if std::env::var("HARNESSLAB_META_001_FORCE_FAILURE").as_deref() != Ok("1") {
            return;
        }

        panic!("HARNESSLAB_META_001_FAILURE_SENTINEL");
    }
}
