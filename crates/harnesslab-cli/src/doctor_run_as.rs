use crate::agent_registry::run_as_requires_sandbox;
use crate::doctor::check_with_details;
use crate::output::DoctorCheck;
use harnesslab_core::AgentProfile;

pub(crate) fn append_run_as_check(profile: &AgentProfile, checks: &mut Vec<DoctorCheck>) {
    let run_as = format!("{:?}", profile.setup.run_as).to_ascii_lowercase();
    let requires_sandbox = run_as_requires_sandbox(profile.setup.run_as);
    let host_agent_paths = host_agent_paths(profile);
    let blocked = requires_sandbox && !host_agent_paths.is_empty();
    checks.push(check_with_details(
        &format!("agent.{}.setup.run_as", profile.name),
        if blocked {
            "error"
        } else if requires_sandbox {
            "warning"
        } else {
            "ok"
        },
        if blocked { "error" } else { "warning" },
        if blocked {
            "setup.run_as is not enforceable for configured host agent execution paths"
        } else if requires_sandbox {
            "setup.run_as requires sandboxed agent execution; host tasks cannot switch users"
        } else {
            "setup.run_as is compatible with host and sandboxed agent execution"
        },
        serde_json::json!({
            "field": "setup.run_as",
            "run_as": run_as,
            "host_supported": !requires_sandbox,
            "sandbox_required": requires_sandbox,
            "blocked_host_agent_paths": host_agent_paths,
            "host_supported_values": ["current"],
            "sandbox_supported_values": ["root", "harnesslab", "current"],
        }),
    ));
}

fn host_agent_paths(profile: &AgentProfile) -> Vec<&'static str> {
    let mut paths = Vec::new();
    if profile
        .labels
        .contains_key("terminal_bench_agent_import_path")
    {
        paths.push("terminal-bench import agent host path");
    }
    if profile
        .labels
        .get("swe_bench_pro_agent")
        .map(String::as_str)
        == Some("gold")
    {
        paths.push("swe-bench-pro gold host path");
    }
    paths
}
