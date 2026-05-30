use crate::benchmark_data::resolve_benchmarks_dir;
use crate::output::{
    BenchmarkInfoOutput, BenchmarkListOutput, DoctorCheck, DoctorOutput, InitOutput, ListOutput,
    PathOutput,
};
use crate::runner::{execute_new_run, replay_run, resume_run};
use crate::{
    AgentCommand, BenchmarkCommand, Cli, Command, ReportCommand, RunAction, RunArgs, print_json,
};
use anyhow::{Context, Result};
use harnesslab_adapters::built_in_descriptors_with_root;
use harnesslab_core::{
    AgentKind, AgentProfile, GlobalConfig, data_state_blocks_run, default_agent_profile,
};
use harnesslab_infra::{
    DockerCliProvider, command_exists, command_succeeds, first_command_word, latest_run_dir,
};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn dispatch(cli: Cli) -> Result<i32> {
    let home = resolve_home(cli.home);
    match cli.command {
        Command::Init { json } => init(&home, json),
        Command::Agent { command } => match command {
            AgentCommand::List { json } => agent_list(&home, json),
        },
        Command::Doctor { json } => doctor(&home, json),
        Command::Benchmark { command } => match command {
            BenchmarkCommand::List { json } => benchmark_list(&home, json),
            BenchmarkCommand::Info { benchmark, json } => benchmark_info(&home, &benchmark, json),
        },
        Command::Run(args) => run_command(&home, args),
        Command::Report { command } => match command {
            ReportCommand::Open { target, json } => report_open(&home, &target, json),
        },
    }
}

fn init(home: &Path, json: bool) -> Result<i32> {
    fs::create_dir_all(home.join("agents"))?;
    fs::create_dir_all(home.join("runs"))?;
    fs::create_dir_all(home.join("benchmarks"))?;
    write_if_missing(
        &home.join("config.toml"),
        &toml::to_string_pretty(&GlobalConfig::default())?,
    )?;

    let mut detected = Vec::new();
    for profile in default_profiles() {
        if profile.is_detected() {
            detected.push(profile.name.to_string());
        }
        let profile = default_agent_profile(profile.name, profile.kind, profile.command);
        write_if_missing(
            &home.join("agents").join(format!("{}.toml", profile.name)),
            &toml::to_string_pretty(&profile)?,
        )?;
    }

    if json {
        print_json(&InitOutput {
            schema_version: 1,
            command: "init",
            status: "ok",
            home: home.display().to_string(),
            detected_agents: detected,
        })?;
    } else {
        println!("init: ok home={}", home.display());
        println!("Detected agents:");
        for profile in default_profiles() {
            let profile_path = home.join("agents").join(format!("{}.toml", profile.name));
            if profile.is_detected() {
                println!("  - {}: found -> {}", profile.name, profile_path.display());
            } else {
                println!(
                    "  - {}: not found -> {}",
                    profile.name,
                    profile_path.display()
                );
            }
        }
        println!("Next:");
        println!("  1. Edit agent profiles in {}/agents", home.display());
        println!("  2. Run: harnesslab doctor");
    }
    Ok(0)
}

fn agent_list(home: &Path, json: bool) -> Result<i32> {
    let profiles = load_profiles(home)?;
    if json {
        print_json(&ListOutput {
            schema_version: 1,
            command: "agent list",
            status: "ok",
            items: profiles.into_iter().map(|profile| profile.name).collect(),
        })?;
    } else {
        for profile in profiles {
            println!("{}", profile.name);
        }
    }
    Ok(0)
}

fn doctor(home: &Path, json: bool) -> Result<i32> {
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
                match profile.validate() {
                    Ok(warnings) => {
                        if warnings.is_empty() {
                            checks.push(check(
                                &format!("agent.{}.validation", profile.name),
                                "ok",
                                "error",
                                "Agent profile configuration is valid",
                            ));
                        } else {
                            checks.push(check_with_details(
                                &format!("agent.{}.validation", profile.name),
                                "warning",
                                "warning",
                                "Agent profile configuration has warnings",
                                serde_json::json!({ "warnings": warnings.iter().map(|warning| &warning.code).collect::<Vec<_>>() }),
                            ));
                        }
                    }
                    Err(error) => checks.push(check_with_details(
                        &format!("agent.{}.validation", profile.name),
                        "error",
                        "error",
                        &error.to_string(),
                        serde_json::json!({ "error": error.to_string() }),
                    )),
                }
                let status = first_command_word(&profile.command)
                    .map(command_exists)
                    .unwrap_or(false);
                checks.push(check(
                    &format!("agent.{}", profile.name),
                    if status { "ok" } else { "error" },
                    "error",
                    "Agent command availability checked",
                ));
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
    }
    Ok(if status == "error" { 3 } else { 0 })
}

fn benchmark_list(home: &Path, json: bool) -> Result<i32> {
    let config = load_config(home).ok();
    let benchmark_root = resolve_benchmarks_dir(home, config.as_ref());
    let descriptors = built_in_descriptors_with_root(benchmark_root.as_deref());
    if json {
        print_json(&BenchmarkListOutput {
            schema_version: 1,
            command: "benchmark list",
            status: "ok",
            benchmarks: descriptors,
        })?;
    } else {
        for descriptor in descriptors {
            println!("{}", descriptor.name);
        }
    }
    Ok(0)
}

fn benchmark_info(home: &Path, name: &str, json: bool) -> Result<i32> {
    let config = load_config(home).ok();
    let benchmark_root = resolve_benchmarks_dir(home, config.as_ref());
    let descriptor = built_in_descriptors_with_root(benchmark_root.as_deref())
        .into_iter()
        .find(|descriptor| descriptor.name == name)
        .with_context(|| format!("unknown benchmark {name}"))?;
    if json {
        print_json(&BenchmarkInfoOutput {
            schema_version: 1,
            command: "benchmark info",
            status: "ok",
            benchmark: descriptor,
        })?;
    } else {
        println!("benchmark info: {}", descriptor.name);
    }
    Ok(0)
}

fn run_command(home: &Path, args: RunArgs) -> Result<i32> {
    match args.action {
        Some(RunAction::Resume { run_dir, json }) => resume_run(home, &run_dir, json),
        Some(RunAction::Replay { run_dir, json }) => replay_run(home, &run_dir, json),
        None => {
            let agent = args.agent.context("--agent is required")?;
            let benchmark = args.benchmark.context("--benchmark is required")?;
            let split = args.split.context("--split is required")?;
            execute_new_run(home, &agent, &benchmark, &split, args.json, None)
        }
    }
}

fn report_open(home: &Path, target: &str, json: bool) -> Result<i32> {
    let run_dir = if target == "latest" {
        latest_run_dir(&home.join("runs"))?.context("no runs found")?
    } else {
        PathBuf::from(target)
    };
    let report = run_dir.join("report.html");
    if json {
        print_json(&PathOutput {
            schema_version: 1,
            command: "report open",
            status: "ok",
            run_dir: report.display().to_string(),
        })?;
    } else {
        println!("{}", report.display());
    }
    Ok(0)
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

struct BuiltInProfile {
    name: &'static str,
    kind: AgentKind,
    executable: &'static str,
    command: &'static str,
    detection_command: Option<&'static str>,
}

impl BuiltInProfile {
    fn is_detected(&self) -> bool {
        command_exists(self.executable) && self.detection_command.is_none_or(command_succeeds)
    }
}

fn default_profiles() -> Vec<BuiltInProfile> {
    vec![
        BuiltInProfile {
            name: "codex-default",
            kind: AgentKind::Codex,
            executable: "codex",
            command: "codex exec --full-auto -",
            detection_command: None,
        },
        BuiltInProfile {
            name: "claude-code-default",
            kind: AgentKind::ClaudeCode,
            executable: "claude",
            command: "claude -p -",
            detection_command: None,
        },
        BuiltInProfile {
            name: "opencode-default",
            kind: AgentKind::Opencode,
            executable: "opencode",
            command: "opencode run -",
            detection_command: None,
        },
        BuiltInProfile {
            name: "pi-coding-agent-default",
            kind: AgentKind::PiCodingAgent,
            executable: "pi",
            command: "pi -",
            detection_command: Some(
                "pi coding --version >/dev/null 2>&1 || pi --version >/dev/null 2>&1",
            ),
        },
    ]
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        fs::write(path, content)?;
    }
    Ok(())
}

fn resolve_home(home: Option<PathBuf>) -> PathBuf {
    home.or_else(|| std::env::var_os("HARNESSLAB_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".harnesslab")))
        .unwrap_or_else(|| PathBuf::from(".harnesslab"))
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
    fn app_001_overall_status_prioritizes_error_then_warning() {
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
}
