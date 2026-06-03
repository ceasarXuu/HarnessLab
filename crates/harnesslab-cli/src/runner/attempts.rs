use super::execute_task;
use crate::agent_registry::MaterializedAgentProfile;
use anyhow::Result;
use harnesslab_core::{AgentProfile, RunSpec, TaskAttemptResult};
use std::any::Any;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::thread;

use super::monitor::RunMonitor;
use super::schedule::AttemptWork;

#[derive(Clone)]
pub(super) struct TaskExecutionContext {
    pub(super) run_dir: std::path::PathBuf,
    pub(super) spec: RunSpec,
    pub(super) profile: AgentProfile,
    pub(super) report_profile: AgentProfile,
    pub(super) materialized_profile: MaterializedAgentProfile,
}

pub(super) fn execute_attempts(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    report_profile: &AgentProfile,
    materialized_profile: &MaterializedAgentProfile,
    attempts: Vec<AttemptWork>,
    concurrency: usize,
) -> Result<Vec<TaskAttemptResult>> {
    let context = TaskExecutionContext {
        run_dir: run_dir.to_path_buf(),
        spec: spec.clone(),
        profile: profile.clone(),
        report_profile: report_profile.clone(),
        materialized_profile: materialized_profile.clone(),
    };
    let run_id = spec.run_id.clone();
    let executor = Arc::new(move |work: AttemptWork| execute_task(&context, work));
    execute_attempts_with(run_dir, &run_id, attempts, concurrency, executor)
}

pub(super) fn execute_attempts_with(
    run_dir: &Path,
    run_id: &str,
    attempts: Vec<AttemptWork>,
    concurrency: usize,
    executor: Arc<dyn Fn(AttemptWork) -> Result<TaskAttemptResult> + Send + Sync + 'static>,
) -> Result<Vec<TaskAttemptResult>> {
    let mut results = Vec::new();
    let mut monitor = RunMonitor::new(run_id, attempts.len());
    let mut pending = VecDeque::from(attempts);
    let (sender, receiver) = mpsc::channel();
    let mut active = 0usize;
    let mut first_error = None;
    let mut aborting = false;

    while active < concurrency.max(1) {
        if !spawn_next(&mut pending, &sender, &executor, &mut active) {
            break;
        }
    }

    while active > 0 {
        match receiver.recv() {
            Ok(AttemptMessage::Result(result)) => {
                active -= 1;
                match *result {
                    Ok(result) => {
                        let abort = monitor.record_result(run_dir, &result)?;
                        results.push(result);
                        if abort.is_some() {
                            aborting = true;
                        }
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                            aborting = true;
                        }
                    }
                }
            }
            Ok(AttemptMessage::Panic(message)) => {
                active -= 1;
                if first_error.is_none() {
                    first_error = Some(anyhow::anyhow!("task panicked: {message}"));
                    aborting = true;
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(anyhow::anyhow!("task worker channel closed: {error}"));
                }
                break;
            }
        }
        while !aborting && active < concurrency.max(1) {
            if !spawn_next(&mut pending, &sender, &executor, &mut active) {
                break;
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }
    if aborting {
        let pending = pending.into_iter().collect::<Vec<_>>();
        results.extend(monitor.interrupted_results(run_dir, &pending)?);
    }
    Ok(results)
}

enum AttemptMessage {
    Result(Box<Result<TaskAttemptResult>>),
    Panic(String),
}

fn spawn_next(
    pending: &mut VecDeque<AttemptWork>,
    sender: &mpsc::Sender<AttemptMessage>,
    executor: &Arc<dyn Fn(AttemptWork) -> Result<TaskAttemptResult> + Send + Sync + 'static>,
    active: &mut usize,
) -> bool {
    let Some(work) = pending.pop_front() else {
        return false;
    };
    let sender = sender.clone();
    let executor = Arc::clone(executor);
    thread::spawn(move || {
        let message =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| executor(work))) {
                Ok(result) => AttemptMessage::Result(Box::new(result)),
                Err(panic) => AttemptMessage::Panic(panic_message(panic)),
            };
        let _ = sender.send(message);
    });
    *active += 1;
    true
}

pub(super) fn panic_message(panic: Box<dyn Any + Send + 'static>) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = panic.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_string()
    }
}

#[cfg(test)]
#[path = "attempts_tests.rs"]
mod tests;
