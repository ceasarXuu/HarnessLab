use crate::ExternalRunnerKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePreflightReport {
    pub task_id: String,
    pub runner_kind: ExternalRunnerKind,
    pub host_execution_reason: Option<String>,
}
