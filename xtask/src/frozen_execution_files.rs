use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const REQUIRED_EXECUTION_FILE_PATHS: &[&str] = &[
    "scripts/test-after-change.sh",
    "scripts/verify-terminal-bench-python-adapter.sh",
    "scripts/verify-terminal-bench-registered-setup.sh",
    "scripts/verify-terminal-bench-docker-activity-watchdog.sh",
    "scripts/verify-terminal-bench-docker-activity-grace-expiry.sh",
    "scripts/verify-terminal-bench-import-success-cleanup.sh",
    "scripts/verify-terminal-bench-import-timeout-cleanup.sh",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct FrozenExecutionFile {
    pub(crate) path: String,
    pub(crate) hash: String,
}

pub(crate) fn current_execution_files() -> Result<Vec<FrozenExecutionFile>> {
    let mut files = Vec::new();
    for path in REQUIRED_EXECUTION_FILE_PATHS {
        let content = read_repo_file(path)?;
        files.push(FrozenExecutionFile {
            path: path.to_string(),
            hash: stable_content_hash(&content),
        });
    }
    Ok(files)
}

pub(crate) fn verify_execution_files(files: &[FrozenExecutionFile]) -> Result<usize> {
    let expected: BTreeSet<&str> = REQUIRED_EXECUTION_FILE_PATHS.iter().copied().collect();
    let actual: BTreeSet<&str> = files.iter().map(|file| file.path.as_str()).collect();
    if actual != expected {
        for path in expected.difference(&actual) {
            bail!("required frozen execution file missing: {path}");
        }
        for path in actual.difference(&expected) {
            bail!("unexpected frozen execution file: {path}");
        }
    }
    for file in files {
        let content = read_repo_file(&file.path)?;
        verify_execution_file_hash(&file.path, &file.hash, &content)?;
    }
    Ok(files.len())
}

fn read_repo_file(path: &str) -> Result<Vec<u8>> {
    fs::read(repo_path(path)).with_context(|| format!("read {path}"))
}

fn repo_path(path: &str) -> PathBuf {
    let direct = PathBuf::from(path);
    if direct.exists() {
        return direct;
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask should live under workspace root")
        .join(path)
}

fn verify_execution_file_hash(path: &str, expected_hash: &str, content: &[u8]) -> Result<()> {
    let actual_hash = stable_content_hash(content);
    if expected_hash != actual_hash {
        bail!("frozen execution file hash changed: {path}");
    }
    Ok(())
}

fn stable_content_hash(content: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_file_hash_rejects_noop_replacement() {
        let original = b"exec scripts/verify-terminal-bench-python-adapter.sh\n";
        let replacement = b"true\n";
        let expected_hash = stable_content_hash(original);
        let error = verify_execution_file_hash(
            "scripts/verify-terminal-bench-python-adapter.sh",
            &expected_hash,
            replacement,
        )
        .expect_err("no-op body replacement should fail");
        assert!(error.to_string().contains("hash changed"));
    }
}
