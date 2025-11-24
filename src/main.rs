mod models;
mod storage;
mod tui;
mod git;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tasktui")]
#[command(about = "A CLI/TUI Task Manager with MCP support", long_about = None)]
struct Cli {
    /// Data directory for task files
    #[arg(short, long, default_value = "./tasks")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run in MCP server mode
    Server,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Server) => {
            println!("MCP server mode not yet implemented");
            Ok(())
        }
        None => {
            // Run TUI mode
            tui::run(cli.data_dir)
        }
    }
}
