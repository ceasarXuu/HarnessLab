use anyhow::{Context, Result, bail};
use std::fs;

const GENERIC_BEHAVIOR_FILES: &[&str] = &[
    "crates/harnesslab-cli/src/app.rs",
    "crates/harnesslab-cli/src/benchmark_data.rs",
    "crates/harnesslab-cli/src/doctor.rs",
    "crates/harnesslab-cli/src/doctor_capabilities.rs",
    "crates/harnesslab-cli/src/doctor_run_as.rs",
    "crates/harnesslab-cli/src/doctor_setup.rs",
    "crates/harnesslab-cli/src/main.rs",
    "crates/harnesslab-cli/src/output.rs",
    "crates/harnesslab-cli/src/runner.rs",
];

const FORBIDDEN_IMPORT_PATTERNS: &[&str] = &[
    "terminal_bench",
    "swe_bench_pro",
    "scaffold_golden_adapter",
];

pub fn verify_forbidden_diff() -> Result<()> {
    let violations = check_files(GENERIC_BEHAVIOR_FILES, FORBIDDEN_IMPORT_PATTERNS)?;

    if !violations.is_empty() {
        for v in &violations {
            eprintln!("forbidden-diff violation: {}", v);
        }
        bail!(
            "forbidden-diff guard failed with {} violation(s): generic behavior files must not contain adapter-specific module references",
            violations.len()
        );
    }

    println!("forbidden-diff guard passed: {} generic behavior files are adapter-free", GENERIC_BEHAVIOR_FILES.len());
    Ok(())
}

fn check_files(files: &[&str], patterns: &[&str]) -> Result<Vec<String>> {
    let mut violations = Vec::new();

    for &path in files {
        let content = fs::read_to_string(path)
            .with_context(|| format!("read {}", path))?;

        for pattern in patterns {
            if content.contains(pattern) {
                violations.push(format!(
                    "{} contains adapter-specific reference '{}'",
                    path, pattern
                ));
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn forbidden_diff_detects_adapter_reference_in_generic_file() {
        let dir = TempDir::new().unwrap();
        let bad_file = dir.path().join("generic.rs");
        let mut f = std::fs::File::create(&bad_file).unwrap();
        write!(f, "use crate::terminal_bench;").unwrap();
        drop(f);

        let path = bad_file.to_str().unwrap();
        let violations = check_files(&[path], FORBIDDEN_IMPORT_PATTERNS).unwrap();
        assert!(
            violations.iter().any(|v| v.contains("terminal_bench")),
            "expected violation for terminal_bench reference, got: {:?}",
            violations
        );
    }

    #[test]
    fn forbidden_diff_passes_when_no_adapter_reference() {
        let dir = TempDir::new().unwrap();
        let good_file = dir.path().join("generic.rs");
        let mut f = std::fs::File::create(&good_file).unwrap();
        write!(f, "fn generic_behavior() {{}}").unwrap();
        drop(f);

        let path = good_file.to_str().unwrap();
        let violations = check_files(&[path], FORBIDDEN_IMPORT_PATTERNS).unwrap();
        assert!(violations.is_empty(), "expected no violations, got: {:?}", violations);
    }
}
