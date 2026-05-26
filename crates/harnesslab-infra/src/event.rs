use anyhow::Result;
use harnesslab_core::redact_known_secret;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
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
    writeln!(file, "{}", serde_json::to_string(&redacted)?)?;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
