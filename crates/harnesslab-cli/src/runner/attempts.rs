use super::execute_task;
use anyhow::Result;
use harnesslab_core::{AgentProfile, RunSpec, TaskAttemptResult};
use std::any::Any;
use std::path::Path;
use std::thread;

use super::schedule::AttemptWork;

pub(super) fn execute_attempts(
    run_dir: &Path,
    spec: &RunSpec,
    profile: &AgentProfile,
    report_profile: &AgentProfile,
    attempts: Vec<AttemptWork>,
    concurrency: usize,
) -> Result<Vec<TaskAttemptResult>> {
    let mut results = Vec::new();
    for chunk in attempts.chunks(concurrency.max(1)) {
        let mut handles = Vec::new();
        for work in chunk.iter().cloned() {
            let run_dir = run_dir.to_path_buf();
            let profile = profile.clone();
            let report_profile = report_profile.clone();
            let spec = spec.clone();
            handles.push(thread::spawn(move || {
                execute_task(
                    &run_dir,
                    &spec,
                    &profile,
                    &report_profile,
                    &work.task,
                    work.attempt,
                    work.provenance,
                )
            }));
        }
        let mut first_error = None;
        for handle in handles {
            match handle.join() {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(error)) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
                Err(panic) => {
                    if first_error.is_none() {
                        first_error =
                            Some(anyhow::anyhow!("task panicked: {}", panic_message(panic)));
                    }
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
    }
    Ok(results)
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
