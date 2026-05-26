use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactManifest {
    pub schema_version: u32,
    pub files: Vec<ArtifactEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub source: String,
    pub destination: String,
    pub artifact_type: String,
    pub status: String,
    pub size: u64,
    pub error: Option<String>,
}

pub fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&content).with_context(|| format!("parse {}", path.display()))
}

pub fn collect_artifacts(
    base: &Path,
    destination: &Path,
    required: &[String],
) -> Result<ArtifactManifest> {
    fs::create_dir_all(destination)?;
    let mut files = Vec::new();
    for required_path in required {
        if !base.join(required_path).exists() {
            bail!("required artifact missing: {required_path}");
        }
    }
    if base.exists() {
        for entry in WalkDir::new(base).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry.path().strip_prefix(base)?.to_path_buf();
            let dest = destination.join(&relative);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
            let size = fs::metadata(&dest)?.len();
            files.push(ArtifactEntry {
                source: relative.display().to_string(),
                destination: dest.display().to_string(),
                artifact_type: "file".to_string(),
                status: "collected".to_string(),
                size,
                error: None,
            });
        }
    }
    Ok(ArtifactManifest {
        schema_version: 1,
        files,
    })
}

pub fn latest_run_dir(runs_dir: &Path) -> Result<Option<PathBuf>> {
    if !runs_dir.exists() {
        return Ok(None);
    }
    let mut entries = fs::read_dir(runs_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries.pop().map(|entry| entry.path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{Outcome, TaskAttemptResult, TaskState, UsageRecord};

    #[test]
    fn art_003_atomic_json_write_produces_valid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("result.json");
        let value = TaskAttemptResult {
            schema_version: 1,
            task_id: "t".to_string(),
            attempt: 1,
            state: TaskState::Success,
            outcome: Outcome::Success,
            failure_class: harnesslab_core::FailureClass::None,
            failure_code: None,
            benchmark_score: 1.0,
            duration_ms: 1,
            agent: None,
            evaluation: None,
            patch: None,
            usage: UsageRecord::Unknown,
            warnings: Vec::new(),
        };

        atomic_write_json(&path, &value).unwrap();
        let restored: TaskAttemptResult = read_json(&path).unwrap();

        assert_eq!(restored.task_id, "t");
    }

    #[test]
    fn art_001_collect_artifacts_copies_files_and_checks_required() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().join("base");
        let dest = tmp.path().join("dest");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("out.txt"), "ok").unwrap();

        let manifest = collect_artifacts(&base, &dest, &["out.txt".to_string()]).unwrap();

        assert_eq!(manifest.files.len(), 1);
        assert!(dest.join("out.txt").exists());
        assert!(collect_artifacts(&base, &dest, &["missing".to_string()]).is_err());
    }

    #[test]
    fn art_002_latest_run_dir_returns_none_for_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();

        assert!(latest_run_dir(&tmp.path().join("runs")).unwrap().is_none());
    }
}
