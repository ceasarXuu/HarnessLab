use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const FORBIDDEN_TOKENS: &[&str] = &[
    "use harnesslab_adapters::TerminalBenchAdapter",
    "use harnesslab_adapters::SweBenchProAdapter",
    "harnesslab_adapters::TerminalBenchAdapter",
    "harnesslab_adapters::SweBenchProAdapter",
    "ExternalRunnerKind",
    "TerminalBench",
    "SweBenchPro",
    "terminal-bench",
    "swe-bench-pro",
    "terminal_bench",
    "swe_bench",
    "TERMINAL_BENCH",
    "SWE_BENCH",
    "benchmark_id.as_str()",
    "adapter_id.as_str()",
    "match benchmark_id",
    "match adapter_id",
    "benchmark_id ==",
    "adapter_id ==",
    "== benchmark_id",
    "== adapter_id",
    "benchmark_id.eq(",
    "adapter_id.eq(",
    ".eq(benchmark_id)",
    ".eq(adapter_id)",
    "benchmark_id !=",
    "adapter_id !=",
    "!= benchmark_id",
    "!= adapter_id",
    ".ne(benchmark_id)",
    ".ne(adapter_id)",
    "benchmark_id.ne(",
    "adapter_id.ne(",
];

const ADAPTER_OWNED_PREFIXES: &[&str] = &[
    "crates/harnesslab-adapters/src/",
    "crates/harnesslab-cli/src/runner/external/terminal_bench",
    "crates/harnesslab-cli/src/runner/external/swe_bench_pro",
];

const ADAPTER_OWNED_FILES: &[&str] = &[
    "crates/harnesslab-cli/src/runner/external/log_scan.rs",
    "crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs",
    "crates/harnesslab-cli/src/runner/external/swe_bench_pro_adapter.rs",
];

const LEGACY_SHIM_FILES: &[&str] = &[
    "crates/harnesslab-core/src/adapter_protocol.rs",
    "crates/harnesslab-core/src/benchmark.rs",
    "crates/harnesslab-core/src/runtime.rs",
    "crates/harnesslab-cli/src/doctor_run_as.rs",
    "crates/harnesslab-cli/src/runtime_compatibility.rs",
    "crates/harnesslab-cli/src/runner/external.rs",
    "crates/harnesslab-cli/src/runner/external/runtime_adapter.rs",
    "crates/harnesslab-cli/src/runner/external/runtime_adapter_test_support.rs",
    "crates/harnesslab-cli/src/runner/external/runtime_anchor.rs",
    "crates/harnesslab-cli/src/runner/external/runtime_authority.rs",
    "crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs",
    "crates/harnesslab-cli/src/runner/replay.rs",
    "crates/harnesslab-cli/src/runner/store.rs",
    "crates/harnesslab-core/src/agent_profile_reference.rs",
];

const METADATA_FILES: &[&str] = &[
    "xtask/src/adapter_claims.rs",
    "xtask/src/forbidden_diff_guard.rs",
    "xtask/src/frozen_execution_files.rs",
    "xtask/src/frozen_selector_ids.rs",
    "xtask/src/frozen_selectors.rs",
    "xtask/src/no_branch_guard.rs",
    "xtask/src/runtime_artifacts.rs",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NoBranchViolation {
    pub path: String,
    pub line: usize,
    pub token: &'static str,
}

#[derive(Serialize)]
struct NoBranchGuardReport {
    schema_version: u32,
    scanned_files: usize,
    adapter_owned_prefixes: &'static [&'static str],
    adapter_owned_files: &'static [&'static str],
    legacy_shim_files: &'static [&'static str],
    metadata_files: &'static [&'static str],
    forbidden_tokens: &'static [&'static str],
    violations: Vec<NoBranchViolation>,
}

pub(crate) fn verify_no_branch_guard() -> Result<()> {
    let report = scan_no_branch_guard(Path::new("."))?;
    write_report(&report)?;
    if !report.violations.is_empty() {
        let details = report
            .violations
            .iter()
            .map(|violation| {
                format!(
                    "{}:{} contains forbidden benchmark branch token `{}`",
                    violation.path, violation.line, violation.token
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        bail!("adapter protocol no-branch guard failed:\n{details}");
    }
    println!(
        "adapter protocol no-branch guard ok: scanned_files={} artifact=artifacts/no-branch-guard.json",
        report.scanned_files
    );
    Ok(())
}

fn scan_no_branch_guard(root: &Path) -> Result<NoBranchGuardReport> {
    let mut scanned_files = 0;
    let violations = scan_no_branch_violations(root, &mut scanned_files)?;
    Ok(NoBranchGuardReport {
        schema_version: 1,
        scanned_files,
        adapter_owned_prefixes: ADAPTER_OWNED_PREFIXES,
        adapter_owned_files: ADAPTER_OWNED_FILES,
        legacy_shim_files: LEGACY_SHIM_FILES,
        metadata_files: METADATA_FILES,
        forbidden_tokens: FORBIDDEN_TOKENS,
        violations,
    })
}

fn write_report(report: &NoBranchGuardReport) -> Result<()> {
    std::fs::create_dir_all("artifacts").context("create artifacts directory")?;
    std::fs::write(
        "artifacts/no-branch-guard.json",
        serde_json::to_vec_pretty(report)?,
    )
    .context("write artifacts/no-branch-guard.json")
}

fn scan_no_branch_violations(
    root: &Path,
    scanned_files: &mut usize,
) -> Result<Vec<NoBranchViolation>> {
    let mut violations = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = relative_path(root, entry.path());
        if !is_scanned_production_source(&relative) || is_allowed_path(&relative) {
            continue;
        }
        *scanned_files += 1;
        let content = std::fs::read_to_string(entry.path())?;
        for (index, line) in content.lines().enumerate() {
            let stripped = strip_line_comment(line);
            if let Some(token) = FORBIDDEN_TOKENS
                .iter()
                .copied()
                .find(|token| stripped.contains(token))
            {
                violations.push(NoBranchViolation {
                    path: relative.clone(),
                    line: index + 1,
                    token,
                });
            }
        }
    }
    violations.sort_by(|left, right| left.path.cmp(&right.path).then(left.line.cmp(&right.line)));
    Ok(violations)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_scanned_production_source(path: &str) -> bool {
    path.ends_with(".rs")
        && (path.starts_with("crates/")
            || path.starts_with("xtask/src/")
            || path.starts_with("integrations/"))
        && !path.contains("/tests/")
        && !path.ends_with("_tests.rs")
        && !path.ends_with("_contract.rs")
}

fn is_allowed_path(path: &str) -> bool {
    ADAPTER_OWNED_PREFIXES
        .iter()
        .any(|prefix| path.starts_with(prefix))
        || ADAPTER_OWNED_FILES.contains(&path)
        || LEGACY_SHIM_FILES.contains(&path)
        || METADATA_FILES.contains(&path)
}

fn strip_line_comment(line: &str) -> &str {
    line.split_once("//")
        .map(|(before, _)| before)
        .unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn adapt_protocol_008_rejects_new_generic_layer_benchmark_branch() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_dispatch.rs",
            r#"fn run(kind: ExternalRunnerKind) { let _ = "terminal-bench"; }"#,
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();

        assert_eq!(violations.len(), 1);
        assert_eq!(scanned_files, 1);
        assert_eq!(
            violations[0].path,
            "crates/harnesslab-cli/src/runner/generic_dispatch.rs"
        );
        assert_eq!(violations[0].token, "ExternalRunnerKind");
    }

    #[test]
    fn adapt_protocol_008_rejects_future_id_and_protocol_key_branches() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_future.rs",
            r#"
fn by_benchmark(benchmark_id: &str, adapter_id: &str) {
    if benchmark_id == "third-bench" {}
    match adapter_id.as_str() { _ => {} }
}
"#,
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();
        let tokens = violations
            .iter()
            .map(|violation| violation.token)
            .collect::<Vec<_>>();

        assert_eq!(scanned_files, 1);
        assert!(tokens.contains(&"benchmark_id =="), "{violations:?}");
        assert!(tokens.contains(&"adapter_id.as_str()"), "{violations:?}");
    }

    #[test]
    fn adapt_protocol_008_rejects_reversed_protocol_key_equality() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_reversed.rs",
            r#"
fn by_benchmark(benchmark_id: &str, adapter_id: &str) {
    if "third-bench" == benchmark_id {}
    if "harnesslab.third-bench.runtime" == adapter_id {}
}
"#,
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();
        let tokens = violations
            .iter()
            .map(|violation| violation.token)
            .collect::<Vec<_>>();

        assert_eq!(scanned_files, 1);
        assert!(tokens.contains(&"== benchmark_id"), "{violations:?}");
        assert!(tokens.contains(&"== adapter_id"), "{violations:?}");
    }

    #[test]
    fn adapt_protocol_008_rejects_literal_side_eq_and_ne() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_literal_eq.rs",
            r#"
fn by_benchmark(benchmark_id: &str, adapter_id: &str) {
    if "third-bench".eq(benchmark_id) {}
    if "harnesslab.third-bench.runtime".ne(adapter_id) {}
}
"#,
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();
        let tokens = violations
            .iter()
            .map(|violation| violation.token)
            .collect::<Vec<_>>();

        assert_eq!(scanned_files, 1);
        assert!(tokens.contains(&".eq(benchmark_id)"), "{violations:?}");
        assert!(tokens.contains(&".ne(adapter_id)"), "{violations:?}");
    }

    #[test]
    fn adapt_protocol_008_rejects_inequality_branches() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_ineq.rs",
            r#"
fn by_benchmark(benchmark_id: &str, adapter_id: &str) {
    if benchmark_id != "third-bench" {}
    if "harnesslab.third-bench.runtime" != adapter_id {}
}
"#,
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();
        let tokens = violations
            .iter()
            .map(|violation| violation.token)
            .collect::<Vec<_>>();

        assert_eq!(scanned_files, 1);
        assert!(tokens.contains(&"benchmark_id !="), "{violations:?}");
        assert!(tokens.contains(&"!= adapter_id"), "{violations:?}");
    }

    #[test]
    fn adapt_protocol_008_rejects_direct_adapter_imports_in_generic_layers() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic_import.rs",
            "use harnesslab_adapters::TerminalBenchAdapter;\nfn generic() {}",
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();

        assert_eq!(scanned_files, 1);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].token,
            "use harnesslab_adapters::TerminalBenchAdapter"
        );
    }

    #[test]
    fn adapt_protocol_008_allows_adapter_owned_and_legacy_paths_only() {
        let root = fixture_root();
        write_file(
            root.path(),
            "crates/harnesslab-adapters/src/terminal_bench.rs",
            r#"fn adapter() { let _ = "terminal-bench"; }"#,
        );
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runtime_compatibility.rs",
            "fn legacy(kind: ExternalRunnerKind) {}",
        );
        write_file(
            root.path(),
            "crates/harnesslab-cli/src/runner/generic.rs",
            "// terminal-bench in a comment is migration documentation\nfn generic() {}",
        );

        let mut scanned_files = 0;
        let violations = scan_no_branch_violations(root.path(), &mut scanned_files).unwrap();

        assert_eq!(scanned_files, 1);
        assert!(violations.is_empty(), "{violations:?}");
    }

    #[test]
    fn adapt_protocol_008_current_production_sources_pass_no_branch_guard() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let report = scan_no_branch_guard(repo_root).unwrap();

        assert!(report.scanned_files > 0);
        assert!(report.violations.is_empty(), "{:?}", report.violations);
    }

    #[test]
    fn adapt_protocol_008_allowlist_inventory_is_review_locked() {
        assert_eq!(
            ADAPTER_OWNED_PREFIXES,
            &[
                "crates/harnesslab-adapters/src/",
                "crates/harnesslab-cli/src/runner/external/terminal_bench",
                "crates/harnesslab-cli/src/runner/external/swe_bench_pro",
            ]
        );
        assert_eq!(
            ADAPTER_OWNED_FILES,
            &[
                "crates/harnesslab-cli/src/runner/external/log_scan.rs",
                "crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs",
                "crates/harnesslab-cli/src/runner/external/swe_bench_pro_adapter.rs",
            ]
        );
        assert_eq!(
            LEGACY_SHIM_FILES,
            &[
                "crates/harnesslab-core/src/adapter_protocol.rs",
                "crates/harnesslab-core/src/benchmark.rs",
                "crates/harnesslab-core/src/runtime.rs",
                "crates/harnesslab-cli/src/doctor_run_as.rs",
                "crates/harnesslab-cli/src/runtime_compatibility.rs",
                "crates/harnesslab-cli/src/runner/external.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_adapter.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_adapter_test_support.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_anchor.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_authority.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs",
                "crates/harnesslab-cli/src/runner/replay.rs",
                "crates/harnesslab-cli/src/runner/store.rs",
                "crates/harnesslab-core/src/agent_profile_reference.rs",
            ]
        );
        assert_eq!(
            METADATA_FILES,
            &[
                "xtask/src/adapter_claims.rs",
                "xtask/src/forbidden_diff_guard.rs",
                "xtask/src/frozen_execution_files.rs",
                "xtask/src/frozen_selector_ids.rs",
                "xtask/src/frozen_selectors.rs",
                "xtask/src/no_branch_guard.rs",
                "xtask/src/runtime_artifacts.rs",
            ]
        );
    }

    fn fixture_root() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_file(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
}
