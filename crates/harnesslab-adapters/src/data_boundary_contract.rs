use crate::data_boundary_rule_sets::{
    allowed_std_fs_calls, artifact_declaration_source, forbidden_runtime_path_literals,
    strip_artifact_declaration_calls,
};
use crate::data_boundary_scan::{
    assert_boundary_scanner_regressions, call_names, has_path_attribute, identifier_tokens,
    path_sequences, qualified_call_paths, string_literals, strip_cfg_test_items,
    strip_comments_and_strings, use_paths,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

struct BoundarySource {
    path: String,
    source: String,
}

pub(crate) fn assert_data_adapter_boundary_contract() {
    assert_boundary_scanner_regressions();
    let sources = boundary_sources();
    assert_dependency_boundary();
    assert_documented_boundary_artifact(&sources);

    for source in &sources {
        let production_source = strip_cfg_test_items(&source.source);
        let stripped = strip_comments_and_strings(&production_source);
        assert_forbidden_imports(&source.path, &stripped);
        assert_forbidden_symbols(&source.path, &stripped);
        assert_forbidden_calls(&source.path, &stripped);
        assert_forbidden_paths(&source.path, &stripped);
        assert_allowed_std_fs_calls(&source.path, &stripped);
        assert_no_path_attribute_modules(&source.path, &stripped);
        assert_no_inline_production_modules(&source.path, &stripped);
        assert_no_production_source_inclusion(&source.path, &stripped);
        assert_forbidden_path_literals(&source.path, &production_source);
    }
}

fn boundary_sources() -> Vec<BoundarySource> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut seen = BTreeSet::new();
    let mut sources = Vec::new();

    collect_module_sources(&src_dir.join("lib.rs"), &src_dir, &mut seen, &mut sources);
    sources.sort_by(|left, right| left.path.cmp(&right.path));

    sources
}

fn collect_module_sources(
    path: &Path,
    src_dir: &Path,
    seen: &mut BTreeSet<PathBuf>,
    sources: &mut Vec<BoundarySource>,
) {
    let canonical = path
        .canonicalize()
        .unwrap_or_else(|error| panic!("failed to canonicalize {}: {error}", path.display()));
    if !seen.insert(canonical.clone()) {
        return;
    }

    let source = std::fs::read_to_string(&canonical)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", canonical.display()));
    let relative_path = canonical
        .strip_prefix(src_dir)
        .unwrap_or(&canonical)
        .display()
        .to_string();
    let child_dir = child_module_dir(&canonical, src_dir);
    let modules = declared_file_modules(&source);
    sources.push(BoundarySource {
        path: format!("crates/harnesslab-adapters/src/{relative_path}"),
        source,
    });

    for module in modules {
        let child = resolve_module_file(&child_dir, &module);
        collect_module_sources(&child, src_dir, seen, sources);
    }
}

fn child_module_dir(path: &Path, src_dir: &Path) -> PathBuf {
    if path.file_name().and_then(|name| name.to_str()) == Some("lib.rs") {
        return src_dir.to_path_buf();
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
        return path.parent().unwrap_or(src_dir).to_path_buf();
    }
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| panic!("module source has no file stem: {}", path.display()));
    path.parent().unwrap_or(src_dir).join(stem)
}

fn declared_file_modules(source: &str) -> Vec<String> {
    let mut modules = Vec::new();
    let mut cfg_test_next = false;

    for raw_line in source.lines() {
        let line = raw_line.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("#[cfg(test")
            || line.starts_with("#[cfg(any(test")
            || line.starts_with("#[cfg(all(test")
        {
            cfg_test_next = true;
            continue;
        }
        if !line.ends_with(';') {
            if !line.starts_with("#[") {
                cfg_test_next = false;
            }
            continue;
        }
        let without_semicolon = line.trim_end_matches(';').trim();
        let parts = without_semicolon.split_whitespace().collect::<Vec<_>>();
        let Some(index) = parts.iter().position(|part| *part == "mod") else {
            if !line.starts_with("#[") {
                cfg_test_next = false;
            }
            continue;
        };
        if cfg_test_next {
            cfg_test_next = false;
            continue;
        }
        let Some(module_name) = parts.get(index + 1) else {
            continue;
        };
        modules.push(
            module_name
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                .to_string(),
        );
        cfg_test_next = false;
    }

    modules
}

fn resolve_module_file(module_dir: &Path, module: &str) -> PathBuf {
    let direct = module_dir.join(format!("{module}.rs"));
    if direct.exists() {
        return direct;
    }
    let mod_rs = module_dir.join(module).join("mod.rs");
    if mod_rs.exists() {
        return mod_rs;
    }
    panic!(
        "declared production module {module} has no source file under {}",
        module_dir.display()
    );
}

fn assert_dependency_boundary() {
    let allowed = allowed_production_dependencies();
    let actual = dependency_specs(include_str!("../Cargo.toml"));

    assert_eq!(
        actual, allowed,
        "harnesslab-adapters production dependencies must stay pure data/core only; renamed package aliases are not allowed"
    );
}

fn assert_documented_boundary_artifact(sources: &[BoundarySource]) {
    let artifact = include_str!(
        "../../../docs/archive/2026-06-15-pre-harbor-webui-redesign/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md"
    );
    for required in [
        "Allowed production dependencies",
        "Forbidden imports",
        "Forbidden runtime symbols",
        "Forbidden runtime calls",
        "Allowed std::fs read calls",
        "Forbidden runtime path literals",
        "Forbidden module path attributes",
        "Module graph coverage",
        "Covered source files",
    ] {
        assert!(
            artifact.contains(required),
            "boundary artifact missing section: {required}"
        );
    }
    for (alias, package) in allowed_production_dependencies() {
        assert!(
            artifact.contains(&alias) && artifact.contains(&package),
            "boundary artifact missing allowed dependency {alias} -> {package}"
        );
    }
    for import in forbidden_import_roots() {
        assert!(
            artifact.contains(import),
            "boundary artifact missing forbidden import {import}"
        );
    }
    for symbol in forbidden_symbols() {
        assert!(
            artifact.contains(symbol),
            "boundary artifact missing forbidden runtime symbol {symbol}"
        );
    }
    for call in forbidden_calls() {
        assert!(
            artifact.contains(call),
            "boundary artifact missing forbidden runtime call {call}"
        );
    }
    for call in allowed_std_fs_calls() {
        assert!(
            artifact.contains(call),
            "boundary artifact missing allowed std::fs call {call}"
        );
    }
    for literal in forbidden_runtime_path_literals() {
        assert!(
            artifact.contains(literal),
            "boundary artifact missing forbidden runtime path literal {literal}"
        );
    }
    for source in sources {
        assert!(
            artifact.contains(&source.path),
            "boundary artifact missing covered source file {}",
            source.path
        );
    }
}

fn allowed_production_dependencies() -> BTreeSet<(String, String)> {
    BTreeSet::from([
        ("harnesslab-core".to_string(), "harnesslab-core".to_string()),
        ("serde".to_string(), "serde".to_string()),
        ("serde_json".to_string(), "serde_json".to_string()),
    ])
}

fn dependency_specs(cargo_toml: &str) -> BTreeSet<(String, String)> {
    let mut in_dependencies = false;
    let mut specs = BTreeSet::new();

    for raw_line in cargo_toml.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_dependencies = is_production_dependency_section(line);
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let Some((key, _value)) = line.split_once('=') else {
            continue;
        };
        let alias = key.trim().split('.').next().unwrap_or("").trim();
        if !alias.is_empty() {
            specs.insert((alias.to_string(), package_name(alias, line)));
        }
    }

    specs
}

fn assert_forbidden_imports(path: &str, source: &str) {
    let imports = use_paths(source);

    for import in imports {
        for forbidden in forbidden_import_roots() {
            let forbidden_runtime_import = import.starts_with(forbidden)
                || import.starts_with("std::") && import.contains("process")
                || import.starts_with("tokio::") && import.contains("process");
            assert!(
                !forbidden_runtime_import,
                "{path} imports forbidden runtime boundary path {import}"
            );
        }
    }
}

fn assert_forbidden_symbols(path: &str, source: &str) {
    let symbols = identifier_tokens(source);

    for symbol in forbidden_symbols() {
        assert!(
            !symbols.contains(symbol),
            "{path} references forbidden runtime boundary symbol {symbol}"
        );
    }
}

fn assert_forbidden_calls(path: &str, source: &str) {
    let calls = call_names(source);

    for call in forbidden_calls() {
        assert!(
            !calls.contains(call),
            "{path} calls forbidden runtime boundary function {call}"
        );
    }
}

fn assert_forbidden_paths(path: &str, source: &str) {
    for path_ref in path_sequences(source) {
        for forbidden in forbidden_import_roots() {
            assert!(
                !path_ref.starts_with(forbidden),
                "{path} references forbidden runtime boundary path {path_ref}"
            );
        }
    }
}

fn assert_allowed_std_fs_calls(path: &str, source: &str) {
    let allowed = allowed_std_fs_calls();
    for call in qualified_call_paths(source) {
        let Some(function) = call.strip_prefix("std::fs::") else {
            continue;
        };
        assert!(
            allowed.contains(function),
            "{path} calls std::fs::{function}; data adapters may only use the documented std::fs read allowlist"
        );
    }
}

fn assert_no_path_attribute_modules(path: &str, source: &str) {
    assert!(
        !has_path_attribute(source),
        "{path} uses #[path] in production source; use normal file modules so boundary scanning can verify compiled helpers"
    );
}

fn assert_no_inline_production_modules(path: &str, source: &str) {
    let tokens = source.split_whitespace().collect::<Vec<_>>();
    for window in tokens.windows(3) {
        if window[0].starts_with("mod") || window[0] == "pub" {
            let joined = window.join(" ");
            assert!(
                !(joined.contains("mod ") && joined.contains('{')),
                "{path} declares an inline production module; use a file module so boundary scanning can verify it"
            );
        }
    }
}

fn assert_no_production_source_inclusion(path: &str, source: &str) {
    let calls = call_names(source);
    assert!(
        !calls.contains("include") && !source.contains("include!") && !source.contains("include !"),
        "{path} includes production source through include!; use a file module so boundary scanning can verify it"
    );
}

fn assert_forbidden_path_literals(path: &str, source: &str) {
    let declaration_stripped;
    let source = if artifact_declaration_source(path) {
        declaration_stripped = strip_artifact_declaration_calls(source);
        declaration_stripped.as_str()
    } else {
        source
    };
    let literals = string_literals(source);

    for literal in literals {
        for token in forbidden_runtime_path_literals() {
            assert!(
                !literal.contains(token),
                "{path} contains forbidden runtime path literal {token}"
            );
        }
    }
}

fn package_name(alias: &str, dependency_line: &str) -> String {
    let Some(package_index) = dependency_line.find("package") else {
        return alias.to_string();
    };
    let package_clause = &dependency_line[package_index..];
    let Some((_key, value)) = package_clause.split_once('=') else {
        return alias.to_string();
    };
    value
        .trim()
        .trim_start_matches('"')
        .split('"')
        .next()
        .filter(|package| !package.is_empty())
        .unwrap_or(alias)
        .to_string()
}

fn is_production_dependency_section(line: &str) -> bool {
    line == "[dependencies]" || (line.starts_with("[target.") && line.ends_with(".dependencies]"))
}

fn forbidden_import_roots() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "std::env",
        "std::process",
        "tokio::process",
        "harnesslab_cli",
        "harnesslab_infra",
    ])
}

fn forbidden_symbols() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "AttemptDir",
        "ArtifactWriter",
        "Command",
        "DockerRunner",
        "EventWriter",
        "ExecSpec",
        "File",
        "HostProcessExecutor",
        "OpenOptions",
        "ProcessExecutor",
        "RunDir",
        "harnesslab_infra",
    ])
}

fn forbidden_calls() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "args",
        "args_os",
        "abort",
        "copy",
        "create",
        "create_new",
        "create_dir",
        "create_dir_all",
        "current_dir",
        "current_exe",
        "env",
        "exec",
        "exec_with",
        "exit",
        "hard_link",
        "id",
        "output",
        "remove_dir",
        "remove_dir_all",
        "remove_file",
        "remove_var",
        "rename",
        "run_command",
        "set_len",
        "set_current_dir",
        "set_permissions",
        "set_times",
        "set_var",
        "spawn",
        "status",
        "symlink",
        "symlink_dir",
        "symlink_file",
        "temp_dir",
        "var",
        "var_os",
        "vars",
        "vars_os",
        "write_all",
        "write_all_vectored",
        "write",
        "write_event",
    ])
}
