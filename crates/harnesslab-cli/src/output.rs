use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct InitOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub home: String,
    pub detected_agents: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct ListOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub items: Vec<String>,
}

#[derive(Serialize)]
pub(crate) struct AgentSchemaOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub fields: Vec<harnesslab_core::AgentProfileFieldReference>,
}

#[derive(Serialize)]
pub(crate) struct DoctorOutput {
    pub schema_version: u32,
    pub status: &'static str,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Serialize)]
pub(crate) struct DoctorCheck {
    pub id: String,
    pub status: String,
    pub severity: String,
    pub message: String,
    pub details: serde_json::Value,
}

#[derive(Serialize)]
pub(crate) struct BenchmarkListOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub benchmarks: Vec<harnesslab_core::BenchmarkDescriptor>,
}

#[derive(Serialize)]
pub(crate) struct BenchmarkInfoOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub benchmark: harnesslab_core::BenchmarkDescriptor,
}

#[derive(Serialize)]
pub(crate) struct RunOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub exit_code: i32,
    pub verdict: &'static str,
    pub run_id: String,
    pub run_dir: String,
    pub results_path: String,
    pub report_path: String,
    pub summary: harnesslab_core::RunSummary,
    pub replay_source_run_id: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PathOutput {
    pub schema_version: u32,
    pub command: &'static str,
    pub status: &'static str,
    pub run_dir: String,
}
