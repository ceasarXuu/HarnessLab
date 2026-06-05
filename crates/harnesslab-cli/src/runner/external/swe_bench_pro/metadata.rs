use super::runtime_snapshot;
use anyhow::{Context, Result, bail};
use harnesslab_infra::{ExecSpec, HostProcessExecutor};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct SweInstance {
    pub(super) instance_id: String,
    pub(super) repo: String,
    pub(super) base_commit: String,
    pub(super) dockerhub_tag: String,
    pub(super) parquet_path: PathBuf,
    pub(super) problem_statement: String,
    pub(super) requirements: String,
    pub(super) gold_patch: String,
}

pub(super) fn load_instance(
    dataset_path: &Path,
    task_id: &str,
    swe_dir: &Path,
) -> Result<SweInstance> {
    let raw_sample = swe_dir.join("raw_sample.jsonl");
    let instance_json = swe_dir.join("instance.json");
    let script_path = swe_dir.join("extract_instance.py");
    fs::write(&script_path, extract_script())?;
    let parquet = first_parquet(dataset_path).context("swe-bench-pro parquet data is missing")?;
    let process = HostProcessExecutor::exec(&ExecSpec {
        command: runtime_snapshot::metadata_extract_command(
            &script_path,
            &parquet,
            task_id,
            &raw_sample,
            &instance_json,
        ),
        stdin: None,
        working_dir: swe_dir.to_path_buf(),
        timeout_sec: 300,
        no_output_timeout_sec: None,
        no_output_progress_paths: Vec::new(),
        no_output_activity_patterns: Vec::new(),
        no_output_activity_event: None,
        env_clear: false,
        env_vars: std::collections::BTreeMap::new(),
        stdout_path: swe_dir.join("metadata.stdout.log"),
        stderr_path: swe_dir.join("metadata.stderr.log"),
    })?;
    if process.exit_code != Some(0) {
        bail!("failed to extract SWE-bench Pro instance metadata");
    }
    let value: Value = serde_json::from_slice(&fs::read(instance_json)?)?;
    Ok(SweInstance {
        instance_id: json_string(&value, "instance_id")?,
        repo: json_string(&value, "repo")?,
        base_commit: json_string(&value, "base_commit")?,
        dockerhub_tag: json_string(&value, "dockerhub_tag")?,
        parquet_path: parquet,
        problem_statement: json_string(&value, "problem_statement")?,
        requirements: json_string(&value, "requirements")?,
        gold_patch: json_string(&value, "patch")?,
    })
}

pub(super) fn first_parquet(dataset_path: &Path) -> Option<PathBuf> {
    let mut files = fs::read_dir(dataset_path.join("data"))
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "parquet"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    files.into_iter().next()
}

fn extract_script() -> &'static str {
    r#"
import json
import pandas as pd
import sys

parquet, instance_id, raw_sample_path, instance_json_path = sys.argv[1:5]
df = pd.read_parquet(parquet)
matches = df[df["instance_id"] == instance_id]
if matches.empty:
    raise SystemExit(f"instance_id not found: {instance_id}")
row = matches.iloc[0].where(pd.notna(matches.iloc[0]), "")
record = row.to_dict()
with open(raw_sample_path, "w") as f:
    f.write(json.dumps(record) + "\n")
keys = ["instance_id", "repo", "base_commit", "dockerhub_tag", "problem_statement", "requirements", "patch"]
with open(instance_json_path, "w") as f:
    json.dump({key: str(record.get(key, "")) for key in keys}, f)
"#
}

fn json_string(value: &Value, key: &str) -> Result<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .with_context(|| format!("missing string field {key}"))
}
