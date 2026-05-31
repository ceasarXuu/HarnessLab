use super::execute_task;
use anyhow::Result;
use harnesslab_core::{AgentProfile, RunSpec, TaskAttemptResult};
use std::any::Any;
use std::path::Path;
use std::thread;

use super::monitor::RunMonitor;
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
    let mut monitor = RunMonitor::new(&spec.run_id, attempts.len());
    let chunk_size = concurrency.max(1);
    let mut cursor = 0;
    while cursor < attempts.len() {
        let end = (cursor + chunk_size).min(attempts.len());
        let chunk = &attempts[cursor..end];
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
        let mut abort_after_chunk = false;
        for handle in handles {
            match handle.join() {
                Ok(Ok(result)) => {
                    let abort = monitor.record_result(run_dir, &result)?;
                    results.push(result);
                    if abort.is_some() {
                        abort_after_chunk = true;
                    }
                }
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
        if abort_after_chunk {
            let pending = &attempts[end..];
            results.extend(monitor.interrupted_results(run_dir, pending)?);
            return Ok(results);
        }
        cursor = end;
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
