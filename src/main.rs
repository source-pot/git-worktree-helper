use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wkspace",
    about = "Manage Git worktrees with lifecycle scripts"
)]
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
        /// Name for the worktree and branch (auto-generated if omitted)
        name: Option<String>,
    },
    /// Run teardown scripts and remove a worktree and its branch
    Rm {
        /// Name of the worktree to remove (interactive picker if omitted)
        name: Option<String>,
    },
    /// List active worktrees
    List,
    /// Open a shell in an existing worktree
    Open {
        /// Name of the worktree to open
        name: String,
    },
    /// Re-run setup scripts in the current worktree
    Setup,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => wkspace::commands::init::run(),
        Commands::New { name } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::names::generate_unique_name()?,
            };
            wkspace::commands::new::run(&name)
        }
        Commands::Rm { name } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::rm::pick_worktree()?,
            };
            wkspace::commands::rm::run(&name)
        }
        Commands::List => wkspace::commands::list::run(),
        Commands::Open { name } => wkspace::commands::open::run(&name),
        Commands::Setup => wkspace::commands::setup::run(),
    }
}
