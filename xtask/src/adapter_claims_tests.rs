use super::*;
use crate::{RegistryDoc, TestEntry, Verifies};
use std::collections::BTreeSet;

const ACTIVE_DATA_ID: &str = "ADAPT-DATA-001";
const ACTIVE_DATA_TEST: &str =
    "data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache";

#[test]
fn registry_001_claim_parser_expands_ranges_across_sources() {
    let sources = vec![
        (
            "a.md".to_string(),
            "`ADAPT-DATA-001..003` plus `SWEPRO-005`".to_string(),
        ),
        (
            "b.md".to_string(),
            "`ADAPT-RUNTIME-001` and ADAPT-DATA-*".to_string(),
        ),
    ];

    let ids = claimed_adapter_test_ids_from_sources(&sources).unwrap();
    assert!(ids.contains("ADAPT-DATA-001"));
    assert!(ids.contains("ADAPT-DATA-002"));
    assert!(ids.contains("ADAPT-DATA-003"));
    assert!(ids.contains("ADAPT-RUNTIME-001"));
    assert!(ids.contains("SWEPRO-005"));
    assert!(!ids.contains("ADAPT-DATA-*"));
}

#[test]
fn registry_002_claim_parser_rejects_descending_ranges() {
    let sources = vec![("bad.md".to_string(), "ADAPT-DATA-003..001".to_string())];

    let error = claimed_adapter_test_ids_from_sources(&sources)
        .unwrap_err()
        .to_string();
    assert!(error.contains("descending"));
}

#[test]
fn registry_003_claimed_planned_id_must_route_to_planned_handler() {
    let claimed = BTreeSet::from(["ADAPT-DATA-001".to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc("ADAPT-DATA-001", "planned");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("planned_adapter_proof"));
}

#[test]
fn registry_004_claimed_ids_must_exist_in_all_surfaces() {
    let claimed = BTreeSet::from(["ADAPT-DATA-001".to_string()]);
    let requirements = BTreeSet::new();
    let tests = claimed.clone();
    let registry = registry_doc("ADAPT-DATA-001", "planned");
    let script = "ADAPT-DATA-001) planned_adapter_proof \"$id\" \"phase\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("REQUIREMENTS"));
}

#[test]
fn registry_005_planned_status_is_limited_to_claimed_adapter_ids() {
    let claimed = BTreeSet::from(["ADAPT-DATA-001".to_string()]);

    assert!(planned_status_allowed("ADAPT-DATA-001", &claimed));
    assert!(!planned_status_allowed("cli_help_stable", &claimed));
}

#[test]
fn registry_006_claimed_active_id_must_not_route_to_planned_handler() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) planned_adapter_proof \"$id\" \"phase\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("must not route to planned_adapter_proof"));
}

#[test]
fn registry_007_claimed_active_id_rejects_marker_spoofed_wrong_test() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"adapt_data_001_unrelated_passing_test\"; test_target=\"lib\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("adapt_data_001_unrelated_passing_test"));
    assert!(error.contains(ACTIVE_DATA_TEST));
}

#[test]
fn registry_008_claimed_active_id_accepts_exact_route_spec() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_target=\"lib\" ;;";

    ensure_claimed_adapter_ids_are_registered(&claimed, &requirements, &tests, &registry, script)
        .unwrap();
}

#[test]
fn registry_009_claimed_active_id_rejects_duplicate_assignment_override() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_name=\"some_other_passing_test\"; test_target=\"lib\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("duplicate test_name"));
    assert!(error.contains("some_other_passing_test"));
}

#[test]
fn registry_010_claimed_active_id_rejects_single_quoted_duplicate_assignment_override() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_name='some_other_passing_test'; test_target=\"lib\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("duplicate test_name"));
    assert!(error.contains("some_other_passing_test"));
}

#[test]
fn registry_011_claimed_active_id_rejects_unquoted_duplicate_assignment_override() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let script = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_name=some_other_passing_test; test_target=\"lib\" ;;";

    let error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        script,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("duplicate test_name"));
    assert!(error.contains("some_other_passing_test"));
}

#[test]
fn registry_012_claimed_active_id_rejects_duplicate_package_and_target_overrides() {
    let claimed = BTreeSet::from([ACTIVE_DATA_ID.to_string()]);
    let requirements = claimed.clone();
    let tests = claimed.clone();
    let registry = registry_doc(ACTIVE_DATA_ID, "active");
    let duplicate_package = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; package='harnesslab-cli'; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_target=\"lib\" ;;";
    let duplicate_target = "ADAPT-DATA-001) package=\"harnesslab-adapters\"; test_name=\"data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache\"; test_target=\"lib\"; test_target=integration ;;";

    let package_error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        duplicate_package,
    )
    .unwrap_err()
    .to_string();
    assert!(package_error.contains("duplicate package"));

    let target_error = ensure_claimed_adapter_ids_are_registered(
        &claimed,
        &requirements,
        &tests,
        &registry,
        duplicate_target,
    )
    .unwrap_err()
    .to_string();
    assert!(target_error.contains("duplicate test_target"));
}

fn registry_doc(id: &str, status: &str) -> RegistryDoc {
    RegistryDoc {
        schema_version: 1,
        tests: vec![TestEntry {
            id: id.to_string(),
            title: "planned".to_string(),
            command: format!("scripts/test-after-change.sh --select {id}"),
            file_patterns: vec![],
            required_artifacts: Some(vec![]),
            status: status.to_string(),
            labels: None,
            verifies: Verifies {
                requirements: vec![id.to_string()],
                contracts: vec!["BenchmarkDataAdapter".to_string()],
            },
        }],
    }
}
