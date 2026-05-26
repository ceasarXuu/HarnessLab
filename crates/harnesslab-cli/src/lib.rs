use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { json } => print_json_or_text(json, "init", "ok"),
        Command::Agent { command } => match command {
            AgentCommand::List { json } => print_json_or_text(json, "agent list", "ok"),
        },
        Command::Doctor { json } => print_doctor(json),
        Command::Benchmark { command } => match command {
            BenchmarkCommand::List { json } => print_json_or_text(json, "benchmark list", "ok"),
            BenchmarkCommand::Info { benchmark, json } => {
                print_named_json_or_text(json, "benchmark info", &benchmark)
            }
        },
        Command::Run(args) => match args.action {
            Some(RunAction::Resume { run_dir, json }) => {
                print_path_json_or_text(json, "run resume", run_dir)
            }
            Some(RunAction::Replay { run_dir, json }) => {
                print_path_json_or_text(json, "run replay", run_dir)
            }
            None => {
                let agent = args.agent.unwrap_or_else(|| "missing-agent".to_string());
                let benchmark = args
                    .benchmark
                    .unwrap_or_else(|| "missing-benchmark".to_string());
                let split = args.split.unwrap_or_else(|| "missing-split".to_string());
                if args.json {
                    print_json(&RunStartOutput {
                        schema_version: 1,
                        command: "run",
                        status: "accepted",
                        agent,
                        benchmark,
                        split,
                    })
                } else {
                    println!("run accepted: agent={agent} benchmark={benchmark} split={split}");
                    Ok(())
                }
            }
        },
        Command::Report { command } => match command {
            ReportCommand::Open { target, json } => {
                print_named_json_or_text(json, "report open", &target)
            }
        },
    }
}

#[derive(Debug, Parser)]
#[command(name = "harnesslab")]
#[command(about = "Harness evaluation lab for CLI agents", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        json: bool,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Benchmark {
        #[command(subcommand)]
        command: BenchmarkCommand,
    },
    Run(RunArgs),
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
}

#[derive(Debug, Subcommand)]
enum AgentCommand {
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum BenchmarkCommand {
    List {
        #[arg(long)]
        json: bool,
    },
    Info {
        benchmark: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Args)]
struct RunArgs {
    #[command(subcommand)]
    action: Option<RunAction>,
    #[arg(long)]
    agent: Option<String>,
    #[arg(long)]
    benchmark: Option<String>,
    #[arg(long)]
    split: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum RunAction {
    Resume {
        run_dir: PathBuf,
        #[arg(long)]
        json: bool,
    },
    Replay {
        run_dir: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ReportCommand {
    Open {
        target: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct SimpleOutput<'a> {
    schema_version: u32,
    command: &'a str,
    status: &'a str,
}

#[derive(Serialize)]
struct NamedOutput<'a> {
    schema_version: u32,
    command: &'a str,
    status: &'a str,
    name: &'a str,
}

#[derive(Serialize)]
struct PathOutput<'a> {
    schema_version: u32,
    command: &'a str,
    status: &'a str,
    run_dir: String,
}

#[derive(Serialize)]
struct DoctorOutput<'a> {
    schema_version: u32,
    status: &'a str,
    checks: Vec<DoctorCheck<'a>>,
}

#[derive(Serialize)]
struct DoctorCheck<'a> {
    id: &'a str,
    status: &'a str,
    severity: &'a str,
    message: &'a str,
    details: serde_json::Value,
}

#[derive(Serialize)]
struct RunStartOutput {
    schema_version: u32,
    command: &'static str,
    status: &'static str,
    agent: String,
    benchmark: String,
    split: String,
}

fn print_doctor(json: bool) -> Result<()> {
    if json {
        print_json(&DoctorOutput {
            schema_version: 1,
            status: "ok",
            checks: vec![DoctorCheck {
                id: "m0.cli",
                status: "ok",
                severity: "info",
                message: "M0 CLI skeleton is available",
                details: serde_json::json!({}),
            }],
        })
    } else {
        println!("doctor: ok");
        Ok(())
    }
}

fn print_json_or_text(json: bool, command: &'static str, status: &'static str) -> Result<()> {
    if json {
        print_json(&SimpleOutput {
            schema_version: 1,
            command,
            status,
        })
    } else {
        println!("{command}: {status}");
        Ok(())
    }
}

fn print_named_json_or_text(json: bool, command: &'static str, name: &str) -> Result<()> {
    if json {
        print_json(&NamedOutput {
            schema_version: 1,
            command,
            status: "ok",
            name,
        })
    } else {
        println!("{command}: {name}");
        Ok(())
    }
}

fn print_path_json_or_text(json: bool, command: &'static str, run_dir: PathBuf) -> Result<()> {
    if json {
        print_json(&PathOutput {
            schema_version: 1,
            command,
            status: "accepted",
            run_dir: run_dir.display().to_string(),
        })
    } else {
        println!("{command}: {}", run_dir.display());
        Ok(())
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}
