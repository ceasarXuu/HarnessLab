use harnesslab_core::AgentProfile;

pub(crate) const TERMINAL_BENCH_AGENT_LABEL: &str = "terminal_bench_agent";
pub(crate) const TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL: &str = "terminal_bench_agent_import_path";
pub(crate) const TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL: &str = "terminal_bench_agent_pythonpath";
pub(crate) const TERMINAL_BENCH_MODEL_LABEL: &str = "terminal_bench_model";
pub(crate) const GENERIC_MODEL_LABEL: &str = "model";
pub(crate) const SWE_BENCH_PRO_AGENT_LABEL: &str = "swe_bench_pro_agent";

pub(crate) const BENCHMARK_RUNTIME_LABEL_ALLOWLIST: &[&str] = &[
    TERMINAL_BENCH_AGENT_LABEL,
    TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL,
    TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL,
    TERMINAL_BENCH_MODEL_LABEL,
    GENERIC_MODEL_LABEL,
    SWE_BENCH_PRO_AGENT_LABEL,
];

/// Per-adapter compatibility profile produced by the adapter itself.
/// Generic layers must not branch on benchmark id; they consume this
/// profile opaquely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdapterCompatibilityProfile {
    pub(crate) host_execution_reason: Option<&'static str>,
    pub(crate) bridge_mode: &'static str,
    pub(crate) consumed_label_keys: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkRuntimeCompatibility {
    pub(crate) terminal_bench_agent: Option<String>,
    pub(crate) terminal_bench_agent_import_path: Option<String>,
    pub(crate) terminal_bench_agent_pythonpath: Option<String>,
    pub(crate) terminal_bench_model: Option<String>,
    pub(crate) swe_bench_pro_agent: Option<String>,
}

impl BenchmarkRuntimeCompatibility {
    pub(crate) fn from_profile(profile: &AgentProfile) -> Self {
        Self {
            terminal_bench_agent: non_empty_label(profile, TERMINAL_BENCH_AGENT_LABEL),
            terminal_bench_agent_import_path: non_empty_label(
                profile,
                TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL,
            ),
            terminal_bench_agent_pythonpath: non_empty_label(
                profile,
                TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL,
            ),
            terminal_bench_model: non_empty_label(profile, TERMINAL_BENCH_MODEL_LABEL)
                .or_else(|| non_empty_label(profile, GENERIC_MODEL_LABEL))
                .filter(|value| value != "user-configured"),
            swe_bench_pro_agent: non_empty_label(profile, SWE_BENCH_PRO_AGENT_LABEL),
        }
    }
}

pub(crate) fn non_empty_label(profile: &AgentProfile, key: &str) -> Option<String> {
    debug_assert!(BENCHMARK_RUNTIME_LABEL_ALLOWLIST.contains(&key));
    profile
        .labels
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
}

pub(crate) fn push_if_some(keys: &mut Vec<&'static str>, key: &'static str, value: &Option<String>) {
    if value.is_some() && !keys.contains(&key) {
        keys.push(key);
    }
}
