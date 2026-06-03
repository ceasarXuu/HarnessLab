use anyhow::Result;
use harnesslab_core::AgentProfile;
use serde_json::Value;

pub(super) fn profile_redaction_values(
    runtime_profile: &AgentProfile,
    report_profile: &AgentProfile,
) -> Result<Vec<String>> {
    let runtime = serde_json::to_value(runtime_profile)?;
    let report = serde_json::to_value(report_profile)?;
    let mut values = Vec::new();
    collect_redacted_values(&runtime, &report, &mut values);
    Ok(values)
}

fn collect_redacted_values(runtime: &Value, report: &Value, values: &mut Vec<String>) {
    match (runtime, report) {
        (Value::String(raw), Value::String(redacted)) => {
            for value in redacted_segments(raw, redacted) {
                push_unique(values, value);
            }
        }
        (Value::Array(raw), Value::Array(redacted)) => {
            for (raw, redacted) in raw.iter().zip(redacted.iter()) {
                collect_redacted_values(raw, redacted, values);
            }
        }
        (Value::Object(raw), Value::Object(redacted)) => {
            for (key, raw_value) in raw {
                if let Some(redacted_value) = redacted.get(key) {
                    collect_redacted_values(raw_value, redacted_value, values);
                }
            }
        }
        _ => {}
    }
}

fn redacted_segments(raw: &str, redacted: &str) -> Vec<String> {
    if !redacted.contains("[REDACTED]") || raw == redacted {
        return Vec::new();
    }
    let parts = redacted.split("[REDACTED]").collect::<Vec<_>>();
    let mut cursor = 0usize;
    let mut values = Vec::new();
    for pair in parts.windows(2) {
        let prefix = pair[0];
        if !prefix.is_empty() {
            let Some(offset) = raw[cursor..].find(prefix) else {
                return token_fallback(raw, redacted);
            };
            cursor += offset + prefix.len();
        }
        let suffix = pair[1];
        let end = if suffix.is_empty() {
            raw.len()
        } else if let Some(offset) = raw[cursor..].find(suffix) {
            cursor + offset
        } else {
            return token_fallback(raw, redacted);
        };
        if end > cursor {
            values.push(raw[cursor..end].to_string());
        }
        cursor = end;
    }
    values
}

fn token_fallback(raw: &str, redacted: &str) -> Vec<String> {
    raw.split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                ch == '\'' || ch == '"' || ch == '`' || ch == ';' || ch == ','
            })
        })
        .filter(|token| !token.is_empty() && !redacted.contains(token))
        .map(str::to_string)
        .collect()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::redacted_segments;

    #[test]
    fn extracts_raw_substrings_aligned_to_redaction_markers() {
        assert_eq!(
            redacted_segments(
                r#"sh -c 'TOKEN=do-not-leak; printf %s "$TOKEN"'"#,
                r#"sh -c 'TOKEN=[REDACTED]; printf %s "$TOKEN"'"#,
            ),
            vec!["do-not-leak"]
        );
        assert_eq!(
            redacted_segments("a=first b=second", "a=[REDACTED] b=[REDACTED]"),
            vec!["first", "second"]
        );
    }
}
