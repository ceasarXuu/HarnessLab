use std::collections::BTreeSet;

pub(crate) fn use_paths(source: &str) -> BTreeSet<String> {
    use_statements(source)
        .into_iter()
        .flat_map(|statement| expand_use_item("", &statement))
        .collect()
}

pub(crate) fn path_sequences(source: &str) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut index = 0;

    while let Some((_start, end, path)) = next_path(source, index) {
        if path.contains("::") {
            paths.insert(path);
        }
        index = end;
    }

    paths
}

pub(crate) fn qualified_call_paths(source: &str) -> BTreeSet<String> {
    let mut calls = BTreeSet::new();
    let mut index = 0;

    while let Some((_start, end, path)) = next_path(source, index) {
        let cursor = skip_whitespace(source, end);
        if path.contains("::") && source[cursor..].starts_with('(') {
            calls.insert(path);
        }
        index = end;
    }

    calls
}

pub(crate) fn has_path_attribute(source: &str) -> bool {
    let source = compact(source);
    source.contains("#[path") || source.contains("#![path")
}

pub(crate) fn assert_boundary_scanner_regressions() {
    let import_probe = r#"
        use std::{
            env as process_env,
            process::{self as proc_mod, Command as HostCommand},
        };
        use tokio::{
            process as async_process,
        };
    "#;
    let imports = use_paths(import_probe);
    assert!(imports.contains("std::env"));
    assert!(imports.contains("std::process"));
    assert!(imports.contains("std::process::Command"));
    assert!(imports.contains("tokio::process"));

    let path_probe = "let _ = std ::\n process ::\n id ( );";
    assert!(path_sequences(path_probe).contains("std::process::id"));
    assert!(qualified_call_paths(path_probe).contains("std::process::id"));

    let attr_probe = "#[ path = \"runtime_bridge.rs\" ]\nmod helper;";
    assert!(has_path_attribute(attr_probe));

    let artifact_probe = r#"
        artifact(
            "events",
            "attempt",
            "events.jsonl",
            "event_log",
        );
        let leaked = "events.jsonl";
    "#;
    let stripped = crate::data_boundary_rule_sets::strip_artifact_declaration_calls(artifact_probe);
    assert!(!stripped.contains("\"attempt\""));
    assert!(stripped.contains("let leaked = \"events.jsonl\""));

    let cfg_all_test_probe = r#"
        #[cfg(all(test, feature = "integration"))]
        mod integration_tests;
    "#;
    let stripped = strip_cfg_test_items(cfg_all_test_probe);
    assert!(!stripped.contains("mod integration_tests"));
}

fn use_statements(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut index = 0;

    while let Some(use_start) = find_keyword(source, "use", index) {
        let mut cursor = skip_whitespace(source, use_start + "use".len());
        let start = cursor;
        let mut brace_depth = 0isize;
        while cursor < source.len() {
            let Some(ch) = source[cursor..].chars().next() else {
                break;
            };
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                ';' if brace_depth == 0 => {
                    statements.push(source[start..cursor].trim().to_string());
                    cursor += ch.len_utf8();
                    break;
                }
                _ => {}
            }
            cursor += ch.len_utf8();
        }
        index = cursor.max(use_start + "use".len());
    }

    statements
}

fn expand_use_item(prefix: &str, item: &str) -> Vec<String> {
    let item = item.trim();
    if item.is_empty() {
        return Vec::new();
    }
    let Some((group_start, group_end)) = top_level_group(item) else {
        let leaf = strip_alias(item);
        if leaf == "self" {
            return vec![prefix.to_string()];
        }
        if leaf == "*" {
            return vec![combine_path(prefix, "*")];
        }
        return vec![combine_path(prefix, &leaf)];
    };

    let before_group = item[..group_start]
        .trim()
        .trim_end_matches(':')
        .trim_end_matches(':')
        .trim();
    let group_prefix = combine_path(prefix, before_group);
    split_top_level_commas(&item[group_start + 1..group_end])
        .into_iter()
        .flat_map(|child| expand_use_item(&group_prefix, child))
        .collect()
}

fn top_level_group(item: &str) -> Option<(usize, usize)> {
    let mut depth = 0isize;
    let mut start = None;

    for (index, ch) in item.char_indices() {
        match ch {
            '{' if depth == 0 => {
                start = Some(index);
                depth = 1;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return start.map(|start| (start, index));
                }
            }
            _ => {}
        }
    }

    None
}

fn split_top_level_commas(group: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0isize;

    for (index, ch) in group.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(group[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if start < group.len() {
        parts.push(group[start..].trim());
    }

    parts
}

fn strip_alias(item: &str) -> String {
    item.split_whitespace()
        .take_while(|part| *part != "as")
        .collect::<Vec<_>>()
        .join("")
        .trim_matches(':')
        .trim_start_matches("crate::")
        .to_string()
}

fn combine_path(prefix: &str, item: &str) -> String {
    let item = normalize_path(item);
    if prefix.is_empty() {
        item
    } else if item.is_empty() {
        prefix.to_string()
    } else {
        format!("{}::{}", normalize_path(prefix), item)
    }
}

fn normalize_path(path: &str) -> String {
    path.split_whitespace()
        .collect::<String>()
        .trim_matches(':')
        .trim_start_matches("crate::")
        .to_string()
}

pub(crate) fn identifier_tokens(source: &str) -> BTreeSet<&str> {
    let mut tokens = BTreeSet::new();
    let mut start = None;

    for (index, ch) in source.char_indices() {
        if ch == '_' || ch.is_ascii_alphanumeric() {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(token_start) = start.take() {
            tokens.insert(&source[token_start..index]);
        }
    }
    if let Some(token_start) = start {
        tokens.insert(&source[token_start..]);
    }

    tokens
}

pub(crate) fn call_names(source: &str) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    let mut index = 0;

    while index < source.len() {
        let Some((token_start, token_end)) = next_identifier(source, index) else {
            break;
        };
        let mut cursor = token_end;
        while let Some(ch) = source[cursor..].chars().next() {
            if !ch.is_whitespace() {
                break;
            }
            cursor += ch.len_utf8();
        }
        if source[cursor..].starts_with('(') {
            names.insert(&source[token_start..token_end]);
        }
        index = token_end;
    }

    names
}

pub(crate) fn string_literals(source: &str) -> Vec<String> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut literals = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        match (chars.get(index), chars.get(index + 1)) {
            (Some('/'), Some('/')) => {
                index += 2;
                while index < chars.len() && chars[index] != '\n' {
                    index += 1;
                }
            }
            (Some('/'), Some('*')) => {
                index += 2;
                while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                    index += 1;
                }
                index = (index + 2).min(chars.len());
            }
            (Some('"'), _) => {
                index += 1;
                let mut literal = String::new();
                while index < chars.len() {
                    let ch = chars[index];
                    index += 1;
                    if ch == '\\' && index < chars.len() {
                        literal.push(chars[index]);
                        index += 1;
                        continue;
                    }
                    if ch == '"' {
                        break;
                    }
                    literal.push(ch);
                }
                literals.push(literal);
            }
            _ => index += 1,
        }
    }

    literals
}

pub(crate) fn strip_comments_and_strings(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;

    while index < chars.len() {
        match (chars.get(index), chars.get(index + 1)) {
            (Some('/'), Some('/')) => {
                output.push(' ');
                output.push(' ');
                index += 2;
                while index < chars.len() && chars[index] != '\n' {
                    output.push(' ');
                    index += 1;
                }
            }
            (Some('/'), Some('*')) => {
                output.push(' ');
                output.push(' ');
                index += 2;
                while index + 1 < chars.len() && !(chars[index] == '*' && chars[index + 1] == '/') {
                    output.push(if chars[index] == '\n' { '\n' } else { ' ' });
                    index += 1;
                }
                if index + 1 < chars.len() {
                    output.push(' ');
                    output.push(' ');
                    index += 2;
                }
            }
            (Some('"'), _) => {
                output.push(' ');
                index += 1;
                while index < chars.len() {
                    let ch = chars[index];
                    output.push(if ch == '\n' { '\n' } else { ' ' });
                    index += 1;
                    if ch == '\\' && index < chars.len() {
                        output.push(' ');
                        index += 1;
                        continue;
                    }
                    if ch == '"' {
                        break;
                    }
                }
            }
            (Some('\''), Some(next)) if next.is_ascii_alphanumeric() || *next == '_' => {
                output.push(' ');
                index += 1;
            }
            (Some(ch), _) => {
                output.push(*ch);
                index += 1;
            }
            (None, _) => break,
        }
    }

    output
}

pub(crate) fn strip_cfg_test_items(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut skip_next_item = false;
    let mut skipping_block = false;
    let mut brace_depth = 0isize;

    for line in source.lines() {
        let trimmed = line.trim_start();
        if skipping_block {
            brace_depth += brace_delta(line);
            output.push('\n');
            if brace_depth <= 0 {
                skipping_block = false;
                brace_depth = 0;
            }
            continue;
        }
        if trimmed.starts_with("#[cfg(test")
            || trimmed.starts_with("#[cfg(any(test")
            || trimmed.starts_with("#[cfg(all(test")
        {
            skip_next_item = true;
            output.push('\n');
            continue;
        }
        if skip_next_item {
            let delta = brace_delta(line);
            if delta > 0 {
                skipping_block = true;
                brace_depth = delta;
            }
            skip_next_item = false;
            output.push('\n');
            continue;
        }

        output.push_str(line);
        output.push('\n');
    }

    output
}

fn next_identifier(source: &str, from: usize) -> Option<(usize, usize)> {
    let mut start = None;

    for (offset, ch) in source[from..].char_indices() {
        let index = from + offset;
        if ch == '_' || ch.is_ascii_alphanumeric() {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(token_start) = start {
            return Some((token_start, index));
        }
    }

    start.map(|token_start| (token_start, source.len()))
}

fn next_path(source: &str, from: usize) -> Option<(usize, usize, String)> {
    let (start, mut cursor) = next_identifier(source, from)?;
    let mut path = source[start..cursor].to_string();

    loop {
        cursor = skip_whitespace(source, cursor);
        if !source[cursor..].starts_with("::") {
            break;
        }
        cursor += "::".len();
        cursor = skip_whitespace(source, cursor);
        let Some((segment_start, segment_end)) = identifier_at(source, cursor) else {
            break;
        };
        path.push_str("::");
        path.push_str(&source[segment_start..segment_end]);
        cursor = segment_end;
    }

    Some((start, cursor, path))
}

fn identifier_at(source: &str, from: usize) -> Option<(usize, usize)> {
    let (start, end) = next_identifier(source, from)?;
    if start == from {
        Some((start, end))
    } else {
        None
    }
}

fn skip_whitespace(source: &str, from: usize) -> usize {
    let mut cursor = from;
    while let Some(ch) = source[cursor..].chars().next() {
        if !ch.is_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }
    cursor
}

fn find_keyword(source: &str, keyword: &str, from: usize) -> Option<usize> {
    let mut search_from = from;
    while let Some(offset) = source[search_from..].find(keyword) {
        let index = search_from + offset;
        let before = source[..index].chars().next_back();
        let after = source[index + keyword.len()..].chars().next();
        if !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char) {
            return Some(index);
        }
        search_from = index + keyword.len();
    }
    None
}

fn is_ident_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn brace_delta(line: &str) -> isize {
    line.chars().fold(0, |depth, ch| match ch {
        '{' => depth + 1,
        '}' => depth - 1,
        _ => depth,
    })
}
