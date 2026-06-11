use crate::ExternalRunnerKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePreflightReport {
    pub task_id: String,
    pub runner_kind: ExternalRunnerKind,
    pub adapter_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_adapter_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_benchmark_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_selected_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_stability: Option<String>,
    #[serde(default)]
    pub protocol_capabilities: Vec<String>,
    #[serde(default)]
    pub legacy_shim_used: bool,
    pub agent_bridge_mode: String,
    pub readiness_status: String,
    pub host_execution_reason: Option<String>,
    pub blocking_reason: Option<String>,
    pub compatibility_exception: Option<String>,
    pub compatibility_label_keys: Vec<String>,
}
