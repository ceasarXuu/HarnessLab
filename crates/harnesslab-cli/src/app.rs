use crate::output::{
    BenchmarkInfoOutput, BenchmarkListOutput, DoctorCheck, DoctorOutput, InitOutput, ListOutput,
    PathOutput,
};
use crate::runner::{execute_new_run, replay_run, resume_run};
use crate::{
    AgentCommand, BenchmarkCommand, Cli, Command, ReportCommand, RunAction, RunArgs, print_json,
};
use anyhow::{Context, Result};
use harnesslab_adapters::built_in_descriptors;
use harnesslab_core::{AgentKind, AgentProfile, GlobalConfig, default_agent_profile};
use harnesslab_infra::{DockerCliProvider, command_exists, first_command_word, latest_run_dir};
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
            BenchmarkCommand::List { json } => benchmark_list(json),
            BenchmarkCommand::Info { benchmark, json } => benchmark_info(&benchmark, json),
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
    for (name, kind, executable, command) in default_profiles() {
        if command_exists(executable) {
            detected.push(name.to_string());
        }
        let profile = default_agent_profile(name, kind, command);
        write_if_missing(
            &home.join("agents").join(format!("{name}.toml")),
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
    for profile in load_profiles(home).unwrap_or_default() {
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

fn benchmark_list(json: bool) -> Result<i32> {
    let descriptors = built_in_descriptors();
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

fn benchmark_info(name: &str, json: bool) -> Result<i32> {
    let descriptor = built_in_descriptors()
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

fn default_profiles() -> Vec<(&'static str, AgentKind, &'static str, &'static str)> {
    vec![
        (
            "codex-default",
            AgentKind::Codex,
            "codex",
            "codex exec --full-auto -",
        ),
        (
            "claude-code-default",
            AgentKind::ClaudeCode,
            "claude",
            "claude -p -",
        ),
        (
            "opencode-default",
            AgentKind::Opencode,
            "opencode",
            "opencode run -",
        ),
        (
            "pi-coding-agent-default",
            AgentKind::PiCodingAgent,
            "pi",
            "pi -",
        ),
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

fn check(id: &str, status: &str, severity: &str, message: &str) -> DoctorCheck {
    DoctorCheck {
        id: id.to_string(),
        status: status.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
        details: serde_json::json!({}),
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
