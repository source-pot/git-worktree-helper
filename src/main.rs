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
        /// Name for the worktree and branch (prompted if omitted)
        name: Option<String>,
        /// Description for the worktree branch
        #[arg(short, long)]
        desc: Option<String>,
        /// Skip spawning an interactive shell after setup
        #[arg(long)]
        no_shell: bool,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },
    /// Create a worktree from an existing branch
    From {
        /// Branch name (interactive picker if omitted)
        branch: Option<String>,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },
    /// Run teardown scripts and remove a worktree and its branch
    Rm {
        /// Name of the worktree to remove (interactive picker if omitted)
        name: Option<String>,
        /// Skip confirmation prompt for uncommitted changes
        #[arg(short, long)]
        force: bool,
        /// Skip running lifecycle scripts
        #[arg(long)]
        no_scripts: bool,
    },
    /// List active worktrees
    List,
    /// Open a shell in an existing worktree
    Open {
        /// Name of the worktree to open (interactive picker if omitted)
        name: Option<String>,
    },
    /// Re-run setup scripts in the current worktree
    Setup,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => wkspace::commands::init::run(),
        Commands::New {
            name,
            desc,
            no_shell,
            no_scripts,
        } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::new::prompt_name()?,
            };
            wkspace::commands::new::run(&name, desc.as_deref(), no_shell, no_scripts)
        }
        Commands::From { branch, no_scripts } => {
            let branch = match branch {
                Some(b) => b,
                None => wkspace::commands::from::pick_branch()?,
            };
            wkspace::commands::from::run(&branch, no_scripts)
        }
        Commands::Rm {
            name,
            force,
            no_scripts,
        } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::pick_worktree("Select worktree to remove")?,
            };
            wkspace::commands::rm::run(&name, force, no_scripts)
        }
        Commands::List => wkspace::commands::list::run(),
        Commands::Open { name } => {
            let name = match name {
                Some(n) => n,
                None => wkspace::commands::pick_worktree("Select worktree to open")?,
            };
            wkspace::commands::open::run(&name)
        }
        Commands::Setup => wkspace::commands::setup::run(),
    }
}
