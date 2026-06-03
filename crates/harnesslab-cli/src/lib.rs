mod agent_registry;
mod app;
mod benchmark_data;
mod doctor;
mod doctor_capabilities;
mod output;
mod runner;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

pub fn run() -> Result<i32> {
    let cli = Cli::parse();
    app::dispatch(cli)
}

#[derive(Debug, Parser)]
#[command(name = "harnesslab")]
#[command(about = "Harness evaluation lab for CLI agents", version)]
pub(crate) struct Cli {
    #[arg(long, global = true)]
    home: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
pub(crate) enum AgentCommand {
    List {
        #[arg(long)]
        json: bool,
    },
    Schema {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum BenchmarkCommand {
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
pub(crate) struct RunArgs {
    #[command(subcommand)]
    pub action: Option<RunAction>,
    #[arg(long)]
    pub agent: Option<String>,
    #[arg(long)]
    pub benchmark: Option<String>,
    #[arg(long)]
    pub split: Option<String>,
    #[arg(long)]
    pub concurrency: Option<usize>,
    #[arg(long)]
    pub attempts: Option<u32>,
    #[arg(long)]
    pub timeout_sec: Option<u64>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RunAction {
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
pub(crate) enum ReportCommand {
    Open {
        target: String,
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn print_json<T: Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}
