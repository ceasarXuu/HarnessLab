use super::RegistryDoc;
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

const INT_011_ARTIFACT_CONTRACT: &str =
    "tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt";

pub(super) fn ensure_runtime_artifact_contracts(registry: &RegistryDoc) -> Result<()> {
    ensure_required_artifacts_are_executable_registry_contracts(registry)?;
    let expected = load_artifact_contract(INT_011_ARTIFACT_CONTRACT)?;
    ensure_int_011_artifacts_match_contract(registry, &expected)
}

fn ensure_required_artifacts_are_executable_registry_contracts(
    registry: &RegistryDoc,
) -> Result<()> {
    for test in &registry.tests {
        if matches!(test.status.as_str(), "deprecated") {
            continue;
        }
        let mut seen = BTreeSet::new();
        for artifact in test.required_artifacts.as_deref().unwrap_or(&[]) {
            let normalized = normalize_required_artifact_path(test.id.as_str(), artifact)?;
            if !seen.insert(normalized) {
                bail!(
                    "test {} has duplicate required_artifact: {artifact}",
                    test.id
                );
            }
        }
    }
    Ok(())
}

fn normalize_required_artifact_path(test_id: &str, artifact: &str) -> Result<String> {
    let raw = artifact;
    let artifact = raw.trim();
    if raw != artifact {
        bail!("test {test_id} required_artifact has surrounding whitespace: {raw}");
    }
    if artifact.is_empty() {
        bail!("test {test_id} has an empty required_artifact");
    }
    if artifact.contains('\\') {
        bail!("test {test_id} required_artifact uses backslashes: {artifact}");
    }
    let path = Path::new(artifact);
    if path.is_absolute() {
        bail!("test {test_id} required_artifact must be relative: {artifact}");
    }
    let mut normalized = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => {
                normalized.push(value.to_string_lossy().to_string());
            }
            Component::ParentDir => {
                bail!("test {test_id} required_artifact escapes run artifacts: {artifact}");
            }
            Component::CurDir => {
                bail!(
                    "test {test_id} required_artifact uses current-directory segments: {artifact}"
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("test {test_id} required_artifact must be relative: {artifact}");
            }
        }
    }
    if normalized.is_empty() {
        bail!("test {test_id} has an empty required_artifact");
    }
    let normalized = normalized.join("/");
    if normalized != artifact {
        bail!("test {test_id} required_artifact is not normalized: {artifact}");
    }
    Ok(normalized)
}

fn ensure_int_011_artifacts_match_contract(
    registry: &RegistryDoc,
    expected: &BTreeSet<String>,
) -> Result<()> {
    let test = registry
        .tests
        .iter()
        .find(|test| test.id == "INT-011")
        .ok_or_else(|| anyhow::anyhow!("INT-011 is missing from tests/TEST_REGISTRY.toml"))?;
    let actual = test
        .required_artifacts
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("INT-011 must declare runtime proof artifacts"))?
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
    let extra = actual.difference(expected).cloned().collect::<Vec<_>>();
    if !missing.is_empty() || !extra.is_empty() {
        bail!(
            "INT-011 required_artifacts must match asserted SWE-bench Pro runtime artifacts; missing [{}], extra [{}]",
            missing.join(", "),
            extra.join(", ")
        );
    }
    Ok(())
}

fn load_artifact_contract(path: &str) -> Result<BTreeSet<String>> {
    let content = fs::read_to_string(path).with_context(|| format!("read {path}"))?;
    Ok(parse_artifact_contract(&content))
}

fn parse_artifact_contract(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RegistryDoc, TestEntry, Verifies};

    #[test]
    fn runtime_artifacts_001_int_011_rejects_bare_attempt_artifact_paths() {
        let mut artifacts = test_contract().iter().cloned().collect::<Vec<_>>();
        replace_artifact(
            &mut artifacts,
            "tasks/<task-id>/attempts/1/patch.diff",
            "patch.diff",
        );
        let registry = registry_with_int_011(artifacts);

        let error =
            ensure_int_011_artifacts_match_contract(&registry, &test_contract()).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("tasks/<task-id>/attempts/1/patch.diff"));
        assert!(message.contains("extra [patch.diff]"));
    }

    #[test]
    fn runtime_artifacts_002_int_011_detects_contract_drift() {
        let artifacts = test_contract().iter().cloned().collect::<Vec<_>>();
        let registry = registry_with_int_011(artifacts);
        let changed_contract = parse_artifact_contract("run.json\nnew-runtime-artifact.json\n");

        let error =
            ensure_int_011_artifacts_match_contract(&registry, &changed_contract).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("new-runtime-artifact.json"));
        assert!(message.contains("extra ["));
    }

    #[test]
    fn runtime_artifacts_003_int_011_accepts_shared_artifact_contract() {
        let artifacts = test_contract().iter().cloned().collect::<Vec<_>>();
        let registry = registry_with_int_011(artifacts);

        ensure_int_011_artifacts_match_contract(&registry, &test_contract()).unwrap();
    }

    #[test]
    fn runtime_artifacts_004_required_artifacts_reject_unsafe_paths() {
        for artifact in [
            "",
            " results.json ",
            "/tmp/result.json",
            "../result.json",
            "./result.json",
            "a//b",
            "a/b/",
            "a\\b",
        ] {
            let registry = registry_with_int_011(vec![artifact.to_string()]);

            let error =
                ensure_required_artifacts_are_executable_registry_contracts(&registry).unwrap_err();

            assert!(
                error.to_string().contains("required_artifact"),
                "unexpected error for {artifact:?}: {error}"
            );
        }
    }

    #[test]
    fn runtime_artifacts_005_required_artifacts_reject_duplicates() {
        let registry =
            registry_with_int_011(vec!["results.json".to_string(), "results.json".to_string()]);

        let error =
            ensure_required_artifacts_are_executable_registry_contracts(&registry).unwrap_err();

        assert!(error.to_string().contains("duplicate required_artifact"));
    }

    #[test]
    fn runtime_artifacts_006_required_artifacts_reject_normalized_duplicate_variants() {
        for artifacts in [
            vec!["a/b".to_string(), "a//b".to_string()],
            vec!["a/b".to_string(), "a/b/".to_string()],
        ] {
            let registry = registry_with_int_011(artifacts);

            let error =
                ensure_required_artifacts_are_executable_registry_contracts(&registry).unwrap_err();

            assert!(
                error.to_string().contains("not normalized"),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn runtime_artifacts_007_required_artifacts_accept_relative_run_paths() {
        let registry = registry_with_int_011(vec![
            "results.json".to_string(),
            "tasks/<task-id>/attempts/1/external-runtime.public.json".to_string(),
        ]);

        ensure_required_artifacts_are_executable_registry_contracts(&registry).unwrap();
    }

    fn replace_artifact(artifacts: &mut [String], old: &str, new: &str) {
        let artifact = artifacts
            .iter_mut()
            .find(|artifact| artifact.as_str() == old)
            .expect("test artifact exists");
        *artifact = new.to_string();
    }

    fn registry_with_int_011(required_artifacts: Vec<String>) -> RegistryDoc {
        RegistryDoc {
            schema_version: 1,
            tests: vec![TestEntry {
                id: "INT-011".to_string(),
                title: "SWE-bench Pro external smoke and failure contracts".to_string(),
                command: "scripts/test-after-change.sh --select INT-011".to_string(),
                file_patterns: vec![
                    "crates/harnesslab-cli/tests/external_smoke_contract.rs".to_string(),
                ],
                required_artifacts: Some(required_artifacts),
                status: "active".to_string(),
                labels: None,
                verifies: Verifies {
                    requirements: vec!["benchmark_adapter_contract".to_string()],
                    contracts: vec!["BenchmarkAdapter".to_string()],
                },
            }],
        }
    }

    fn test_contract() -> BTreeSet<String> {
        parse_artifact_contract(include_str!(
            "../../tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt"
        ))
    }
}
