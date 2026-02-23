mod commands;
mod config;
mod error;
mod git;
mod scripts;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wkspace", about = "Manage Git worktrees with lifecycle scripts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create .wkspace.toml with default configuration
    Init,
    /// Create a new worktree with a branch, run setup scripts, and open a shell
    New {
        /// Name for the worktree and branch
        name: String,
    },
    /// Run teardown scripts and remove a worktree and its branch
    Rm {
        /// Name of the worktree to remove
        name: String,
    },
    /// List active worktrees
    List,
    /// Open a shell in an existing worktree
    Open {
        /// Name of the worktree to open
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::New { name } => commands::new::run(&name),
        Commands::Rm { name } => commands::rm::run(&name),
        Commands::List => commands::list::run(),
        Commands::Open { name } => commands::open::run(&name),
    }
}
