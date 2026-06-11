use anyhow::{Context, Result, bail};
use std::fs;

const GENERIC_BEHAVIOR_FILES: &[&str] = &[
    "crates/harnesslab-cli/src/runner.rs",
    "crates/harnesslab-cli/src/doctor_run_as.rs",
];

const FORBIDDEN_IMPORT_PATTERNS: &[&str] = &[
    "terminal_bench",
    "swe_bench_pro",
    "scaffold_golden_adapter",
];

pub fn verify_forbidden_diff() -> Result<()> {
    let mut violations = Vec::new();

    for &path in GENERIC_BEHAVIOR_FILES {
        let content = fs::read_to_string(path)
            .with_context(|| format!("read {}", path))?;

        for pattern in FORBIDDEN_IMPORT_PATTERNS {
            if content.contains(pattern) {
                violations.push(format!(
                    "{} contains adapter-specific reference '{}'",
                    path, pattern
                ));
            }
        }
    }

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
