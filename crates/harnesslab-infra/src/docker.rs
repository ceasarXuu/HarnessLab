use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResult {
    pub status: String,
    pub message: String,
}

pub struct DockerCliProvider;

impl DockerCliProvider {
    pub fn health_check() -> HealthResult {
        let status = Command::new("docker")
            .arg("info")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match status {
            Ok(status) if status.success() => HealthResult {
                status: "ok".to_string(),
                message: "Docker daemon reachable".to_string(),
            },
            Ok(_) => HealthResult {
                status: "error".to_string(),
                message: "Docker daemon unavailable".to_string(),
            },
            Err(_) => HealthResult {
                status: "error".to_string(),
                message: "Docker CLI not found".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_sbox_001_health_check_is_structured() {
        let result = DockerCliProvider::health_check();

        assert!(matches!(result.status.as_str(), "ok" | "error"));
        assert!(!result.message.is_empty());
    }
}
