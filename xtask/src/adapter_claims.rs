use super::RegistryDoc;
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs;

const ADAPTER_CLAIM_SOURCES: &[&str] = &[
    "docs/plans/2026-06-04-benchmark-adapter-architecture-design.md",
    "docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md",
    "docs/plans/2026-06-04-benchmark-adapter-phase-1-inventory.md",
];

pub(super) fn load_adapter_claim_sources() -> Result<Vec<(String, String)>> {
    ADAPTER_CLAIM_SOURCES
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .with_context(|| format!("read adapter claim source {path}"))
                .map(|content| ((*path).to_string(), content))
        })
        .collect()
}

pub(super) fn claimed_adapter_test_ids_from_sources(
    sources: &[(String, String)],
) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for (source, content) in sources {
        collect_claimed_adapter_test_ids(content, source, &mut ids)?;
    }
    Ok(ids)
}

pub(super) fn ensure_claimed_adapter_ids_are_registered(
    claimed: &BTreeSet<String>,
    requirement_ids: &BTreeSet<String>,
    test_ids: &BTreeSet<String>,
    registry: &RegistryDoc,
    selector_script: &str,
) -> Result<()> {
    for id in claimed {
        if !requirement_ids.contains(id) {
            bail!("adapter plan claims {id}, but tests/REQUIREMENTS.toml does not register it");
        }
        if !test_ids.contains(id) {
            bail!("adapter plan claims {id}, but tests/TEST_REGISTRY.toml does not register it");
        }
        let entry = registry
            .tests
            .iter()
            .find(|test| test.id == *id)
            .expect("test id checked above");
        if entry.command != format!("scripts/test-after-change.sh --select {id}") {
            bail!("adapter plan claims {id}, but registry command is not routed through --select");
        }
        let route = selector_route_for_id(selector_script, id).ok_or_else(|| {
            anyhow::anyhow!(
                "adapter plan claims {id}, but scripts/test-after-change.sh has no selector route"
            )
        })?;
        if entry.status == "planned" && !route.contains("planned_adapter_proof") {
            bail!("planned adapter proof {id} must route to planned_adapter_proof, got: {route}");
        }
        if entry.status == "active" {
            ensure_active_route_matches_claimed_id(id, entry, route)?;
        }
    }
    Ok(())
}

pub(super) fn planned_status_allowed(id: &str, claimed: &BTreeSet<String>) -> bool {
    claimed.contains(id)
}

pub(super) fn write_adapter_proof_inventory(
    claimed: &BTreeSet<String>,
    sources: &[(String, String)],
    registry: &RegistryDoc,
    selector_script: &str,
) -> Result<()> {
    fs::create_dir_all("artifacts")?;
    let statuses = claimed
        .iter()
        .map(|id| {
            let status = registry
                .tests
                .iter()
                .find(|test| test.id == *id)
                .map(|test| test.status.as_str())
                .unwrap_or("missing");
            let route = selector_route_for_id(selector_script, id).unwrap_or("missing");
            serde_json::json!({
                "id": id,
                "test_status": status,
                "selector_class": selector_class(status, route),
                "selector_route": route
            })
        })
        .collect::<Vec<_>>();
    let source_paths = sources
        .iter()
        .map(|(path, _)| path)
        .cloned()
        .collect::<Vec<_>>();
    fs::write(
        "artifacts/adapter-proof-inventory.json",
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": 1,
            "claim_sources": source_paths,
            "claimed_ids": statuses
        }))?,
    )?;
    Ok(())
}

fn collect_claimed_adapter_test_ids(
    content: &str,
    source: &str,
    ids: &mut BTreeSet<String>,
) -> Result<()> {
    for prefix in ["ADAPT-DATA-", "ADAPT-RUNTIME-", "SWEPRO-"] {
        let mut offset = 0;
        while let Some(relative) = content[offset..].find(prefix) {
            let start = offset + relative;
            let digits_start = start + prefix.len();
            let digits = leading_digits(&content[digits_start..]);
            if digits.is_empty() {
                offset = digits_start;
                continue;
            }
            let after_digits = digits_start + digits.len();
            if content[after_digits..].starts_with("..") {
                let end_start = after_digits + 2;
                let end_digits = leading_digits(&content[end_start..]);
                if end_digits.is_empty() {
                    bail!("invalid adapter id range in {source}: {prefix}{digits}..");
                }
                let first: u32 = digits.parse()?;
                let last: u32 = end_digits.parse()?;
                if last < first {
                    bail!(
                        "invalid descending adapter id range in {source}: {prefix}{digits}..{end_digits}"
                    );
                }
                for value in first..=last {
                    ids.insert(format!("{prefix}{value:0width$}", width = digits.len()));
                }
                offset = end_start + end_digits.len();
            } else {
                ids.insert(format!("{prefix}{digits}"));
                offset = after_digits;
            }
        }
    }
    Ok(())
}

fn leading_digits(value: &str) -> &str {
    let end = value
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    &value[..end]
}

fn selector_route_for_id<'a>(script: &'a str, id: &str) -> Option<&'a str> {
    let needle = format!("{id})");
    script
        .lines()
        .map(str::trim_start)
        .find(|line| line.starts_with(&needle))
}

fn ensure_active_route_matches_claimed_id(
    id: &str,
    entry: &super::TestEntry,
    route: &str,
) -> Result<()> {
    if route.contains("planned_adapter_proof") {
        bail!("active adapter proof {id} must not route to planned_adapter_proof, got: {route}");
    }
    let spec = active_route_spec(id)
        .ok_or_else(|| anyhow::anyhow!("active adapter proof {id} has no expected route spec"))?;
    ensure_assignment(route, "package", spec.package, id)?;
    ensure_assignment(route, "test_name", spec.test_name, id)?;
    if let Some(test_target) = spec.test_target {
        ensure_assignment(route, "test_target", test_target, id)?;
    }
    for expected in spec.file_patterns {
        if !entry
            .file_patterns
            .iter()
            .any(|pattern| pattern == expected)
        {
            bail!(
                "active adapter proof {id} file_patterns missing expected selector file {expected}"
            );
        }
    }
    Ok(())
}

fn selector_class(status: &str, route: &str) -> &'static str {
    if status == "planned" && route.contains("planned_adapter_proof") {
        "planned-proof"
    } else if status == "active" {
        "active-test"
    } else {
        "unknown"
    }
}

struct ActiveRouteSpec {
    package: &'static str,
    test_name: &'static str,
    test_target: Option<&'static str>,
    file_patterns: &'static [&'static str],
}

fn active_route_spec(id: &str) -> Option<ActiveRouteSpec> {
    match id {
        "ADAPT-DATA-001" => Some(ActiveRouteSpec {
            package: "harnesslab-adapters",
            test_name: "data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-adapters/src/data_contract_tests.rs"],
        }),
        "ADAPT-DATA-002" => Some(ActiveRouteSpec {
            package: "harnesslab-adapters",
            test_name: "data_contract_tests::adapt_data_002_prepare_is_idempotent_and_rejects_unready_data",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-adapters/src/data_contract_tests.rs"],
        }),
        "ADAPT-DATA-003" => Some(ActiveRouteSpec {
            package: "harnesslab-adapters",
            test_name: "data_contract_tests::adapt_data_003_list_tasks_returns_stable_task_ids_and_source_refs",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-adapters/src/data_contract_tests.rs"],
        }),
        "ADAPT-DATA-004" => Some(ActiveRouteSpec {
            package: "harnesslab-adapters",
            test_name: "data_contract_tests::adapt_data_004_snapshot_task_captures_replay_sufficient_identity",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-adapters/src/data_contract_tests.rs"],
        }),
        "ADAPT-DATA-005" => Some(ActiveRouteSpec {
            package: "harnesslab-adapters",
            test_name: "data_contract_tests::adapt_data_005_create_task_plan_is_stable_and_plan_is_wrapper",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-adapters/src/data_contract_tests.rs"],
        }),
        "ADAPT-RUNTIME-001" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "runner::external::runtime_adapter::tests::adapt_runtime_001_external_entrypoints_delegate_to_runtime_registry",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs"],
        }),
        "ADAPT-RUNTIME-002" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "runner::external::runtime_adapter::tests::adapt_runtime_002_preflight_reports_and_enforces_current_compatibility",
            test_target: Some("lib"),
            file_patterns: &["crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs"],
        }),
        "ADAPT-RUNTIME-003" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "adapt_runtime_003_external_runtime_snapshots_are_written_and_redacted",
            test_target: Some("test:external_runtime_snapshot_contract"),
            file_patterns: &[
                "crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs",
                "crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench.rs",
            ],
        }),
        "ADAPT-RUNTIME-004" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "adapt_runtime_004_cleanup_report_is_structured_and_affects_final_verdict",
            test_target: Some("test:external_runtime_snapshot_contract"),
            file_patterns: &[
                "crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs",
            ],
        }),
        "ADAPT-RUNTIME-005" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "adapt_runtime_005_terminal_bench_event_taxonomy_is_stable",
            test_target: Some("test:terminal_bench_runtime_event_contract"),
            file_patterns: &[
                "crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs",
                "crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs",
            ],
        }),
        "SWEPRO-001" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "swepro_001_metadata_failure_is_classified_and_observable",
            test_target: Some("test:swe_runtime_phase_contract"),
            file_patterns: SWE_PHASE_FILE_PATTERNS,
        }),
        "SWEPRO-002" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "swepro_002_workspace_failure_is_classified_and_observable",
            test_target: Some("test:swe_runtime_phase_contract"),
            file_patterns: SWE_PHASE_FILE_PATTERNS,
        }),
        "SWEPRO-003" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "swepro_003_diff_capture_failure_and_empty_patch_are_distinct",
            test_target: Some("test:swe_runtime_phase_contract"),
            file_patterns: SWE_PHASE_FILE_PATTERNS,
        }),
        "SWEPRO-004" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "swepro_004_evaluator_parse_corruption_is_not_patch_failure",
            test_target: Some("test:swe_runtime_phase_contract"),
            file_patterns: SWE_PHASE_FILE_PATTERNS,
        }),
        "SWEPRO-005" => Some(ActiveRouteSpec {
            package: "harnesslab-cli",
            test_name: "swepro_005_replay_requires_stored_swe_runtime_materials",
            test_target: Some("test:swe_runtime_snapshot_contract"),
            file_patterns: &[
                "crates/harnesslab-cli/src/runner/replay.rs",
                "crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs",
                "crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs",
            ],
        }),
        _ => None,
    }
}

const SWE_PHASE_FILE_PATTERNS: &[&str] = &[
    "crates/harnesslab-cli/tests/swe_runtime_phase_contract.rs",
    "crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs",
    "crates/harnesslab-cli/src/runner/external/swe_bench_pro_adapter.rs",
];

fn ensure_assignment(route: &str, key: &str, expected: &str, id: &str) -> Result<()> {
    let values = assignment_values(route, key);
    if values.is_empty() {
        bail!("active adapter proof {id} route is missing {key}= assignment: {route}");
    }
    if values.len() > 1 {
        bail!(
            "active adapter proof {id} route has duplicate {key}= assignments: {}",
            values.join(", ")
        );
    }
    let actual = values[0];
    if actual != expected {
        bail!("active adapter proof {id} route has {key}={actual}, expected {expected}: {route}");
    }
    Ok(())
}

fn assignment_values<'a>(route: &'a str, key: &str) -> Vec<&'a str> {
    let needle = format!("{key}=");
    let mut values = Vec::new();
    let mut offset = 0;
    while let Some(relative_start) = route[offset..].find(&needle) {
        let value_start = offset + relative_start + needle.len();
        let value = &route[value_start..];
        let Some((value, consumed)) = shell_assignment_value(value) else {
            offset = value_start;
            continue;
        };
        values.push(value);
        offset = value_start + consumed;
    }
    values
}

fn shell_assignment_value(value: &str) -> Option<(&str, usize)> {
    match value.as_bytes().first().copied()? {
        b'"' => {
            let end = value[1..].find('"')? + 1;
            Some((&value[1..end], end + 1))
        }
        b'\'' => {
            let end = value[1..].find('\'')? + 1;
            Some((&value[1..end], end + 1))
        }
        _ => {
            let end = value
                .bytes()
                .take_while(|byte| !byte.is_ascii_whitespace() && *byte != b';')
                .count();
            if end == 0 {
                None
            } else {
                Some((&value[..end], end))
            }
        }
    }
}

#[cfg(test)]
#[path = "adapter_claims_tests.rs"]
mod tests;
