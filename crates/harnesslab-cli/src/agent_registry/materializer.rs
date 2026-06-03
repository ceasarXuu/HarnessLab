use anyhow::Result;
use harnesslab_core::{
    AgentKind, AgentProfile, CapabilityPolicy, RunAs, SetupPreset, policy_is_default,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct MaterializedAgentProfile {
    pub setup_script: Option<String>,
    pub setup_summary: String,
    pub skills_summary: String,
    pub tools_summary: String,
    pub hooks_summary: String,
    pub run_as: RunAs,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MaterializationError {
    pub field: &'static str,
    pub message: String,
    pub suggested_fix: String,
}

pub(crate) fn materialize_profile(
    profile: &AgentProfile,
) -> Result<MaterializedAgentProfile, MaterializationError> {
    reject_unsupported_policy(profile, "skills", &profile.skills)?;
    reject_unsupported_policy(profile, "tools", &profile.tools)?;
    reject_unsupported_policy(profile, "hooks", &profile.hooks)?;
    let mut warnings = Vec::new();
    let setup_script = match profile.setup.preset {
        SetupPreset::None => legacy_setup(profile, &mut warnings),
        SetupPreset::Builtin => {
            builtin_setup(profile).or_else(|| legacy_setup(profile, &mut warnings))
        }
        SetupPreset::Custom => {
            warnings.push("advanced_custom_setup".to_string());
            Some(profile.setup.commands.join("\n"))
        }
    };
    Ok(MaterializedAgentProfile {
        setup_script: setup_script.filter(|script| !script.trim().is_empty()),
        setup_summary: setup_summary(profile),
        skills_summary: policy_summary(&profile.skills),
        tools_summary: policy_summary(&profile.tools),
        hooks_summary: policy_summary(&profile.hooks),
        run_as: profile.setup.run_as,
        warnings,
    })
}

pub(crate) fn wrap_rendered_command(command: &str, run_as: RunAs) -> String {
    match run_as {
        RunAs::Root | RunAs::Current => command.to_string(),
        RunAs::Harnesslab => format!(
            "if [ \"$(id -u)\" = \"0\" ] && id -u harnesslab >/dev/null 2>&1; then exec runuser -u harnesslab --preserve-environment -- bash -lc {}; else exec bash -lc {}; fi",
            shell_quote(command),
            shell_quote(command)
        ),
    }
}

pub(crate) fn materialization_error_to_anyhow(error: MaterializationError) -> anyhow::Error {
    anyhow::anyhow!(
        "{}: {}; suggested_fix={}",
        error.field,
        error.message,
        error.suggested_fix
    )
}

fn reject_unsupported_policy(
    profile: &AgentProfile,
    field: &'static str,
    policy: &CapabilityPolicy,
) -> Result<(), MaterializationError> {
    if policy_is_default(policy) {
        return Ok(());
    }
    Err(MaterializationError {
        field,
        message: format!(
            "non-default {field} policy is not materializable for kind {:?}",
            profile.kind
        ),
        suggested_fix: format!(
            "use default {field} policy or implement a {field} materializer for this kind"
        ),
    })
}

fn legacy_setup(profile: &AgentProfile, warnings: &mut Vec<String>) -> Option<String> {
    let command = profile.labels.get("sandbox_setup_command")?;
    if command.trim().is_empty() {
        return None;
    }
    warnings.push("legacy_sandbox_setup_command".to_string());
    Some(command.clone())
}

fn builtin_setup(profile: &AgentProfile) -> Option<String> {
    let mut script = match profile.kind {
        AgentKind::Codex => Some(missing_command_installer(
            "codex",
            "npm install -g @openai/codex",
            "codex",
        )),
        AgentKind::ClaudeCode => Some(missing_command_installer(
            "claude",
            "npm install -g @anthropic-ai/claude-code",
            "claude-code",
        )),
        AgentKind::Opencode => Some(missing_command_installer(
            "opencode",
            "npm install -g opencode-ai",
            "opencode",
        )),
        AgentKind::PiCodingAgent => None,
        AgentKind::Fake | AgentKind::Custom => None,
    }?;
    if matches!(profile.setup.run_as, RunAs::Harnesslab) {
        script.push_str("; if ! id -u harnesslab >/dev/null 2>&1; then useradd -m -s /bin/bash harnesslab; fi; chown -R harnesslab:harnesslab /workspace /home/harnesslab 2>/dev/null || true");
    }
    if profile
        .setup
        .required_commands
        .iter()
        .any(|command| command == "claude-ds")
    {
        script.push_str("; cat >/usr/local/bin/claude-ds <<'EOF'\n#!/usr/bin/env bash\nset -e\nexec claude --dangerously-skip-permissions \"$@\"\nEOF\nchmod +x /usr/local/bin/claude-ds");
    }
    Some(script)
}

fn missing_command_installer(binary: &str, install: &str, slug: &str) -> String {
    format!(
        "if ! command -v {binary} >/dev/null 2>&1; then if command -v npm >/dev/null 2>&1; then {install} >/tmp/harnesslab-{slug}-install.log 2>&1 || {{ cat /tmp/harnesslab-{slug}-install.log >&2; exit 127; }}; else echo '{binary} CLI missing and npm unavailable' >&2; exit 127; fi; fi"
    )
}

fn setup_summary(profile: &AgentProfile) -> String {
    format!(
        "preset={:?}; run_as={:?}; required_commands={:?}",
        profile.setup.preset, profile.setup.run_as, profile.setup.required_commands
    )
}

fn policy_summary(policy: &CapabilityPolicy) -> String {
    format!(
        "inherit={}; allow={:?}; deny={:?}; include_paths={:?}",
        policy.inherit, policy.allow, policy.deny, policy.include_paths
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{AgentKind, SetupPreset, default_agent_profile};

    #[test]
    fn agt_reg_004_codex_builtin_setup_comes_from_semantic_setup() {
        let profile = default_agent_profile("codex", AgentKind::Codex, "codex exec -");

        let materialized = materialize_profile(&profile).unwrap();

        let setup = materialized.setup_script.unwrap();
        assert!(setup.contains("npm install -g @openai/codex"));
        assert!(
            !materialized
                .warnings
                .contains(&"legacy_sandbox_setup_command".to_string())
        );
    }

    #[test]
    fn agt_reg_004_claude_builtin_setup_creates_claude_ds_wrapper() {
        let mut profile =
            default_agent_profile("claude-ds", AgentKind::ClaudeCode, "claude-ds -p -");
        profile
            .setup
            .required_commands
            .push("claude-ds".to_string());

        let materialized = materialize_profile(&profile).unwrap();

        let setup = materialized.setup_script.unwrap();
        assert!(setup.contains("npm install -g @anthropic-ai/claude-code"));
        assert!(setup.contains("cat >/usr/local/bin/claude-ds"));
        assert!(setup.contains("exec claude --dangerously-skip-permissions"));
    }

    #[test]
    fn agt_reg_004_materializer_blocks_non_default_tools_for_custom() {
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
        profile.tools.deny = vec!["bash".to_string()];

        let error = materialize_profile(&profile).unwrap_err();

        assert_eq!(error.field, "tools");
        assert!(error.message.contains("Custom"));
        assert!(error.suggested_fix.contains("default tools policy"));
    }

    #[test]
    fn agt_reg_004_custom_profile_with_default_capabilities_materializes() {
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
        profile.setup.preset = SetupPreset::None;

        let materialized = materialize_profile(&profile).unwrap();

        assert!(materialized.setup_script.is_none());
        assert_eq!(
            materialized.setup_summary,
            "preset=None; run_as=Harnesslab; required_commands=[]"
        );
    }

    #[test]
    fn agt_reg_004_custom_setup_is_joined_and_warned() {
        let mut profile = default_agent_profile("custom", AgentKind::Custom, "agent");
        profile.setup.preset = SetupPreset::Custom;
        profile.setup.commands = vec!["echo one".to_string(), "echo two".to_string()];

        let materialized = materialize_profile(&profile).unwrap();

        assert_eq!(
            materialized.setup_script.as_deref(),
            Some("echo one\necho two")
        );
        assert!(
            materialized
                .warnings
                .contains(&"advanced_custom_setup".to_string())
        );
    }
}
