use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct BenchmarkId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AdapterId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AdapterVersion(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AdapterProtocolVersion(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SelectedMode(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CapabilityId(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterStability {
    Experimental,
    Stable,
    Legacy,
    ConditionalStableBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdapterProtocolAuthority {
    pub benchmark_id: BenchmarkId,
    pub adapter_id: AdapterId,
    pub protocol_version: AdapterProtocolVersion,
    pub adapter_version: AdapterVersion,
    pub selected_mode: SelectedMode,
    pub capabilities: Vec<CapabilityId>,
    pub stability: AdapterStability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_runner_kind: Option<crate::ExternalRunnerKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRuntimeBinding {
    pub authority: AdapterProtocolAuthority,
    pub dataset_ref: String,
    pub task_ref: String,
    pub artifact_contract_id: String,
    pub readiness_contract_id: String,
}

macro_rules! stable_id {
    ($name:ident) => {
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, AdapterProtocolIdError> {
                let value = value.into();
                validate_protocol_id(stringify!($name), &value)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct IdVisitor;

                impl Visitor<'_> for IdVisitor {
                    type Value = String;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a normalized protocol id string")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(value.to_string())
                    }

                    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(value)
                    }
                }

                let value = deserializer.deserialize_string(IdVisitor)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

stable_id!(BenchmarkId);
stable_id!(AdapterId);
stable_id!(AdapterVersion);
stable_id!(AdapterProtocolVersion);
stable_id!(SelectedMode);
stable_id!(CapabilityId);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field} must be a non-empty normalized protocol id, got {value:?}")]
pub struct AdapterProtocolIdError {
    field: &'static str,
    value: String,
}

fn validate_protocol_id(field: &'static str, value: &str) -> Result<(), AdapterProtocolIdError> {
    let valid = !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    if valid {
        Ok(())
    } else {
        Err(AdapterProtocolIdError {
            field,
            value: value.to_string(),
        })
    }
}

impl AdapterProtocolAuthority {
    pub fn new(
        benchmark_id: BenchmarkId,
        adapter_id: AdapterId,
        adapter_version: AdapterVersion,
        selected_mode: SelectedMode,
        mut capabilities: Vec<CapabilityId>,
        stability: AdapterStability,
    ) -> Self {
        capabilities.sort();
        capabilities.dedup();
        Self {
            benchmark_id,
            adapter_id,
            protocol_version: AdapterProtocolVersion("1".to_string()),
            adapter_version,
            selected_mode,
            capabilities,
            stability,
            legacy_runner_kind: None,
        }
    }

    pub fn with_legacy_runner_kind(mut self, kind: crate::ExternalRunnerKind) -> Self {
        self.legacy_runner_kind = Some(kind);
        self
    }
}

impl<'de> Deserialize<'de> for AdapterProtocolAuthority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AuthorityWire {
            benchmark_id: BenchmarkId,
            adapter_id: AdapterId,
            protocol_version: AdapterProtocolVersion,
            adapter_version: AdapterVersion,
            selected_mode: SelectedMode,
            capabilities: Vec<CapabilityId>,
            stability: AdapterStability,
            #[serde(default)]
            legacy_runner_kind: Option<crate::ExternalRunnerKind>,
        }

        let wire = AuthorityWire::deserialize(deserializer)?;
        if wire.protocol_version.as_str() != "1" {
            return Err(de::Error::custom(format!(
                "unsupported adapter protocol version {}",
                wire.protocol_version
            )));
        }
        let mut authority = AdapterProtocolAuthority::new(
            wire.benchmark_id,
            wire.adapter_id,
            wire.adapter_version,
            wire.selected_mode,
            wire.capabilities,
            wire.stability,
        );
        authority.legacy_runner_kind = wire.legacy_runner_kind;
        Ok(authority)
    }
}

pub fn legacy_runner_kind_authority(
    kind: crate::ExternalRunnerKind,
) -> Result<AdapterProtocolAuthority, AdapterProtocolIdError> {
    let (benchmark, adapter, mode, capabilities) = match kind {
        crate::ExternalRunnerKind::TerminalBench => (
            "terminal-bench",
            "harnesslab.terminal-bench.runtime",
            "official-runner",
            vec![
                "descriptor",
                "data.lifecycle",
                "readiness.basic",
                "artifacts.basic",
                "failure.mapping",
                "replay.authority",
                "report.metadata",
                "official.runner",
                "docker.orchestration",
                "cleanup.verdict_override",
                "host.agent_execution",
                "run_as.readiness",
            ],
        ),
        crate::ExternalRunnerKind::SweBenchPro => (
            "swe-bench-pro",
            "harnesslab.swe-bench-pro.runtime",
            "patch-evaluator",
            vec![
                "descriptor",
                "data.lifecycle",
                "readiness.basic",
                "failure.mapping",
                "report.metadata",
                "patch.evaluator",
                "host.agent_execution",
                "artifacts.basic",
                "replay.authority",
            ],
        ),
    };
    Ok(AdapterProtocolAuthority::new(
        BenchmarkId::new(benchmark)?,
        AdapterId::new(adapter)?,
        AdapterVersion::new("legacy")?,
        SelectedMode::new(mode)?,
        capabilities
            .into_iter()
            .map(CapabilityId::new)
            .collect::<Result<Vec<_>, _>>()?,
        AdapterStability::Legacy,
    )
    .with_legacy_runner_kind(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapt_protocol_001_identity_authority_and_legacy_shim_contract() {
        assert!(BenchmarkId::new("terminal-bench").is_ok());
        assert!(AdapterId::new("harnesslab.terminal-bench.runtime").is_ok());
        assert!(BenchmarkId::new("Terminal-Bench").is_err());
        assert!(AdapterId::new("-bad").is_err());
        assert!(CapabilityId::new("bad space").is_err());

        let authority = AdapterProtocolAuthority::new(
            BenchmarkId::new("terminal-bench").unwrap(),
            AdapterId::new("harnesslab.terminal-bench.runtime").unwrap(),
            AdapterVersion::new("1.0.0").unwrap(),
            SelectedMode::new("official-runner").unwrap(),
            vec![
                CapabilityId::new("official.runner").unwrap(),
                CapabilityId::new("artifacts.basic").unwrap(),
                CapabilityId::new("official.runner").unwrap(),
            ],
            AdapterStability::Experimental,
        );
        let json = serde_json::to_value(&authority).unwrap();

        assert_eq!(json["protocol_version"], "1");
        assert_eq!(json["capabilities"][0], "artifacts.basic");
        assert_eq!(json["capabilities"][1], "official.runner");
        assert_eq!(json["capabilities"].as_array().unwrap().len(), 2);

        let decoded: AdapterProtocolAuthority = serde_json::from_value(json).unwrap();
        assert_eq!(decoded.capabilities.len(), 2);
        assert_eq!(decoded.capabilities[0].as_str(), "artifacts.basic");
        assert!(
            serde_json::from_str::<AdapterProtocolAuthority>(
                r#"{
                    "benchmark_id":"Terminal Bench",
                    "adapter_id":"harnesslab.terminal-bench.runtime",
                    "protocol_version":"1",
                    "adapter_version":"1.0.0",
                    "selected_mode":"official-runner",
                    "capabilities":["descriptor"],
                    "stability":"experimental"
                }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<AdapterProtocolAuthority>(
                r#"{
                    "benchmark_id":"terminal-bench",
                    "adapter_id":"harnesslab.terminal-bench.runtime",
                    "protocol_version":"2",
                    "adapter_version":"1.0.0",
                    "selected_mode":"official-runner",
                    "capabilities":["descriptor"],
                    "stability":"experimental"
                }"#
            )
            .is_err()
        );

        let legacy_authority =
            legacy_runner_kind_authority(crate::ExternalRunnerKind::SweBenchPro).unwrap();

        assert_eq!(legacy_authority.benchmark_id.as_str(), "swe-bench-pro");
        assert_eq!(
            legacy_authority.adapter_id.as_str(),
            "harnesslab.swe-bench-pro.runtime"
        );
        assert_eq!(
            legacy_authority.legacy_runner_kind,
            Some(crate::ExternalRunnerKind::SweBenchPro)
        );

        let binding = TaskRuntimeBinding {
            authority: legacy_authority,
            dataset_ref: "dataset://swe-bench-pro/smoke".to_string(),
            task_ref: "task://swe-bench-pro/smoke/1".to_string(),
            artifact_contract_id: "artifact.basic.v1".to_string(),
            readiness_contract_id: "readiness.basic.v1".to_string(),
        };
        let binding_json = serde_json::to_value(&binding).unwrap();
        assert_eq!(binding_json["authority"]["benchmark_id"], "swe-bench-pro");
        assert_eq!(
            binding_json["authority"]["adapter_id"],
            "harnesslab.swe-bench-pro.runtime"
        );
        assert_eq!(binding_json["dataset_ref"], "dataset://swe-bench-pro/smoke");
        let binding_round_trip: TaskRuntimeBinding = serde_json::from_value(binding_json).unwrap();
        assert_eq!(
            binding_round_trip.authority.benchmark_id.as_str(),
            "swe-bench-pro"
        );
    }
}
