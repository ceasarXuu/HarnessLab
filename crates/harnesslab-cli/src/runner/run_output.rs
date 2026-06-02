use crate::output::RunOutput;
use crate::print_json;
use anyhow::{Context, Result};
use harnesslab_core::RunResults;
use std::path::Path;

pub(super) fn emit_run_output(
    json: bool,
    code: i32,
    run_id: String,
    run_dir: &Path,
    replay_source_run_id: Option<String>,
) -> Result<()> {
    let report_path = run_dir.join("report.html").display().to_string();
    let results_path = run_dir.join("results.json");
    if json {
        let results: RunResults = serde_json::from_slice(
            &std::fs::read(&results_path)
                .with_context(|| format!("failed to read {}", results_path.display()))?,
        )?;
        print_json(&RunOutput {
            schema_version: 1,
            command: "run",
            status: if code == 0 { "success" } else { "failure" },
            exit_code: code,
            verdict: verdict(code, &results),
            run_id,
            run_dir: run_dir.display().to_string(),
            results_path: results_path.display().to_string(),
            report_path,
            summary: results.summary,
            replay_source_run_id,
        })
    } else {
        println!("run: {}", run_dir.display());
        println!("report: {report_path}");
        Ok(())
    }
}

fn verdict(code: i32, results: &RunResults) -> &'static str {
    if code == 130 || results.summary.interrupted > 0 {
        "interrupted"
    } else if code == 3 {
        "run_failed"
    } else if results.summary.execution_failure > 0 {
        "execution_failure"
    } else if results.summary.benchmark_failure > 0 {
        "benchmark_failure"
    } else if results.summary.partial_success > 0 {
        "partial_success"
    } else {
        "success"
    }
}
