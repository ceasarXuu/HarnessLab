use crate::ExternalRunnerKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePreflightReport {
    pub task_id: String,
    pub runner_kind: ExternalRunnerKind,
    pub adapter_id: String,
    pub agent_bridge_mode: String,
    pub readiness_status: String,
    pub host_execution_reason: Option<String>,
    pub blocking_reason: Option<String>,
    pub compatibility_exception: Option<String>,
    pub compatibility_label_keys: Vec<String>,
}
