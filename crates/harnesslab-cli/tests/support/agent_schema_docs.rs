use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct DocFieldRow {
    required: String,
    values: String,
    example: String,
    status: String,
    meaning: String,
    cells: Vec<String>,
}

pub fn assert_schema_docs_match(json: &serde_json::Value) {
    let reference_doc = read_repo_file(
        "docs/archive/2026-06-15-pre-harbor-webui-redesign/agent-profile-reference.md",
    );
    let guide_doc = read_repo_file(
        "docs/archive/2026-06-15-pre-harbor-webui-redesign/agent-registration-guide.md",
    );
    let reference_rows = parse_doc_field_rows(&reference_doc);
    let guide_rows = parse_doc_field_rows(&guide_doc);

    for field in json["fields"].as_array().unwrap() {
        assert_doc_field_semantics("agent-profile-reference.md", &reference_rows, field);
        assert_doc_field_semantics("agent-registration-guide.md", &guide_rows, field);
    }
}

fn parse_doc_field_rows(doc: &str) -> HashMap<String, DocFieldRow> {
    let mut rows = HashMap::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("| `") || !trimmed.ends_with('|') {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect::<Vec<_>>();
        if cells.len() != 6 {
            continue;
        }
        let Some(path) = markdown_code_value(&cells[0]) else {
            continue;
        };
        rows.insert(
            path,
            DocFieldRow {
                required: cells[1].clone(),
                values: cells[2].clone(),
                example: cells[3].clone(),
                status: cells[4].clone(),
                meaning: cells[5].clone(),
                cells,
            },
        );
    }
    rows
}

fn assert_doc_field_semantics(
    doc_name: &str,
    rows: &HashMap<String, DocFieldRow>,
    field: &serde_json::Value,
) {
    let path = field["path"].as_str().unwrap();
    let row = rows
        .get(path)
        .unwrap_or_else(|| panic!("{doc_name} missing table row for `{path}`"));
    assert_eq!(
        row.cells.len(),
        6,
        "{doc_name} `{path}` must keep full field semantics"
    );
    assert_required(doc_name, path, row, field["required"].as_bool().unwrap());
    assert_allowed_values(doc_name, path, row, field);
    assert_example(doc_name, path, row, field);
    assert_default(doc_name, path, row, field);
    assert_status_and_meaning(doc_name, path, row, field);
}

fn assert_required(doc_name: &str, path: &str, row: &DocFieldRow, required: bool) {
    let required_norm = normalize_for_match(&row.required);
    let matches = if required {
        required_norm.contains("yes") || required_norm.contains("是")
    } else {
        required_norm.contains("no") || required_norm.contains("否")
    };
    assert!(
        matches,
        "{doc_name} `{path}` required cell does not match schema"
    );
}

fn assert_allowed_values(doc_name: &str, path: &str, row: &DocFieldRow, field: &serde_json::Value) {
    for allowed in field["allowed_values"].as_array().unwrap() {
        let allowed = allowed.as_str().unwrap();
        assert!(
            contains_semantic_token(&row.values, allowed),
            "{doc_name} `{path}` values cell missing schema token `{allowed}`"
        );
    }
}

fn assert_example(doc_name: &str, path: &str, row: &DocFieldRow, field: &serde_json::Value) {
    let example = render_schema_value(&field["example"]);
    assert!(
        contains_semantic_token(&row.example, &example),
        "{doc_name} `{path}` example cell missing schema example `{example}`"
    );
}

fn assert_default(doc_name: &str, path: &str, row: &DocFieldRow, field: &serde_json::Value) {
    if let Some(default_value) = field.get("default_value") {
        let rendered_default = render_schema_value(default_value);
        let combined = format!("{} {} {}", row.example, row.status, row.meaning);
        assert!(
            contains_semantic_token(&combined, &rendered_default),
            "{doc_name} `{path}` row missing schema default `{rendered_default}`"
        );
    }
}

fn assert_status_and_meaning(
    doc_name: &str,
    path: &str,
    row: &DocFieldRow,
    field: &serde_json::Value,
) {
    let status = field["status"].as_str().unwrap();
    assert!(
        contains_semantic_token(&row.status, status),
        "{doc_name} `{path}` status cell missing schema status `{status}`"
    );
    let description = field["description"].as_str().unwrap();
    assert!(
        contains_semantic_token(&row.meaning, description),
        "{doc_name} `{path}` meaning cell missing schema description `{description}`"
    );
}

fn markdown_code_value(cell: &str) -> Option<String> {
    let start = cell.find('`')?;
    let rest = &cell[start + 1..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

fn contains_semantic_token(haystack: &str, needle: &str) -> bool {
    normalize_for_match(haystack).contains(&normalize_for_match(needle))
}

fn normalize_for_match(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| match ch {
            '`' | '"' | '\'' | ' ' | '\t' | '\n' | '\r' => None,
            '、' | '，' => Some(','),
            _ => Some(ch.to_ascii_lowercase()),
        })
        .collect()
}

fn render_schema_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        _ => serde_json::to_string(value).unwrap(),
    }
}

fn read_repo_file(path: &str) -> String {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(repo_root.join(path)).unwrap_or_else(|err| panic!("read {path}: {err}"))
}
