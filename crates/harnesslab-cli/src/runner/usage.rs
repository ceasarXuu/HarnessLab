use harnesslab_core::{AgentProfile, FailureCode, UsageRecord, parse_keyed_usage};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MAX_USAGE_SCAN_BYTES: u64 = 64 * 1024;

pub(super) fn collect_usage(
    profile: &AgentProfile,
    attempt_dir: &Path,
) -> (UsageRecord, Vec<FailureCode>) {
    match profile.usage.parser.as_str() {
        "none" => (UsageRecord::Unknown, vec![FailureCode::UsageUnknown]),
        "regex" => {
            let text = match read_usage_source(profile, attempt_dir) {
                Ok(text) if !text.trim().is_empty() => text,
                _ => {
                    return (
                        UsageRecord::ParseError {
                            message: "usage source unreadable or empty".to_string(),
                        },
                        vec![FailureCode::UsageParserFailed],
                    );
                }
            };
            let usage = parse_keyed_usage(
                &text,
                &profile.usage.input_tokens_key,
                &profile.usage.output_tokens_key,
                &profile.usage.total_tokens_key,
                &profile.usage.cost_usd_key,
            );
            let warnings = if matches!(usage, UsageRecord::Parsed { .. }) {
                Vec::new()
            } else {
                vec![FailureCode::UsageParserFailed]
            };
            (usage, warnings)
        }
        "json_path" => {
            let text = match read_usage_source(profile, attempt_dir) {
                Ok(text) if !text.trim().is_empty() => text,
                _ => {
                    return (
                        UsageRecord::ParseError {
                            message: "usage source unreadable or empty".to_string(),
                        },
                        vec![FailureCode::UsageParserFailed],
                    );
                }
            };
            let usage = parse_json_usage(profile, &text);
            let warnings = if matches!(usage, UsageRecord::Parsed { .. }) {
                Vec::new()
            } else {
                vec![FailureCode::UsageParserFailed]
            };
            (usage, warnings)
        }
        other => (
            UsageRecord::ParseError {
                message: format!("unknown usage parser: {other}"),
            },
            vec![FailureCode::UsageParserFailed],
        ),
    }
}

fn read_usage_source(profile: &AgentProfile, attempt_dir: &Path) -> std::io::Result<String> {
    let agent_dir = attempt_dir.join("agent");
    match profile.usage.source.as_str() {
        "agent_stdout" => read_tail(&agent_dir.join("stdout.log")),
        "agent_stderr" => read_tail(&agent_dir.join("stderr.log")),
        source if source.starts_with("file:") => {
            let relative = &source[5..];
            if harnesslab_core::report_artifact_path(relative).is_err() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "unsafe usage source path",
                ));
            }
            read_tail(&attempt_dir.join(relative))
        }
        "agent_logs" => {
            let mut text = read_tail(&agent_dir.join("stdout.log")).unwrap_or_default();
            text.push('\n');
            text.push_str(&read_tail(&agent_dir.join("stderr.log")).unwrap_or_default());
            Ok(text)
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "unsupported usage source",
        )),
    }
}

fn read_tail(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(MAX_USAGE_SCAN_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.take(MAX_USAGE_SCAN_BYTES).read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn parse_json_usage(profile: &AgentProfile, text: &str) -> UsageRecord {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) else {
        return UsageRecord::ParseError {
            message: "usage json source is not valid json".to_string(),
        };
    };
    let input = json_u64(&value, &profile.usage.input_tokens_key);
    let output = json_u64(&value, &profile.usage.output_tokens_key);
    let total = json_u64(&value, &profile.usage.total_tokens_key);
    let cost_usd = json_f64(&value, &profile.usage.cost_usd_key);
    match (input, output) {
        (Some(input_tokens), Some(output_tokens)) => UsageRecord::Parsed {
            input_tokens,
            output_tokens,
            total_tokens: total.unwrap_or(input_tokens + output_tokens),
            cost_usd,
        },
        _ => UsageRecord::ParseError {
            message: "usage json paths not found".to_string(),
        },
    }
}

fn json_u64(value: &serde_json::Value, path: &str) -> Option<u64> {
    json_at(value, path).and_then(serde_json::Value::as_u64)
}

fn json_f64(value: &serde_json::Value, path: &str) -> Option<f64> {
    json_at(value, path).and_then(serde_json::Value::as_f64)
}

fn json_at<'a>(value: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{AgentKind, default_agent_profile};

    #[test]
    fn use_005_collect_usage_regex_reads_agent_logs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("agent")).unwrap();
        std::fs::write(tmp.path().join("agent/stdout.log"), "input_tokens=2\n").unwrap();
        std::fs::write(tmp.path().join("agent/stderr.log"), "output_tokens=3\n").unwrap();
        let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
        profile.usage.parser = "regex".to_string();

        let (usage, warnings) = collect_usage(&profile, tmp.path());

        assert_eq!(usage.total_tokens(), Some(5));
        assert!(warnings.is_empty());
    }

    #[test]
    fn use_005_collect_usage_json_path_reads_configured_source() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("agent")).unwrap();
        std::fs::write(
            tmp.path().join("agent/stdout.log"),
            r#"{"usage":{"input":2,"output":5,"cost":0.07}}"#,
        )
        .unwrap();
        let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
        profile.usage.parser = "json_path".to_string();
        profile.usage.source = "agent_stdout".to_string();
        profile.usage.input_tokens_key = "usage.input".to_string();
        profile.usage.output_tokens_key = "usage.output".to_string();
        profile.usage.cost_usd_key = "usage.cost".to_string();

        let (usage, warnings) = collect_usage(&profile, tmp.path());

        assert_eq!(usage.total_tokens(), Some(7));
        assert!(matches!(
            usage,
            UsageRecord::Parsed {
                cost_usd: Some(0.07),
                ..
            }
        ));
        assert!(warnings.is_empty());
    }

    #[test]
    fn use_005_collect_usage_reports_missing_and_unknown_sources() {
        let tmp = tempfile::tempdir().unwrap();
        let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
        profile.usage.parser = "regex".to_string();
        profile.usage.source = "agent_stdout".to_string();

        let (usage, warnings) = collect_usage(&profile, tmp.path());

        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);

        std::fs::create_dir_all(tmp.path().join("agent")).unwrap();
        std::fs::write(
            tmp.path().join("agent/stdout.log"),
            "input_tokens=1 output_tokens=2",
        )
        .unwrap();
        profile.usage.parser = "mystery".to_string();
        let (usage, warnings) = collect_usage(&profile, tmp.path());
        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);

        profile.usage.parser = "regex".to_string();
        profile.usage.source = "typo".to_string();
        let (usage, warnings) = collect_usage(&profile, tmp.path());
        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);
    }

    #[test]
    fn use_005_collect_usage_json_path_reports_invalid_json_and_missing_paths() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("agent")).unwrap();
        std::fs::write(tmp.path().join("agent/stdout.log"), "not-json").unwrap();
        let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
        profile.usage.parser = "json_path".to_string();
        profile.usage.source = "agent_stdout".to_string();

        let (usage, warnings) = collect_usage(&profile, tmp.path());

        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);

        std::fs::write(tmp.path().join("agent/stdout.log"), "{}").unwrap();
        let (usage, warnings) = collect_usage(&profile, tmp.path());
        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);
    }

    #[test]
    fn use_005_collect_usage_reads_configured_file_tail() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("usage.log"),
            format!("{}input_tokens=8 output_tokens=13", "x".repeat(70 * 1024)),
        )
        .unwrap();
        let mut profile = default_agent_profile("fake", AgentKind::Fake, "agent");
        profile.usage.parser = "regex".to_string();
        profile.usage.source = "file:usage.log".to_string();

        let (usage, warnings) = collect_usage(&profile, tmp.path());

        assert_eq!(usage.total_tokens(), Some(21));
        assert!(warnings.is_empty());

        profile.usage.source = "file:../usage.log".to_string();
        let (usage, warnings) = collect_usage(&profile, tmp.path());
        assert!(matches!(usage, UsageRecord::ParseError { .. }));
        assert_eq!(warnings, vec![FailureCode::UsageParserFailed]);
    }
}
