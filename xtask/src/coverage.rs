use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageStats {
    pub lines_found: u64,
    pub lines_hit: u64,
    pub branches_found: u64,
    pub branches_hit: u64,
}

impl CoverageStats {
    fn add(&mut self, other: &CoverageStats) {
        self.lines_found += other.lines_found;
        self.lines_hit += other.lines_hit;
        self.branches_found += other.branches_found;
        self.branches_hit += other.branches_hit;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageRecord {
    pub source_file: PathBuf,
    pub stats: CoverageStats,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageThreshold {
    pub line: f64,
    pub branch: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleThreshold {
    pub name: String,
    pub path: PathBuf,
    pub threshold: CoverageThreshold,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoverageGateSummary {
    pub global: CoverageStats,
    pub module_count: usize,
}

#[derive(Deserialize)]
struct CoverageConfigDoc {
    schema_version: u32,
    #[serde(default)]
    modules: Vec<CoverageConfigModule>,
}

#[derive(Deserialize)]
struct CoverageConfigModule {
    name: String,
    path: PathBuf,
    line: f64,
    branch: f64,
}

pub fn check_lcov_file(
    lcov: &Path,
    config: &Path,
    min_line: f64,
    min_branch: f64,
) -> Result<CoverageGateSummary> {
    let content = fs::read_to_string(lcov)
        .with_context(|| format!("read coverage report {}", lcov.display()))?;
    let records = parse_lcov(&content)?;
    let modules = load_module_thresholds(config)?;
    let workspace_root = std::env::current_dir().context("resolve workspace root")?;
    assert_thresholds(
        &records,
        CoverageThreshold {
            line: min_line,
            branch: min_branch,
        },
        &modules,
        &workspace_root,
    )
}

pub fn check_new_file_coverage_file(
    lcov: &Path,
    min_line: f64,
    base: Option<&str>,
) -> Result<usize> {
    let content = fs::read_to_string(lcov)
        .with_context(|| format!("read coverage report {}", lcov.display()))?;
    let records = parse_lcov(&content)?;
    let workspace_root = std::env::current_dir().context("resolve workspace root")?;
    let new_files = discover_new_production_files(base)?;
    assert_new_file_coverage(&records, &new_files, min_line, &workspace_root)?;
    Ok(new_files.len())
}

pub fn parse_lcov(content: &str) -> Result<Vec<CoverageRecord>> {
    let mut records = Vec::new();
    let mut source_file: Option<PathBuf> = None;
    let mut stats = CoverageStats {
        lines_found: 0,
        lines_hit: 0,
        branches_found: 0,
        branches_hit: 0,
    };

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("SF:") {
            if let Some(path) = source_file.take() {
                records.push(CoverageRecord {
                    source_file: path,
                    stats: stats.clone(),
                });
                reset_stats(&mut stats);
            }
            source_file = Some(PathBuf::from(value));
        } else if let Some(value) = line.strip_prefix("LF:") {
            stats.lines_found = parse_counter(value, "LF")?;
        } else if let Some(value) = line.strip_prefix("LH:") {
            stats.lines_hit = parse_counter(value, "LH")?;
        } else if let Some(value) = line.strip_prefix("BRF:") {
            stats.branches_found = parse_counter(value, "BRF")?;
        } else if let Some(value) = line.strip_prefix("BRH:") {
            stats.branches_hit = parse_counter(value, "BRH")?;
        } else if line == "end_of_record"
            && let Some(path) = source_file.take()
        {
            records.push(CoverageRecord {
                source_file: path,
                stats: stats.clone(),
            });
            reset_stats(&mut stats);
        }
    }

    if let Some(path) = source_file {
        records.push(CoverageRecord {
            source_file: path,
            stats,
        });
    }
    Ok(records)
}

pub fn assert_thresholds(
    records: &[CoverageRecord],
    global: CoverageThreshold,
    modules: &[ModuleThreshold],
    workspace_root: &Path,
) -> Result<CoverageGateSummary> {
    let global_stats = aggregate(records.iter().map(|record| &record.stats));
    assert_stats("global", &global_stats, &global)?;

    for module in modules {
        let module_root = workspace_root.join(&module.path);
        let module_stats = aggregate(
            records
                .iter()
                .filter(|record| path_starts_with(&record.source_file, &module_root))
                .map(|record| &record.stats),
        );
        assert_stats(
            &format!("module {}", module.name),
            &module_stats,
            &module.threshold,
        )?;
    }

    Ok(CoverageGateSummary {
        global: global_stats,
        module_count: modules.len(),
    })
}

pub fn assert_new_file_coverage(
    records: &[CoverageRecord],
    new_files: &[PathBuf],
    min_line: f64,
    workspace_root: &Path,
) -> Result<()> {
    for file in new_files
        .iter()
        .filter(|path| is_production_source_file(path))
    {
        let absolute_file = if file.is_absolute() {
            file.clone()
        } else {
            workspace_root.join(file)
        };
        let Some(record) = records
            .iter()
            .find(|record| same_path(&record.source_file, &absolute_file))
        else {
            bail!(
                "missing coverage for new production file {}",
                file.display()
            );
        };
        let line_percent = percent(record.stats.lines_hit, record.stats.lines_found);
        if line_percent < min_line {
            bail!(
                "new production file {} line coverage {line_percent:.2}% is below required {min_line:.2}%",
                file.display()
            );
        }
    }
    Ok(())
}

fn load_module_thresholds(config: &Path) -> Result<Vec<ModuleThreshold>> {
    let content = fs::read_to_string(config)
        .with_context(|| format!("read coverage config {}", config.display()))?;
    let doc: CoverageConfigDoc = toml::from_str(&content)
        .with_context(|| format!("parse coverage config {}", config.display()))?;
    if doc.schema_version != 1 {
        bail!(
            "{} has unsupported schema_version {}",
            config.display(),
            doc.schema_version
        );
    }
    Ok(doc
        .modules
        .into_iter()
        .map(|module| ModuleThreshold {
            name: module.name,
            path: module.path,
            threshold: CoverageThreshold {
                line: module.line,
                branch: module.branch,
            },
        })
        .collect())
}

fn discover_new_production_files(base: Option<&str>) -> Result<Vec<PathBuf>> {
    let mut files = BTreeSet::new();
    if let Some(base_ref) = base.map(str::to_string).or_else(discover_default_base) {
        insert_git_files(
            &mut files,
            &[
                "diff",
                "--name-only",
                "--diff-filter=A",
                &format!("{base_ref}...HEAD"),
            ],
        )?;
    }
    insert_git_files(&mut files, &["diff", "--name-only", "--diff-filter=A"])?;
    insert_git_files(
        &mut files,
        &["diff", "--name-only", "--diff-filter=A", "--cached"],
    )?;
    insert_git_files(&mut files, &["ls-files", "--others", "--exclude-standard"])?;

    Ok(files
        .into_iter()
        .filter(|path| is_production_source_file(path))
        .collect())
}

fn discover_default_base() -> Option<String> {
    run_git_text(&["merge-base", "HEAD", "origin/main"])
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            run_git_text(&["merge-base", "HEAD", "main"])
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .map(|value| value.trim().to_string())
}

fn insert_git_files(files: &mut BTreeSet<PathBuf>, args: &[&str]) -> Result<()> {
    let output = run_git_text(args)?;
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        files.insert(PathBuf::from(line));
    }
    Ok(())
}

fn run_git_text(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn assert_stats(label: &str, stats: &CoverageStats, threshold: &CoverageThreshold) -> Result<()> {
    if stats.lines_found == 0 {
        bail!("{label} coverage has no line data");
    }

    let line_percent = percent(stats.lines_hit, stats.lines_found);
    if line_percent < threshold.line {
        bail!(
            "{label} line coverage {line_percent:.2}% is below required {:.2}%",
            threshold.line
        );
    }

    if threshold.branch > 0.0 && stats.branches_found == 0 {
        bail!("{label} branch coverage has no branch data");
    }
    let branch_percent = percent(stats.branches_hit, stats.branches_found);
    if branch_percent < threshold.branch {
        bail!(
            "{label} branch coverage {branch_percent:.2}% is below required {:.2}%",
            threshold.branch
        );
    }
    Ok(())
}

fn aggregate<'a>(stats: impl Iterator<Item = &'a CoverageStats>) -> CoverageStats {
    let mut total = CoverageStats {
        lines_found: 0,
        lines_hit: 0,
        branches_found: 0,
        branches_hit: 0,
    };
    for item in stats {
        total.add(item);
    }
    total
}

fn reset_stats(stats: &mut CoverageStats) {
    stats.lines_found = 0;
    stats.lines_hit = 0;
    stats.branches_found = 0;
    stats.branches_hit = 0;
}

fn parse_counter(value: &str, label: &str) -> Result<u64> {
    value
        .parse::<u64>()
        .with_context(|| format!("parse {label} counter {value}"))
}

fn is_production_source_file(path: &Path) -> bool {
    let normalized = path.to_string_lossy();
    normalized.starts_with("crates/")
        && normalized.contains("/src/")
        && path.extension().and_then(|extension| extension.to_str()) == Some("rs")
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    same_or_normalized(path).starts_with(&same_or_normalized(prefix))
}

fn same_path(left: &Path, right: &Path) -> bool {
    same_or_normalized(left) == same_or_normalized(right)
}

fn same_or_normalized(path: &Path) -> String {
    path.components()
        .collect::<PathBuf>()
        .to_string_lossy()
        .to_string()
}

pub fn percent(hit: u64, found: u64) -> f64 {
    if found == 0 {
        100.0
    } else {
        (hit as f64 / found as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
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
            Path::new("/repo"),
        )
        .unwrap_err()
        .to_string();

        assert!(error.contains("missing coverage for new production file"));
    }
}
