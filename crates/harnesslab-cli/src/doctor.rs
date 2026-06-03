use crate::agent_registry::materialize_profile;
use crate::benchmark_data::resolve_benchmarks_dir;
use crate::output::{DoctorCheck, DoctorOutput};
use crate::print_json;
use anyhow::Result;
use harnesslab_adapters::built_in_descriptors_with_root;
use harnesslab_core::{
    AgentProfile, GlobalConfig, data_state_blocks_run, effective_auth_mount_specs,
    parse_auth_mount, report_artifact_path,
};
use harnesslab_infra::{DockerCliProvider, command_exists, first_command_word};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn run(home: &Path, json: bool) -> Result<i32> {
    let mut checks = Vec::new();
    checks.push(check(
        "m0.cli",
        "ok",
        "info",
        "M0 CLI skeleton is available",
    ));
    checks.push(if home.join("config.toml").exists() {
        check("config.global", "ok", "error", "Global config readable")
    } else {
        check("config.global", "error", "error", "Global config missing")
    });
    let docker = DockerCliProvider::health_check();
    checks.push(check(
        "docker.daemon",
        &docker.status,
        "error",
        &docker.message,
    ));
    let config = load_config(home).ok();
    let benchmark_root = resolve_benchmarks_dir(home, config.as_ref());
    for descriptor in built_in_descriptors_with_root(benchmark_root.as_deref()) {
        for split in descriptor.splits {
            let blocked = data_state_blocks_run(split.data_state);
            let message = if blocked {
                format!(
                    "Benchmark split is not ready locally (data_state={})",
                    split.data_state
                )
            } else {
                format!("Benchmark split is ready (data_state={})", split.data_state)
            };
            checks.push(check_with_details(
                &format!("benchmark.{}.{}", descriptor.name, split.name),
                if blocked { "warning" } else { "ok" },
                "warning",
                &message,
                serde_json::json!({
                    "data_state": split.data_state,
                    "task_count": split.task_count,
                }),
            ));
        }
    }
    match load_profiles(home) {
        Ok(profiles) => {
            for profile in profiles {
                append_profile_checks(home, &profile, &mut checks);
            }
        }
        Err(error) => checks.push(check_with_details(
            "agents.load",
            "error",
            "error",
            "Agent profiles failed to load",
            serde_json::json!({ "error": error.to_string() }),
        )),
    }
    let status = overall_status(&checks);
    if json {
        print_json(&DoctorOutput {
            schema_version: 1,
            status,
            checks,
        })?;
    } else {
        println!("doctor: {status}");
        for check in checks {
            println!("  - {} [{}]: {}", check.id, check.status, check.message);
        }
    }
    Ok(if status == "error" { 3 } else { 0 })
}

fn append_profile_checks(home: &Path, profile: &AgentProfile, checks: &mut Vec<DoctorCheck>) {
    let report = profile.validation_report();
    if !report.errors.is_empty() {
        checks.push(check_with_details(
            &format!("agent.{}.validation", profile.name),
            "error",
            "error",
            "Agent profile configuration is invalid",
            serde_json::json!({ "errors": report.errors }),
        ));
    } else if !report.warnings.is_empty() {
        checks.push(check_with_details(
            &format!("agent.{}.validation", profile.name),
            "warning",
            "warning",
            "Agent profile configuration has warnings",
            serde_json::json!({ "warnings": report.warnings }),
        ));
    } else {
        checks.push(check(
            &format!("agent.{}.validation", profile.name),
            "ok",
            "error",
            "Agent profile configuration is valid",
        ));
    }
    append_materialization_check(profile, checks);
    let available = first_command_word(&profile.command)
        .map(command_exists)
        .unwrap_or(false);
    checks.push(check(
        &format!("agent.{}", profile.name),
        if available { "ok" } else { "error" },
        "error",
        "Agent command availability checked",
    ));
    append_auth_checks(home, profile, checks);
    checks.push(usage_check(profile));
}

fn append_materialization_check(profile: &AgentProfile, checks: &mut Vec<DoctorCheck>) {
    match materialize_profile(profile) {
        Ok(materialized) => {
            let status = if materialized.warnings.is_empty() {
                "ok"
            } else {
                "warning"
            };
            checks.push(check_with_details(
                &format!("agent.{}.capabilities.materialization", profile.name),
                status,
                "error",
                "Agent registry materialization checked",
                serde_json::json!({
                    "setup": materialized.setup_summary,
                    "skills": materialized.skills_summary,
                    "tools": materialized.tools_summary,
                    "hooks": materialized.hooks_summary,
                    "warnings": materialized.warnings,
                }),
            ));
        }
        Err(error) => checks.push(check_with_details(
            &format!("agent.{}.capabilities.materialization", profile.name),
            "error",
            "error",
            "Agent registry policy cannot be materialized",
            serde_json::json!({
                "field": error.field,
                "message": error.message,
                "suggested_fix": error.suggested_fix,
            }),
        )),
    }
}

fn append_auth_checks(_home: &Path, profile: &AgentProfile, checks: &mut Vec<DoctorCheck>) {
    let effective_mounts = effective_auth_mount_specs(profile);
    let parsed = profile
        .auth
        .include_paths
        .iter()
        .map(|entry| (entry, parse_auth_mount(entry)))
        .collect::<Vec<_>>();
    let include_paths = parsed
        .iter()
        .map(|(entry, mount)| {
            let host_path = mount
                .as_ref()
                .map(|mount| PathBuf::from(&mount.host))
                .unwrap_or_else(|| PathBuf::from(entry));
            let active = mount.as_ref().is_some_and(|mount| {
                effective_mounts
                    .iter()
                    .any(|effective| effective.mount == mount.mount)
            });
            serde_json::json!({
                "entry": entry,
                "host_path": host_path.display().to_string(),
                "exists": host_path.exists(),
                "mount": mount.as_ref().map(|mount| mount.mount.as_str()),
                "active": active,
            })
        })
        .collect::<Vec<_>>();
    let missing = include_paths
        .iter()
        .filter(|entry| {
            entry["active"].as_bool().unwrap_or(false)
                && !entry["exists"].as_bool().unwrap_or(false)
        })
        .count();
    checks.push(check_with_details(
        &format!("agent.{}.auth.include_paths", profile.name),
        if missing == 0 { "ok" } else { "warning" },
        "warning",
        if missing == 0 {
            "Auth include paths are readable or not configured"
        } else {
            "One or more auth include paths are missing"
        },
        serde_json::json!({ "paths": include_paths }),
    ));
    let existing_mounts = effective_mounts
        .iter()
        .filter_map(|mount| {
            Path::new(&mount.host)
                .exists()
                .then_some(mount.mount.clone())
        })
        .collect::<Vec<_>>();
    let dry_run = DockerCliProvider::mount_check(&existing_mounts);
    checks.push(check_with_details(
        &format!("agent.{}.auth.docker_mount", profile.name),
        &dry_run.status,
        "error",
        &dry_run.message,
        serde_json::json!({ "mounts_checked": existing_mounts }),
    ));
    checks.push(check_with_details(
        &format!("agent.{}.auth.env", profile.name),
        "ok",
        "info",
        "Auth environment inheritance inspected",
        serde_json::json!({
            "inherit": profile.auth.inherit,
            "configured": profile.auth.inherit_env,
            "present": profile.auth.inherit_env.iter().filter(|name| std::env::var_os(name).is_some()).collect::<Vec<_>>(),
        }),
    ));
    if profile.auth.mount_ssh_socket {
        let socket = std::env::var_os("SSH_AUTH_SOCK").map(PathBuf::from);
        checks.push(check_with_details(
            &format!("agent.{}.auth.ssh_socket", profile.name),
            if socket.as_ref().is_some_and(|path| path.exists()) {
                "ok"
            } else {
                "warning"
            },
            "warning",
            "SSH socket mount requested",
            serde_json::json!({ "path": socket.map(|path| path.display().to_string()) }),
        ));
    }
}

fn usage_check(profile: &AgentProfile) -> DoctorCheck {
    let parser_valid = matches!(
        profile.usage.parser.as_str(),
        "none" | "regex" | "json_path"
    );
    let source_valid =
        profile.usage.parser == "none" || usage_source_is_valid(&profile.usage.source);
    let status = if profile.usage.parser == "none" || !parser_valid || !source_valid {
        "warning"
    } else {
        "ok"
    };
    let message = if profile.usage.parser == "none" {
        "Usage parser is not configured; token/cost will be unavailable".to_string()
    } else if !parser_valid {
        format!("unknown usage parser: {}", profile.usage.parser)
    } else if !source_valid {
        format!("unsupported usage source: {}", profile.usage.source)
    } else {
        "Usage parser configuration is valid".to_string()
    };
    check_with_details(
        &format!("agent.{}.usage", profile.name),
        status,
        "warning",
        &message,
        serde_json::json!({
            "parser": profile.usage.parser,
            "source": profile.usage.source,
        }),
    )
}

fn usage_source_is_valid(source: &str) -> bool {
    matches!(source, "agent_stdout" | "agent_stderr" | "agent_logs")
        || source
            .strip_prefix("file:")
            .is_some_and(|path| report_artifact_path(path).is_ok())
}

fn load_profiles(home: &Path) -> Result<Vec<AgentProfile>> {
    let agents_dir = home.join("agents");
    if !agents_dir.exists() {
        return Ok(Vec::new());
    }
    let mut profiles = Vec::new();
    for entry in fs::read_dir(agents_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("toml") {
            profiles.push(toml::from_str(&fs::read_to_string(entry.path())?)?);
        }
    }
    profiles.sort_by(|left: &AgentProfile, right| left.name.cmp(&right.name));
    Ok(profiles)
}

fn load_config(home: &Path) -> Result<GlobalConfig> {
    Ok(toml::from_str(&fs::read_to_string(
        home.join("config.toml"),
    )?)?)
}

fn check(id: &str, status: &str, severity: &str, message: &str) -> DoctorCheck {
    DoctorCheck {
        id: id.to_string(),
        status: status.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
        details: serde_json::json!({}),
    }
}

fn check_with_details(
    id: &str,
    status: &str,
    severity: &str,
    message: &str,
    details: serde_json::Value,
) -> DoctorCheck {
    DoctorCheck {
        id: id.to_string(),
        status: status.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
        details,
    }
}

fn overall_status(checks: &[DoctorCheck]) -> &'static str {
    if checks.iter().any(|check| check.status == "error") {
        "error"
    } else if checks.iter().any(|check| check.status == "warning") {
        "warning"
    } else {
        "ok"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_008_overall_status_prioritizes_error_then_warning() {
        assert_eq!(overall_status(&[check("a", "ok", "info", "ok")]), "ok");
        assert_eq!(
            overall_status(&[
                check("a", "ok", "info", "ok"),
                check("b", "warning", "warning", "warn"),
            ]),
            "warning"
        );
        assert_eq!(
            overall_status(&[
                check("a", "warning", "warning", "warn"),
                check("b", "error", "error", "err"),
            ]),
            "error"
        );
    }

    #[test]
    fn doc_007_usage_source_validation_rejects_unsafe_file_paths() {
        assert!(usage_source_is_valid("agent_logs"));
        assert!(usage_source_is_valid("file:usage.json"));
        assert!(!usage_source_is_valid("file:../secret.json"));
    }

    #[test]
    fn doc_007_usage_check_rejects_unsupported_source() {
        let mut profile = harnesslab_core::default_agent_profile(
            "agent",
            harnesslab_core::AgentKind::Custom,
            "sh",
        );
        profile.usage.parser = "regex".to_string();
        profile.usage.source = "unsupported".to_string();

        let check = usage_check(&profile);

        assert_eq!(check.status, "warning");
        assert!(check.message.contains("unsupported usage source"));
    }

    #[test]
    fn doc_007_auth_mount_parser_matches_doctor_and_sandbox() {
        let host_home = std::env::var("HOME").unwrap_or_default();
        assert_eq!(parse_auth_mount("~:/root/home:ro").unwrap().host, host_home);
        assert_eq!(
            parse_auth_mount("~/agent:/root/agent:ro").unwrap().mount,
            format!(
                "{}/agent:/root/agent:ro",
                std::env::var("HOME").unwrap_or_default()
            )
        );
        assert_eq!(
            parse_auth_mount("relative:/root/relative:ro")
                .unwrap()
                .mount,
            "relative:/root/relative:ro"
        );
        let absolute = "/tmp/harnesslab-auth-absolute";
        assert_eq!(
            parse_auth_mount(&format!("{absolute}:/root/absolute:ro"))
                .unwrap()
                .host,
            absolute
        );
    }
}
