use crate::{NoOutputActivityEvent, ProcessActivity, ProgressWatcher, append_event, event};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ACTIVITY_REPROBE: Duration = Duration::from_secs(1);
const ACTIVITY_EVENT_INTERVAL: Duration = Duration::from_secs(30);

pub(super) struct NoOutputWatchdog {
    progress_watcher: ProgressWatcher,
    next_probe: Instant,
    last_event_at: Option<Instant>,
    activity_deferral_started_at: Option<Instant>,
    activity_event_emitted: bool,
    last_activity_detail: Option<String>,
    last_progress_detail: Option<String>,
}

impl NoOutputWatchdog {
    pub(super) fn new(progress_paths: Vec<PathBuf>, started: Instant) -> Self {
        Self {
            progress_watcher: ProgressWatcher::new(progress_paths),
            next_probe: started,
            last_event_at: None,
            activity_deferral_started_at: None,
            activity_event_emitted: false,
            last_activity_detail: None,
            last_progress_detail: None,
        }
    }

    pub(super) fn ready_to_probe(&self, now: Instant) -> bool {
        now >= self.next_probe
    }

    pub(super) fn mark_probe(&mut self, now: Instant) {
        self.next_probe = now + ACTIVITY_REPROBE;
    }

    pub(super) fn changed_path(&mut self) -> Option<PathBuf> {
        self.progress_watcher.changed_path()
    }

    pub(super) fn record_progress(
        &mut self,
        config: Option<&NoOutputActivityEvent>,
        path: PathBuf,
        now: Instant,
    ) {
        self.clear_activity_deferral();
        self.last_progress_detail = Some(format!("path={}", path.display()));
        self.append_deferral_event(
            config,
            format!(
                "no-output watchdog deferred by progress file path={}",
                path.display()
            ),
            now,
            false,
        );
    }

    pub(super) fn record_activity_or_expired(
        &mut self,
        config: Option<&NoOutputActivityEvent>,
        activity: &ProcessActivity,
        now: Instant,
        timeout: Duration,
    ) -> bool {
        let detail = activity_detail(activity);
        self.last_activity_detail = Some(detail.clone());
        let deferral_started = *self.activity_deferral_started_at.get_or_insert(now);
        if now.duration_since(deferral_started) >= timeout {
            if !self.activity_event_emitted {
                self.activity_event_emitted = self.append_deferral_event(
                    config,
                    format!("no-output watchdog deferred by active process {detail}"),
                    now,
                    true,
                );
            }
            return false;
        }
        if self.append_deferral_event(
            config,
            format!("no-output watchdog deferred by active process {detail}"),
            now,
            false,
        ) {
            self.activity_event_emitted = true;
        }
        true
    }

    pub(super) fn clear_activity_deferral(&mut self) {
        self.activity_deferral_started_at = None;
        self.activity_event_emitted = false;
    }

    pub(super) fn emit_no_progress(
        &self,
        config: Option<&NoOutputActivityEvent>,
        timeout: Duration,
        current_activity: Option<&ProcessActivity>,
    ) {
        let Some(config) = config else {
            return;
        };
        let Some(event_name) = config.no_progress_event_name.as_deref() else {
            return;
        };
        let current_activity_detail = current_activity
            .map(activity_detail)
            .unwrap_or_else(|| "none".to_string());
        let message = format!(
            "no-output watchdog killed process tree: no durable stdout/stderr or progress file growth within {}s; activity_grace_exhausted={}; current_activity={}; last_activity={}; last_progress={}",
            timeout.as_secs(),
            current_activity.is_some(),
            current_activity_detail,
            self.last_activity_detail.as_deref().unwrap_or("none"),
            self.last_progress_detail.as_deref().unwrap_or("none"),
        );
        let _ = append_event(
            &config.path,
            &event(
                &config.run_id,
                config.task_id.as_deref(),
                event_name,
                &message,
            ),
            &[],
        );
    }

    fn append_deferral_event(
        &mut self,
        config: Option<&NoOutputActivityEvent>,
        message: String,
        now: Instant,
        force: bool,
    ) -> bool {
        let Some(config) = config else {
            return false;
        };
        let should_emit = force
            || self
                .last_event_at
                .map(|last| now.duration_since(last) >= ACTIVITY_EVENT_INTERVAL)
                .unwrap_or(true);
        if !should_emit {
            return false;
        }
        let appended = append_event(
            &config.path,
            &event(
                &config.run_id,
                config.task_id.as_deref(),
                &config.event_name,
                &message,
            ),
            &[],
        )
        .is_ok();
        if appended {
            self.last_event_at = Some(now);
        }
        appended
    }
}

fn activity_detail(activity: &ProcessActivity) -> String {
    format!(
        "pid={} command={} pattern={}",
        activity.pid, activity.command_name, activity.pattern
    )
}
