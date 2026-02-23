use crate::config::Config;
use crate::git;
use std::path::{Path, PathBuf};

/// Resolved context for a wkspace command: repo root + loaded config.
pub struct Context {
    pub repo_root: PathBuf,
    pub config: Config,
}

impl Context {
    /// Get the worktree directory path.
    pub fn worktrees_dir(&self) -> PathBuf {
        self.repo_root.join(&self.config.worktree.directory)
    }

    /// Get the path to a specific worktree.
    pub fn worktree_path(&self, name: &str) -> PathBuf {
        self.worktrees_dir().join(name)
    }
}

/// Find repo root, auto-create config if missing, load config.
pub fn resolve(start_dir: &Path) -> anyhow::Result<Context> {
    let repo_root = git::find_repo_root(start_dir)?;
    let config_path = repo_root.join(".wkspace.toml");

    if !config_path.exists() {
        std::fs::write(&config_path, Config::default_template())?;
        println!("Created .wkspace.toml with defaults");
    }

    let config = Config::load(&repo_root)?;
    Ok(Context { repo_root, config })
}
