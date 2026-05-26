use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UsageRecord {
    Unknown,
    Parsed {
        input_tokens: u64,
        output_tokens: u64,
        total_tokens: u64,
        cost_usd: Option<f64>,
    },
    ParseError {
        message: String,
    },
}

impl UsageRecord {
    pub fn unknown() -> Self {
        Self::Unknown
    }

    pub fn total_tokens(&self) -> Option<u64> {
        match self {
            Self::Parsed { total_tokens, .. } => Some(*total_tokens),
            _ => None,
        }
    }
}

pub fn parse_regex_like_usage(text: &str) -> UsageRecord {
    let input = find_number_after(text, "input_tokens");
    let output = find_number_after(text, "output_tokens");
    match (input, output) {
        (Some(input_tokens), Some(output_tokens)) => UsageRecord::Parsed {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost_usd: None,
        },
        _ => UsageRecord::ParseError {
            message: "usage tokens not found".to_string(),
        },
    }
}

pub fn aggregate_usage(records: &[UsageRecord]) -> UsageRecord {
    let mut input = 0;
    let mut output = 0;
    let mut cost = 0.0;
    let mut has_cost = false;
    for record in records {
        let UsageRecord::Parsed {
            input_tokens,
            output_tokens,
            cost_usd,
            ..
        } = record
        else {
            return UsageRecord::Unknown;
        };
        input += input_tokens;
        output += output_tokens;
        if let Some(value) = cost_usd {
            has_cost = true;
            cost += value;
        }
    }
    UsageRecord::Parsed {
        input_tokens: input,
        output_tokens: output,
        total_tokens: input + output,
        cost_usd: has_cost.then_some(cost),
    }
}

fn find_number_after(text: &str, key: &str) -> Option<u64> {
    let index = text.find(key)?;
    let tail = &text[index + key.len()..];
    let digits: String = tail
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_001_parser_none_is_unknown() {
        assert_eq!(UsageRecord::unknown().total_tokens(), None);
    }

    #[test]
    fn use_002_regex_parser_extracts_tokens() {
        let usage = parse_regex_like_usage("input_tokens=12 output_tokens: 8");

        assert_eq!(
            usage,
            UsageRecord::Parsed {
                input_tokens: 12,
                output_tokens: 8,
                total_tokens: 20,
                cost_usd: None
            }
        );
    }

    #[test]
    fn use_003_regex_parser_miss_is_parse_error() {
        let usage = parse_regex_like_usage("no usage here");

        assert!(matches!(usage, UsageRecord::ParseError { .. }));
    }

    #[test]
    fn use_004_attempts_aggregate_parsed_usage() {
        let usage = aggregate_usage(&[
            UsageRecord::Parsed {
                input_tokens: 1,
                output_tokens: 2,
                total_tokens: 3,
                cost_usd: Some(0.1),
            },
            UsageRecord::Parsed {
                input_tokens: 3,
                output_tokens: 4,
                total_tokens: 7,
                cost_usd: Some(0.2),
            },
        ]);

        assert_eq!(usage.total_tokens(), Some(10));
    }

    #[test]
    fn use_004_attempts_aggregate_unknown_if_any_attempt_unknown() {
        let usage = aggregate_usage(&[UsageRecord::Unknown]);

        assert_eq!(usage, UsageRecord::Unknown);
    }
}
