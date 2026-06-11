use std::collections::BTreeSet;

pub(crate) fn allowed_std_fs_calls() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "canonicalize",
        "metadata",
        "read",
        "read_dir",
        "read_link",
        "read_to_string",
        "symlink_metadata",
    ])
}

pub(crate) fn forbidden_runtime_path_literals() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "attempt",
        "attempt_dir",
        "events.jsonl",
        "external-runtime",
        "run_dir",
    ])
}

pub(crate) fn artifact_declaration_source(path: &str) -> bool {
    matches!(
        path,
        "crates/harnesslab-adapters/src/scaffold_golden_adapter.rs"
            | "crates/harnesslab-adapters/src/swe_bench_pro_artifacts.rs"
            | "crates/harnesslab-adapters/src/swe_bench_pro_protocol.rs"
            | "crates/harnesslab-adapters/src/terminal_bench_protocol.rs"
    )
}

pub(crate) fn strip_artifact_declaration_calls(source: &str) -> String {
    let mut stripped = String::new();
    let mut skipping = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("artifact(") {
            skipping = true;
        }
        if !skipping {
            stripped.push_str(line);
            stripped.push('\n');
        }
        if skipping && matches!(line.trim(), ")," | ");") {
            skipping = false;
        }
    }
    stripped
}
