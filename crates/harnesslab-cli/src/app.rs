use crate::agent_registry::{agents_readme, profile_template};
use crate::benchmark_data::resolve_benchmarks_dir;
use crate::output::{
    AgentSchemaOutput, BenchmarkInfoOutput, BenchmarkListOutput, InitOutput, ListOutput, PathOutput,
};
use crate::runner::{RunOverrides, execute_new_run, replay_run, resume_run};
use crate::{
    AgentCommand, BenchmarkCommand, Cli, Command, ReportCommand, RunAction, RunArgs, print_json,
};
use anyhow::{Context, Result};
use harnesslab_adapters::built_in_descriptors_with_root;
use harnesslab_core::{AgentKind, AgentProfile, GlobalConfig, data_state_blocks_run};
use harnesslab_infra::{command_exists, command_succeeds, latest_run_dir, validate_event_log};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn dispatch(cli: Cli) -> Result<i32> {
    let home = resolve_home(cli.home);
    match cli.command {
        Command::Init { json } => init(&home, json),
        Command::Agent { command } => match command {
            AgentCommand::List { json } => agent_list(&home, json),
            AgentCommand::Schema { json } => agent_schema(json),
        },
        Command::Doctor { json } => crate::doctor::run(&home, json),
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
        write_if_missing(
            &home.join("agents").join(format!("{}.toml", profile.name)),
            &profile_template(profile.name, profile.kind, profile.command),
        )?;
    }
    write_if_missing(&home.join("agents").join("README.md"), agents_readme())?;

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

fn agent_schema(json: bool) -> Result<i32> {
    let fields = harnesslab_core::agent_profile_field_reference();
    if json {
        print_json(&AgentSchemaOutput {
            schema_version: 1,
            command: "agent schema",
            status: "ok",
            fields,
        })?;
    } else {
        println!("Agent profile fields:");
        for field in fields {
            let values = if field.allowed_values.is_empty() {
                "string".to_string()
            } else {
                field.allowed_values.join(" | ")
            };
            println!("  - {}: {}", field.path, values);
        }
    }
    Ok(0)
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
            let splits = descriptor
                .splits
                .iter()
                .map(|split| format!("{}:{}:{}", split.name, split.task_count, split.data_state))
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{} ({:?}) version={} splits=[{}]",
                descriptor.name, descriptor.style, descriptor.version, splits
            );
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
        println!("  style: {:?}", descriptor.style);
        println!("  version: {}", descriptor.version);
        println!("  homepage: {}", descriptor.homepage);
        println!("  splits:");
        for split in &descriptor.splits {
            println!(
                "    - {}: tasks={}, data_state={}",
                split.name, split.task_count, split.data_state
            );
        }
        if let Some(split) = descriptor
            .splits
            .iter()
            .find(|split| !data_state_blocks_run(split.data_state))
        {
            println!(
                "  run: harnesslab run --agent <agent-profile> --benchmark {} --split {}",
                descriptor.name, split.name
            );
        }
    }
    Ok(0)
}

fn run_command(home: &Path, args: RunArgs) -> Result<i32> {
    match args.action {
        Some(RunAction::Resume { run_dir, json }) => {
            validate_run_event_log(&run_dir)?;
            resume_run(home, &run_dir, json)
        }
        Some(RunAction::Replay { run_dir, json }) => {
            validate_run_event_log(&run_dir)?;
            replay_run(home, &run_dir, json)
        }
        None => {
            let agent = args.agent.context("--agent is required")?;
            let benchmark = args.benchmark.context("--benchmark is required")?;
            let split = args.split.context("--split is required")?;
            execute_new_run(
                home,
                &agent,
                &benchmark,
                &split,
                args.json,
                RunOverrides {
                    concurrency: args.concurrency,
                    attempts: args.attempts,
                    timeout_sec: args.timeout_sec,
                },
                None,
            )
        }
    }
}

fn report_open(home: &Path, target: &str, json: bool) -> Result<i32> {
    let run_dir = if target == "latest" {
        let config = load_config(home).ok();
        let runs_dir = config.as_ref().map_or_else(
            || home.join("runs"),
            |config| crate::runner::runs_dir(home, config),
        );
        latest_run_dir(&runs_dir)?.context("no runs found")?
    } else {
        PathBuf::from(target)
    };
    validate_run_event_log(&run_dir)?;
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

fn validate_run_event_log(run_dir: &Path) -> Result<()> {
    validate_event_log(&run_dir.join("events.jsonl"))
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
