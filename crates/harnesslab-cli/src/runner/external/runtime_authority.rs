use anyhow::{Result, bail};
use harnesslab_core::{ExternalRunnerKind, TaskPlan};

pub(in crate::runner) fn runtime_dataset_ref(task: &TaskPlan) -> Result<&str> {
    if let Some(binding) = &task.runtime_binding {
        if let Some(runner) = &task.external_runner
            && runner.dataset_path != binding.dataset_ref
        {
            bail!(
                "protocol runtime binding dataset_ref mismatch for task {}: legacy={} protocol={}",
                task.task_id,
                runner.dataset_path,
                binding.dataset_ref
            );
        }
        return Ok(&binding.dataset_ref);
    }
    if let Some(runner) = &task.external_runner {
        return Ok(&runner.dataset_path);
    }
    bail!("external task missing dataset authority")
}

pub(in crate::runner) fn runtime_source_ref(task: &TaskPlan) -> Result<Option<&str>> {
    if let Some(binding) = &task.runtime_binding {
        if let Some(runner) = &task.external_runner {
            let Some(source_path) = runner.source_path.as_deref() else {
                bail!(
                    "protocol runtime binding task_ref mismatch for task {}: legacy=none protocol={}",
                    task.task_id,
                    binding.task_ref
                );
            };
            if source_path != binding.task_ref {
                bail!(
                    "protocol runtime binding task_ref mismatch for task {}: legacy={} protocol={}",
                    task.task_id,
                    source_path,
                    binding.task_ref
                );
            }
        }
        return Ok(Some(binding.task_ref.as_str()));
    }
    Ok(task
        .external_runner
        .as_ref()
        .and_then(|runner| runner.source_path.as_deref()))
}

pub(in crate::runner) fn runtime_snapshot_source_ref(
    task: &TaskPlan,
    kind: ExternalRunnerKind,
) -> Result<Option<&str>> {
    match kind {
        ExternalRunnerKind::TerminalBench => Ok(task
            .external_runner
            .as_ref()
            .and_then(|runner| runner.source_path.as_deref())),
        ExternalRunnerKind::SweBenchPro => runtime_source_ref(task),
    }
}
