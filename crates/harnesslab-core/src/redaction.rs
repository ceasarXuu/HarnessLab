pub fn redact_known_secret(value: &str, secrets: &[&str]) -> String {
    let mut redacted = value.to_string();
    for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
        redacted = redacted.replace(secret, "[REDACTED]");
    }
    redacted
}

pub fn redact_public_value(value: &str, secrets: &[&str]) -> String {
    redact_known_secret(value, secrets)
        .split_inclusive(char::is_whitespace)
        .map(redact_sensitive_token)
        .collect()
}

fn redact_sensitive_token(token: &str) -> String {
    let trimmed = token.trim_end_matches(char::is_whitespace);
    let suffix = &token[trimmed.len()..];
    if is_sensitive_token(trimmed) {
        format!("[REDACTED]{suffix}")
    } else {
        token.to_string()
    }
}

fn is_sensitive_token(token: &str) -> bool {
    let normalized = token
        .trim_matches(|c: char| c == '\'' || c == '"' || c == '`' || c == ';' || c == ',')
        .to_ascii_lowercase();
    normalized.contains("sk-")
        || normalized.contains("github_pat_")
        || normalized.contains("ghp_")
        || normalized.contains("gho_")
        || normalized.contains("ghu_")
        || normalized.contains("ghs_")
        || normalized.contains("xoxb-")
        || normalized.contains("xoxp-")
        || normalized.contains("api_key")
        || normalized.contains("apikey")
        || normalized.contains("access_token")
        || normalized.contains("auth_token")
        || normalized.contains("password")
        || normalized.contains("passwd")
        || normalized.contains("secret")
}

#[cfg(test)]
mod tests {
    use super::{redact_known_secret, redact_public_value};

    #[test]
    fn cfg_003_redacts_secret_values_without_removing_names() {
        let rendered = redact_known_secret(
            "OPENAI_API_KEY=sk-test-secret and PATH=/bin",
            &["sk-test-secret"],
        );

        assert_eq!(rendered, "OPENAI_API_KEY=[REDACTED] and PATH=/bin");
    }

    #[test]
    fn cfg_003_redacts_sensitive_public_tokens() {
        let rendered = redact_public_value("agent --token sk-hardcoded --name ok", &[]);

        assert_eq!(rendered, "agent --token [REDACTED] --name ok");
    }
}
