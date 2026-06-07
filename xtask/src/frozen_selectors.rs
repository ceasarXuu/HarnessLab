use crate::frozen_execution_files::{self, FrozenExecutionFile};
use crate::frozen_selector_ids::REQUIRED_FROZEN_IDS;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const MANIFEST_PATH: &str = "tests/FROZEN_SELECTOR_MANIFEST.toml";
const REGISTRY_PATH: &str = "tests/TEST_REGISTRY.toml";
const ROUTER_PATH: &str = "scripts/test-after-change.sh";

pub(crate) fn verify_frozen_selector_manifest() -> Result<()> {
    let registry = load_registry(REGISTRY_PATH)?;
    let router = fs::read_to_string(ROUTER_PATH).context("read selector router")?;
    let manifest = load_manifest(MANIFEST_PATH)?;
    let summary = verify_manifest(&manifest, &registry, &router)?;
    println!(
        "frozen selector manifest ok: total={} execution_files={} {}",
        summary.counts.total,
        summary.execution_files,
        summary.counts.render_families()
    );
    Ok(())
}

pub(crate) fn print_frozen_selector_manifest() -> Result<()> {
    let registry = load_registry(REGISTRY_PATH)?;
    let router = fs::read_to_string(ROUTER_PATH).context("read selector router")?;
    let manifest = current_manifest(&registry, &router)?;
    print!("{}", toml::to_string_pretty(&manifest)?);
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct FrozenSelectorManifest {
    schema_version: u32,
    execution_files: Vec<FrozenExecutionFile>,
    selectors: Vec<FrozenSelector>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct FrozenSelector {
    id: String,
    status: String,
    command: String,
    router_case: String,
    expected_test_count: Option<u32>,
    expected_pass_threshold: String,
    file_patterns: Vec<String>,
    required_artifacts: Vec<String>,
    owning_contracts: Vec<String>,
}

#[derive(Deserialize)]
struct RegistryDoc {
    schema_version: u32,
    tests: Vec<TestEntry>,
}

#[derive(Deserialize)]
struct TestEntry {
    id: String,
    command: String,
    file_patterns: Vec<String>,
    required_artifacts: Option<Vec<String>>,
    status: String,
    verifies: Verifies,
}

#[derive(Deserialize)]
struct Verifies {
    contracts: Vec<String>,
}

fn load_manifest(path: &str) -> Result<FrozenSelectorManifest> {
    let content = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    toml::from_str(&content).with_context(|| format!("parse {path}"))
}

fn load_registry(path: &str) -> Result<RegistryDoc> {
    let content = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    toml::from_str(&content).with_context(|| format!("parse {path}"))
}

fn verify_manifest(
    manifest: &FrozenSelectorManifest,
    registry: &RegistryDoc,
    router: &str,
) -> Result<ManifestSummary> {
    if manifest.schema_version != 1 {
        bail!(
            "{MANIFEST_PATH} has unsupported schema_version {}",
            manifest.schema_version
        );
    }
    if registry.schema_version != 1 {
        bail!(
            "{REGISTRY_PATH} has unsupported schema_version {}",
            registry.schema_version
        );
    }
    let execution_files =
        frozen_execution_files::verify_execution_files(&manifest.execution_files)?;

    let registry_by_id: BTreeMap<&str, &TestEntry> = registry
        .tests
        .iter()
        .map(|test| (test.id.as_str(), test))
        .collect();
    let mut seen = BTreeSet::new();
    let required_ids: BTreeSet<&str> = REQUIRED_FROZEN_IDS.iter().copied().collect();

    for selector in &manifest.selectors {
        if !seen.insert(selector.id.as_str()) {
            bail!("duplicate frozen selector id: {}", selector.id);
        }
        let test = registry_by_id
            .get(selector.id.as_str())
            .with_context(|| format!("frozen selector {} missing from registry", selector.id))?;
        verify_selector(selector, test, router)?;
    }

    let manifest_ids: BTreeSet<&str> = manifest
        .selectors
        .iter()
        .map(|selector| selector.id.as_str())
        .collect();
    if manifest_ids != required_ids {
        for id in required_ids.difference(&manifest_ids) {
            bail!("required frozen selector {id} missing from {MANIFEST_PATH}");
        }
        for id in manifest_ids.difference(&required_ids) {
            bail!("unexpected frozen selector {id} in {MANIFEST_PATH}");
        }
    }

    Ok(ManifestSummary {
        counts: FamilyCounts::from_ids(&manifest_ids),
        execution_files,
    })
}

fn verify_selector(selector: &FrozenSelector, test: &TestEntry, router: &str) -> Result<()> {
    if selector.expected_pass_threshold.trim().is_empty() {
        bail!("frozen selector {} has empty pass threshold", selector.id);
    }
    if selector.status != test.status {
        bail!(
            "frozen selector {} status changed: manifest={} registry={}",
            selector.id,
            selector.status,
            test.status
        );
    }
    if selector.command != test.command {
        bail!(
            "frozen selector {} command changed: manifest={} registry={}",
            selector.id,
            selector.command,
            test.command
        );
    }
    let artifacts = test.required_artifacts.clone().unwrap_or_default();
    if selector.required_artifacts != artifacts {
        bail!("frozen selector {} required_artifacts changed", selector.id);
    }
    if selector.file_patterns != test.file_patterns {
        bail!("frozen selector {} file_patterns changed", selector.id);
    }
    if selector.owning_contracts != test.verifies.contracts {
        bail!("frozen selector {} owning_contracts changed", selector.id);
    }
    for pattern in &selector.file_patterns {
        if !Path::new(pattern).exists() {
            bail!(
                "frozen selector {} file pattern missing: {pattern}",
                selector.id
            );
        }
    }

    let route = selector_route(router, &selector.id)
        .with_context(|| format!("frozen selector {} missing router case", selector.id))?;
    if selector.router_case != route {
        bail!("frozen selector {} router case changed", selector.id);
    }
    let inferred_count = inferred_test_count(route);
    if selector.expected_test_count != inferred_count {
        bail!(
            "frozen selector {} expected_test_count changed: manifest={:?} router={:?}",
            selector.id,
            selector.expected_test_count,
            inferred_count
        );
    }
    Ok(())
}

fn current_manifest(registry: &RegistryDoc, router: &str) -> Result<FrozenSelectorManifest> {
    let mut selectors = Vec::new();
    for test in registry
        .tests
        .iter()
        .filter(|test| REQUIRED_FROZEN_IDS.contains(&test.id.as_str()))
    {
        let route = selector_route(router, &test.id)
            .with_context(|| format!("frozen selector {} missing router case", test.id))?;
        selectors.push(FrozenSelector {
            id: test.id.clone(),
            status: test.status.clone(),
            command: test.command.clone(),
            router_case: route.to_string(),
            expected_test_count: inferred_test_count(route),
            expected_pass_threshold: pass_threshold(route),
            file_patterns: test.file_patterns.clone(),
            required_artifacts: test.required_artifacts.clone().unwrap_or_default(),
            owning_contracts: test.verifies.contracts.clone(),
        });
    }
    selectors.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(FrozenSelectorManifest {
        schema_version: 1,
        execution_files: frozen_execution_files::current_execution_files()?,
        selectors,
    })
}

fn selector_route<'a>(router: &'a str, id: &str) -> Option<&'a str> {
    let needle = format!("{id})");
    router.lines().map(str::trim_start).find(|line| {
        line.starts_with(&needle)
            || line
                .split('|')
                .any(|case_id| case_id.trim_start().starts_with(&needle))
    })
}

#[derive(Debug)]
struct ManifestSummary {
    counts: FamilyCounts,
    execution_files: usize,
}

#[derive(Debug)]
struct FamilyCounts {
    total: usize,
    families: BTreeMap<String, usize>,
}

impl FamilyCounts {
    fn from_ids(ids: &BTreeSet<&str>) -> Self {
        let mut families = BTreeMap::new();
        for id in ids {
            *families.entry(family(id)).or_insert(0) += 1;
        }
        Self {
            total: ids.len(),
            families,
        }
    }

    fn render_families(&self) -> String {
        self.families
            .iter()
            .map(|(family, count)| format!("{family}={count}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn family(id: &str) -> String {
    if id.starts_with("ADAPT-DATA-") {
        "ADAPT-DATA".to_string()
    } else if id.starts_with("ADAPT-RUNTIME-") {
        "ADAPT-RUNTIME".to_string()
    } else if id.starts_with("C-BENCH-") {
        "C-BENCH".to_string()
    } else if id.starts_with("INT-") {
        "INT".to_string()
    } else if id.starts_with("SEC-") {
        "SEC".to_string()
    } else if id.starts_with("SWEPRO-") {
        "SWEPRO".to_string()
    } else if id.starts_with("TB-") {
        "TB".to_string()
    } else {
        id.to_string()
    }
}

fn inferred_test_count(route: &str) -> Option<u32> {
    let mut count = 0;
    for segment in route.split("run_filtered_tests").skip(1) {
        let command = segment.split(';').next().unwrap_or(segment);
        if let Some(value) = command
            .split(|character: char| !character.is_ascii_digit())
            .filter(|token| !token.is_empty())
            .filter_map(|token| token.parse::<u32>().ok())
            .next_back()
        {
            count += value;
        }
    }
    if count > 0 {
        return Some(count);
    }
    if route.contains("package=") && route.contains("test_name=") {
        return Some(1);
    }
    None
}

fn pass_threshold(route: &str) -> String {
    match inferred_test_count(route) {
        Some(1) => "selected test runs exactly once and passes".to_string(),
        Some(count) => format!("{count} selected tests run and pass"),
        None => "selector command exits successfully".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> RegistryDoc {
        RegistryDoc {
            schema_version: 1,
            tests: vec![TestEntry {
                id: "INT-001".to_string(),
                command: "scripts/test-after-change.sh --select INT-001".to_string(),
                file_patterns: vec!["xtask/Cargo.toml".to_string()],
                required_artifacts: Some(vec!["results.json".to_string()]),
                status: "active".to_string(),
                verifies: Verifies {
                    contracts: vec!["RunArtifacts".to_string()],
                },
            }],
        }
    }

    #[test]
    fn frozen_manifest_rejects_artifact_weakening() {
        let manifest = FrozenSelectorManifest {
            schema_version: 1,
            execution_files: frozen_execution_files::current_execution_files()
                .expect("execution files load"),
            selectors: vec![FrozenSelector {
                id: "INT-001".to_string(),
                status: "active".to_string(),
                command: "scripts/test-after-change.sh --select INT-001".to_string(),
                router_case: r#"INT-001) package="harnesslab-cli"; test_name="int_001" ;;"#
                    .to_string(),
                expected_test_count: Some(1),
                expected_pass_threshold: "selected test runs exactly once and passes".to_string(),
                file_patterns: vec!["xtask/Cargo.toml".to_string()],
                required_artifacts: vec![],
                owning_contracts: vec!["RunArtifacts".to_string()],
            }],
        };
        let router = r#"INT-001) package="harnesslab-cli"; test_name="int_001" ;;"#;
        let error = verify_manifest(&manifest, &registry(), router)
            .expect_err("artifact weakening should fail");
        assert!(error.to_string().contains("required_artifacts changed"));
    }

    #[test]
    fn frozen_manifest_requires_all_frozen_registry_ids() {
        let manifest = FrozenSelectorManifest {
            schema_version: 1,
            execution_files: frozen_execution_files::current_execution_files()
                .expect("execution files load"),
            selectors: vec![],
        };
        let error =
            verify_manifest(&manifest, &registry(), "").expect_err("missing id should fail");
        assert!(error.to_string().contains("required frozen selector"));
    }

    #[test]
    fn frozen_manifest_rejects_same_count_route_substitution() {
        let selector = FrozenSelector {
            id: "INT-001".to_string(),
            status: "active".to_string(),
            command: "scripts/test-after-change.sh --select INT-001".to_string(),
            router_case: r#"INT-001) package="harnesslab-cli"; test_name="int_001" ;;"#.to_string(),
            expected_test_count: Some(1),
            expected_pass_threshold: "selected test runs exactly once and passes".to_string(),
            file_patterns: vec!["xtask/Cargo.toml".to_string()],
            required_artifacts: vec!["results.json".to_string()],
            owning_contracts: vec!["RunArtifacts".to_string()],
        };
        let manifest = FrozenSelectorManifest {
            schema_version: 1,
            execution_files: frozen_execution_files::current_execution_files()
                .expect("execution files load"),
            selectors: vec![selector.clone()],
        };
        let route = r#"INT-001) package="harnesslab-cli"; test_name="different_test" ;;"#;
        assert!(
            verify_selector(&selector, &registry().tests[0], route).is_err(),
            "same-count route substitution should fail"
        );
        assert!(verify_manifest(&manifest, &registry(), route).is_err());
    }

    #[test]
    fn frozen_manifest_rejects_external_script_noop_substitution() {
        let selector = FrozenSelector {
            id: "PY-TB-001".to_string(),
            status: "active".to_string(),
            command: "scripts/test-after-change.sh --select PY-TB-001".to_string(),
            router_case: "PY-TB-001) exec scripts/verify-terminal-bench-python-adapter.sh ;;"
                .to_string(),
            expected_test_count: None,
            expected_pass_threshold: "selector command exits successfully".to_string(),
            file_patterns: vec!["xtask/Cargo.toml".to_string()],
            required_artifacts: vec![],
            owning_contracts: vec!["TerminalBenchExternalRunner".to_string()],
        };
        let test = TestEntry {
            id: "PY-TB-001".to_string(),
            command: "scripts/test-after-change.sh --select PY-TB-001".to_string(),
            file_patterns: vec!["xtask/Cargo.toml".to_string()],
            required_artifacts: Some(vec![]),
            status: "active".to_string(),
            verifies: Verifies {
                contracts: vec!["TerminalBenchExternalRunner".to_string()],
            },
        };
        let route = "PY-TB-001) true ;;";
        assert!(
            verify_selector(&selector, &test, route).is_err(),
            "external script no-op substitution should fail"
        );
    }

    #[test]
    fn route_count_sums_filtered_test_groups() {
        let route = r#"AGT-REG-012) run_filtered_tests "$id" "a" "lib" "x" 2; run_filtered_tests "$id" "b" "lib" "y" 4; exit 0 ;;"#;
        assert_eq!(inferred_test_count(route), Some(6));
    }
}
