pub fn redact_known_secret(value: &str, secrets: &[&str]) -> String {
    let mut redacted = value.to_string();
    for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
        redacted = redacted.replace(secret, "[REDACTED]");
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::redact_known_secret;

    #[test]
    fn cfg_003_redacts_secret_values_without_removing_names() {
        let rendered = redact_known_secret(
            "OPENAI_API_KEY=sk-test-secret and PATH=/bin",
            &["sk-test-secret"],
        );

        assert_eq!(rendered, "OPENAI_API_KEY=[REDACTED] and PATH=/bin");
    }
}
