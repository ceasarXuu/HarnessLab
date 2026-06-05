use harnesslab_core::{AgentProfile, ExternalRunnerKind};

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

    pub(crate) fn host_execution_reason(
        &self,
        runner_kind: ExternalRunnerKind,
    ) -> Option<&'static str> {
        match runner_kind {
            ExternalRunnerKind::TerminalBench
                if self.terminal_bench_agent_import_path.is_some() =>
            {
                Some("terminal-bench import agent host path")
            }
            ExternalRunnerKind::SweBenchPro if self.swe_bench_pro_uses_gold_agent() => {
                Some("swe-bench-pro gold host path")
            }
            _ => None,
        }
    }

    pub(crate) fn agent_bridge_mode(&self, runner_kind: ExternalRunnerKind) -> &'static str {
        match runner_kind {
            ExternalRunnerKind::TerminalBench
                if self.terminal_bench_agent_import_path.is_some() =>
            {
                "terminal-bench-import-path"
            }
            ExternalRunnerKind::TerminalBench => "terminal-bench-official-agent",
            ExternalRunnerKind::SweBenchPro if self.swe_bench_pro_uses_gold_agent() => {
                "swe-bench-pro-gold"
            }
            ExternalRunnerKind::SweBenchPro => "swe-bench-pro-sandbox-agent",
        }
    }

    pub(crate) fn consumed_label_keys(&self, runner_kind: ExternalRunnerKind) -> Vec<&'static str> {
        let mut keys = Vec::new();
        match runner_kind {
            ExternalRunnerKind::TerminalBench => {
                push_if_some(
                    &mut keys,
                    TERMINAL_BENCH_AGENT_LABEL,
                    &self.terminal_bench_agent,
                );
                push_if_some(
                    &mut keys,
                    TERMINAL_BENCH_AGENT_IMPORT_PATH_LABEL,
                    &self.terminal_bench_agent_import_path,
                );
                push_if_some(
                    &mut keys,
                    TERMINAL_BENCH_AGENT_PYTHONPATH_LABEL,
                    &self.terminal_bench_agent_pythonpath,
                );
                push_if_some(
                    &mut keys,
                    TERMINAL_BENCH_MODEL_LABEL,
                    &self.terminal_bench_model,
                );
            }
            ExternalRunnerKind::SweBenchPro => {
                push_if_some(
                    &mut keys,
                    SWE_BENCH_PRO_AGENT_LABEL,
                    &self.swe_bench_pro_agent,
                );
            }
        }
        keys
    }

    pub(crate) fn swe_bench_pro_uses_gold_agent(&self) -> bool {
        self.swe_bench_pro_agent.as_deref() == Some("gold")
    }
}

fn non_empty_label(profile: &AgentProfile, key: &str) -> Option<String> {
    debug_assert!(BENCHMARK_RUNTIME_LABEL_ALLOWLIST.contains(&key));
    profile
        .labels
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
}

fn push_if_some(keys: &mut Vec<&'static str>, key: &'static str, value: &Option<String>) {
    if value.is_some() && !keys.contains(&key) {
        keys.push(key);
    }
}
