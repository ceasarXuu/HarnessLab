use super::*;

#[test]
fn coverage_001_module_thresholds_are_enforced() {
    let records = parse_lcov(
        "\
SF:/repo/crates/harnesslab-cli/src/lib.rs
LF:100
LH:94
BRF:10
BRH:10
end_of_record
",
    )
    .unwrap();
    let modules = [ModuleThreshold {
        name: "cli".to_string(),
        path: PathBuf::from("crates/harnesslab-cli/src"),
        threshold: CoverageThreshold {
            line: 95.0,
            branch: 95.0,
        },
    }];

    let error = assert_thresholds(
        &records,
        CoverageThreshold {
            line: 0.0,
            branch: 0.0,
        },
        &modules,
        Path::new("/repo"),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("module cli line coverage 94.00%"));
}

#[test]
fn coverage_002_branch_threshold_requires_branch_data() {
    let records = parse_lcov(
        "\
SF:/repo/crates/harnesslab-core/src/lib.rs
LF:10
LH:10
BRF:0
BRH:0
end_of_record
",
    )
    .unwrap();

    let error = assert_thresholds(
        &records,
        CoverageThreshold {
            line: 95.0,
            branch: 95.0,
        },
        &[],
        Path::new("/repo"),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("branch coverage has no branch data"));
}

#[test]
fn coverage_003_new_files_must_appear_in_lcov() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("crates/harnesslab-cli/src/lib.rs");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "pub fn run() {}\n").unwrap();
    let records = parse_lcov(
        "\
SF:/repo/crates/harnesslab-core/src/lib.rs
LF:10
LH:10
BRF:0
BRH:0
end_of_record
",
    )
    .unwrap();

    let error = assert_new_file_coverage(
        &records,
        &[PathBuf::from("crates/harnesslab-cli/src/lib.rs")],
        95.0,
        tmp.path(),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("missing coverage for new production file"));
}

#[test]
fn coverage_007_type_only_new_files_do_not_require_lcov_records() {
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("crates/harnesslab-cli/src/output.rs");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "pub struct Output { pub status: String }\n").unwrap();

    assert_new_file_coverage(
        &[],
        &[PathBuf::from("crates/harnesslab-cli/src/output.rs")],
        95.0,
        tmp.path(),
    )
    .unwrap();
}
