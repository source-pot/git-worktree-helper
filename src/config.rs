use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub scripts: ScriptsConfig,
    #[serde(default)]
    pub ports: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default = "default_base_branch")]
    pub base_branch: String,
    #[serde(default = "default_directory")]
    pub directory: String,
    #[serde(default = "default_stale_days")]
    pub stale_days: u64,
}

#[derive(Debug, Default, Deserialize)]
pub struct ScriptsConfig {
    #[serde(default)]
    pub setup: Vec<String>,
    #[serde(default)]
    pub teardown: Vec<String>,
}

fn default_base_branch() -> String {
    "main".to_string()
}

fn default_directory() -> String {
    ".worktrees".to_string()
}

fn default_stale_days() -> u64 {
    7
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            base_branch: default_base_branch(),
            directory: default_directory(),
            stale_days: default_stale_days(),
        }
    }
}

impl Config {
    /// Parse a TOML string into a Config.
    pub fn parse(toml_str: &str) -> anyhow::Result<Self> {
        toml::from_str(toml_str).context("Failed to parse .wkspace.toml")
    }

    /// Load config from a .wkspace.toml file at the given repo root.
    /// Returns default config if the file doesn't exist.
    pub fn load(repo_root: &Path) -> anyhow::Result<Self> {
        let config_path = repo_root.join(".wkspace.toml");
        if config_path.exists() {
            let contents =
                std::fs::read_to_string(&config_path).context("Failed to read .wkspace.toml")?;
            Self::parse(&contents)
        } else {
            Ok(Self::default())
        }
    }

    /// Return the default .wkspace.toml content with comments.
    pub fn default_template() -> String {
        r#"[worktree]
# Branch that new worktrees are based on
base_branch = "main"

# Directory (relative to repo root) where worktrees are stored
directory = ".worktrees"

# Days since last commit before a worktree is marked "stale"
# stale_days = 7

[scripts]
# Commands to run after creating a worktree (runs in worktree directory)
setup = []

# Commands to run before removing a worktree (runs in worktree directory)
teardown = []

# [ports]
# Allocate random available ports and expose as env vars to scripts and shell
# Format: label = "ENV_VAR_NAME"
# frontend_port = "FRONTEND_PORT"
# backend_port = "BACKEND_PORT"

# Note: $WORKTREE_NAME is automatically available in all scripts and the shell
"#
        .to_string()
    }
}
