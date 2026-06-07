use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod adapter_claims;
mod coverage;
mod frozen_execution_files;
mod frozen_selector_ids;
mod frozen_selectors;
mod runtime_artifacts;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::VerifyTestRegistry => verify_test_registry(),
        Command::VerifyFrozenSelectorManifest => {
            frozen_selectors::verify_frozen_selector_manifest()
        }
        Command::PrintFrozenSelectorManifest => frozen_selectors::print_frozen_selector_manifest(),
        Command::GenerateTestTraceability => generate_traceability(),
        Command::ListAdapterProofSelectors => list_adapter_proof_selectors(),
        Command::ScanSecrets { secret, paths } => scan_secrets(&secret, &paths),
        Command::CheckNewFileCoverage {
            lcov,
            min_line,
            base,
        } => {
            let count = coverage::check_new_file_coverage_file(&lcov, min_line, base.as_deref())?;
            if count == 0 {
                println!("new-file coverage ok: no new production Rust files detected");
            } else {
                println!(
                    "new-file coverage ok: {count} new production Rust files are present in coverage data"
                );
            }
            Ok(())
        }
        Command::CheckCoverage {
            lcov,
            config,
            min_line,
            min_branch,
        } => {
            let summary = coverage::check_lcov_file(&lcov, &config, min_line, min_branch)?;
            let line_percent =
                coverage::percent(summary.global.lines_hit, summary.global.lines_found);
            let branch_percent =
                coverage::percent(summary.global.branches_hit, summary.global.branches_found);
            println!(
                "coverage ok: lines {line_percent:.2}% ({}/{}), branches {branch_percent:.2}% ({}/{}), modules {}",
                summary.global.lines_hit,
                summary.global.lines_found,
                summary.global.branches_hit,
                summary.global.branches_found,
                summary.module_count
            );
            Ok(())
        }
    }
}

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    VerifyTestRegistry,
    VerifyFrozenSelectorManifest,
    PrintFrozenSelectorManifest,
    GenerateTestTraceability,
    ListAdapterProofSelectors,
    ScanSecrets {
        #[arg(long)]
        secret: String,
        #[arg(long = "path")]
        paths: Vec<PathBuf>,
    },
    CheckNewFileCoverage {
        #[arg(long, default_value = "coverage/lcov.info")]
        lcov: PathBuf,
        #[arg(long, default_value_t = 0.0)]
        min_line: f64,
        #[arg(long)]
        base: Option<String>,
    },
    CheckCoverage {
        #[arg(long)]
        lcov: PathBuf,
        #[arg(long, default_value = "coverage-critical.toml")]
        config: PathBuf,
        #[arg(long)]
        min_line: f64,
        #[arg(long)]
        min_branch: f64,
    },
}

#[derive(Deserialize)]
struct RequirementDoc {
    schema_version: u32,
    requirements: Vec<Requirement>,
}

#[derive(Deserialize)]
struct Requirement {
    id: String,
    title: String,
    source: Source,
    risk: String,
    status: Option<String>,
    required_runtime_proof: bool,
}

#[derive(Deserialize)]
struct RegistryDoc {
    schema_version: u32,
    tests: Vec<TestEntry>,
}

#[derive(Deserialize)]
struct TestEntry {
    id: String,
    title: String,
    command: String,
    file_patterns: Vec<String>,
    required_artifacts: Option<Vec<String>>,
    status: String,
    labels: Option<Vec<String>>,
    verifies: Verifies,
}

#[derive(Deserialize)]
struct Verifies {
    requirements: Vec<String>,
    contracts: Vec<String>,
}

#[derive(Deserialize)]
struct Source {
    doc: String,
    section: String,
}

#[derive(Serialize)]
struct TraceabilityDoc {
    schema_version: u32,
    requirements: Vec<TraceabilityRequirement>,
}

#[derive(Serialize)]
struct TraceabilityRequirement {
    id: String,
    title: String,
    source: String,
    risk: String,
    test_ids: Vec<String>,
    runtime_proof: Vec<String>,
    status: String,
}

fn verify_test_registry() -> Result<()> {
    let requirements = load_requirements()?;
    let registry = load_registry()?;
    ensure_schema(requirements.schema_version, "tests/REQUIREMENTS.toml")?;
    ensure_schema(registry.schema_version, "tests/TEST_REGISTRY.toml")?;
    let claim_sources = adapter_claims::load_adapter_claim_sources()?;
    let claimed_adapter_ids =
        adapter_claims::claimed_adapter_test_ids_from_sources(&claim_sources)?;

    let mut requirement_ids = BTreeSet::new();
    let mut active_requirement_ids = BTreeSet::new();
    for requirement in &requirements.requirements {
        if !requirement_ids.insert(requirement.id.clone()) {
            bail!("duplicate requirement id: {}", requirement.id);
        }
        match requirement_status(requirement) {
            "active" => {
                active_requirement_ids.insert(requirement.id.clone());
            }
            "planned"
                if adapter_claims::planned_status_allowed(
                    &requirement.id,
                    &claimed_adapter_ids,
                ) => {}
            "planned" => bail!(
                "planned requirement status is only allowed for claimed adapter proof ids: {}",
                requirement.id
            ),
            status => bail!("requirement {} has unknown status {status}", requirement.id),
        }
    }

    let mut test_ids = BTreeSet::new();
    let mut covered_requirements = BTreeSet::new();
    for test in &registry.tests {
        if !test_ids.insert(test.id.clone()) {
            bail!("duplicate test id: {}", test.id);
        }
        if !matches!(test.status.as_str(), "active" | "planned" | "deprecated") {
            bail!("test {} has unknown status {}", test.id, test.status);
        }
        if test.status == "planned"
            && !adapter_claims::planned_status_allowed(&test.id, &claimed_adapter_ids)
        {
            bail!(
                "planned test status is only allowed for claimed adapter proof ids: {}",
                test.id
            );
        }
        if test.command.trim().is_empty() {
            bail!("test {} has an empty command", test.id);
        }
        if test.status == "active" || test.status == "planned" {
            ensure_patterns_match(&test.file_patterns)
                .with_context(|| format!("test {} has missing file pattern", test.id))?;
        }
        if test.title.trim().is_empty() {
            bail!("test {} has an empty title", test.id);
        }
        if test.verifies.contracts.is_empty() {
            bail!("test {} has no verified contract", test.id);
        }
        for requirement in &test.verifies.requirements {
            if !requirement_ids.contains(requirement) {
                bail!(
                    "test {} references unknown requirement {}",
                    test.id,
                    requirement
                );
            }
            if test.status == "active" {
                covered_requirements.insert(requirement.clone());
            }
        }
        if test.verifies.requirements.is_empty() && !is_infrastructure_or_regression(test) {
            bail!("test {} has no requirement and no allowed label", test.id);
        }
    }

    for requirement in &requirements.requirements {
        if !active_requirement_ids.contains(&requirement.id) {
            continue;
        }
        if !covered_requirements.contains(&requirement.id) {
            bail!("requirement has no active test: {}", requirement.id);
        }
        if requirement.required_runtime_proof {
            let has_artifact = registry.tests.iter().any(|test| {
                test.status == "active"
                    && test.verifies.requirements.contains(&requirement.id)
                    && test
                        .required_artifacts
                        .as_ref()
                        .is_some_and(|artifacts| !artifacts.is_empty())
            });
            if !has_artifact {
                bail!(
                    "requirement lacks runtime proof artifacts: {}",
                    requirement.id
                );
            }
        }
    }
    let selector_script =
        fs::read_to_string("scripts/test-after-change.sh").context("read test selector script")?;
    adapter_claims::ensure_claimed_adapter_ids_are_registered(
        &claimed_adapter_ids,
        &requirement_ids,
        &test_ids,
        &registry,
        &selector_script,
    )?;
    runtime_artifacts::ensure_runtime_artifact_contracts(&registry)?;
    adapter_claims::write_adapter_proof_inventory(
        &claimed_adapter_ids,
        &claim_sources,
        &registry,
        &selector_script,
    )?;

    println!(
        "registry ok: {} requirements, {} tests",
        requirement_ids.len(),
        test_ids.len()
    );
    println!(
        "adapter proof claims ok: {} ids from {} sources",
        claimed_adapter_ids.len(),
        claim_sources.len()
    );
    Ok(())
}

fn requirement_status(requirement: &Requirement) -> &str {
    requirement.status.as_deref().unwrap_or("active")
}

fn list_adapter_proof_selectors() -> Result<()> {
    let registry = load_registry()?;
    ensure_schema(registry.schema_version, "tests/TEST_REGISTRY.toml")?;
    let claim_sources = adapter_claims::load_adapter_claim_sources()?;
    let claimed_adapter_ids =
        adapter_claims::claimed_adapter_test_ids_from_sources(&claim_sources)?;
    for id in claimed_adapter_ids {
        let test = registry
            .tests
            .iter()
            .find(|test| test.id == id)
            .with_context(|| format!("{id} is missing from tests/TEST_REGISTRY.toml"))?;
        println!("{}\t{}", test.status, id);
    }
    Ok(())
}

fn generate_traceability() -> Result<()> {
    let requirements = load_requirements()?;
    let registry = load_registry()?;
    let mut by_requirement: BTreeMap<String, Vec<&TestEntry>> = BTreeMap::new();
    for test in &registry.tests {
        if test.status == "deprecated" {
            continue;
        }
        for requirement in &test.verifies.requirements {
            by_requirement
                .entry(requirement.clone())
                .or_default()
                .push(test);
        }
    }

    let mut rows = Vec::new();
    for requirement in requirements.requirements {
        let status = if requirement_status(&requirement) == "planned" {
            "planned"
        } else if by_requirement.contains_key(&requirement.id) {
            "covered"
        } else {
            "missing"
        }
        .to_string();
        let tests = by_requirement.remove(&requirement.id).unwrap_or_default();
        let mut test_ids: Vec<String> = tests.iter().map(|test| test.id.clone()).collect();
        test_ids.sort();
        let mut runtime_proof: Vec<String> = tests
            .iter()
            .flat_map(|test| test.required_artifacts.clone().unwrap_or_default())
            .collect();
        runtime_proof.sort();
        rows.push(TraceabilityRequirement {
            id: requirement.id,
            title: requirement.title,
            source: format!("{}#{}", requirement.source.doc, requirement.source.section),
            risk: requirement.risk,
            status,
            test_ids,
            runtime_proof,
        });
    }
    rows.sort_by(|left, right| left.id.cmp(&right.id));

    let doc = TraceabilityDoc {
        schema_version: 1,
        requirements: rows,
    };
    fs::create_dir_all("artifacts")?;
    fs::write(
        "artifacts/test-traceability.json",
        serde_json::to_string_pretty(&doc)?,
    )?;
    fs::write(
        "artifacts/test-traceability.md",
        render_traceability_markdown(&doc),
    )?;
    println!("traceability generated: artifacts/test-traceability.json");
    Ok(())
}

fn scan_secrets(secret: &str, paths: &[PathBuf]) -> Result<()> {
    if secret.is_empty() {
        bail!("--secret must not be empty");
    }
    let scan_roots = if paths.is_empty() {
        vec![PathBuf::from("artifacts"), PathBuf::from("coverage")]
    } else {
        paths.to_vec()
    };
    let mut leaks = Vec::new();
    for root in scan_roots {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() || !is_scannable(entry.path()) {
                continue;
            }
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains(secret) {
                leaks.push(entry.path().display().to_string());
            }
        }
    }
    if !leaks.is_empty() {
        bail!("secret leak detected in {}", leaks.join(", "));
    }
    println!("secret scan ok");
    Ok(())
}

fn load_requirements() -> Result<RequirementDoc> {
    let content =
        fs::read_to_string("tests/REQUIREMENTS.toml").context("read tests/REQUIREMENTS.toml")?;
    toml::from_str(&content).context("parse tests/REQUIREMENTS.toml")
}

fn load_registry() -> Result<RegistryDoc> {
    let content =
        fs::read_to_string("tests/TEST_REGISTRY.toml").context("read tests/TEST_REGISTRY.toml")?;
    toml::from_str(&content).context("parse tests/TEST_REGISTRY.toml")
}

fn ensure_schema(schema_version: u32, path: &str) -> Result<()> {
    if schema_version != 1 {
        bail!("{path} has unsupported schema_version {schema_version}");
    }
    Ok(())
}

fn ensure_patterns_match(patterns: &[String]) -> Result<()> {
    for pattern in patterns {
        if !Path::new(pattern).exists() {
            bail!("missing path: {pattern}");
        }
    }
    Ok(())
}

fn is_infrastructure_or_regression(test: &TestEntry) -> bool {
    test.labels.as_ref().is_some_and(|labels| {
        labels
            .iter()
            .any(|label| label == "infrastructure" || label == "regression")
    })
}

fn render_traceability_markdown(doc: &TraceabilityDoc) -> String {
    let mut output =
        String::from("| Requirement | Tests | Runtime Proof | Status |\n|---|---|---|---|\n");
    for requirement in &doc.requirements {
        output.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            requirement.id,
            requirement.test_ids.join(", "),
            requirement.runtime_proof.join(", "),
            requirement.status
        ));
    }
    output
}

fn is_scannable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("json" | "jsonl" | "html" | "toml" | "txt" | "md" | "xml")
    )
}
