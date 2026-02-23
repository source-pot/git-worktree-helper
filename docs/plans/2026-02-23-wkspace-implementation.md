# wkspace Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI tool that manages git worktrees with lifecycle scripts and subshell access.

**Architecture:** Single binary with subcommands (init, new, rm, list, open). Config parsed from `.wkspace.toml` at repo root. Git operations via `std::process::Command` shelling out to `git`. Scripts run sequentially via the user's shell.

**Tech Stack:** Rust, clap (derive), serde + toml, anyhow, assert_cmd + assert_fs + predicates (testing)

**Design doc:** `docs/plans/2026-02-23-wkspace-design.md`

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/error.rs`
- Create: `src/config.rs`
- Create: `src/git.rs`
- Create: `src/scripts.rs`
- Create: `src/commands/mod.rs`
- Create: `src/commands/init.rs`
- Create: `src/commands/new.rs`
- Create: `src/commands/rm.rs`
- Create: `src/commands/list.rs`
- Create: `src/commands/open.rs`

**Step 1: Create Cargo.toml with all dependencies**

```toml
[package]
name = "wkspace"
version = "0.1.0"
edition = "2021"
description = "A CLI tool to manage Git worktrees with lifecycle scripts"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
assert_cmd = "2"
assert_fs = "1"
predicates = "3"
tempfile = "3"
```

**Step 2: Create src/main.rs with CLI skeleton**

```rust
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
```

**Step 3: Create stub modules**

`src/error.rs`:
```rust
use std::fmt;

#[derive(Debug)]
pub enum WkspaceError {
    NotAGitRepo,
    WorktreeExists(String),
    WorktreeNotFound(String),
    BranchExists(String),
    ScriptFailed { command: String, exit_code: Option<i32> },
    GitError(String),
}

impl fmt::Display for WkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAGitRepo => write!(f, "Not inside a git repository"),
            Self::WorktreeExists(name) => {
                write!(f, "Worktree '{name}' already exists. Use `wkspace open {name}` to open it")
            }
            Self::WorktreeNotFound(name) => write!(f, "Worktree '{name}' not found"),
            Self::BranchExists(name) => {
                write!(f, "Branch '{name}' already exists. Choose a different name")
            }
            Self::ScriptFailed { command, exit_code } => {
                write!(f, "Script failed: `{command}` (exit code: {})",
                    exit_code.map_or("unknown".to_string(), |c| c.to_string()))
            }
            Self::GitError(msg) => write!(f, "Git error: {msg}"),
        }
    }
}

impl std::error::Error for WkspaceError {}
```

`src/config.rs`:
```rust
pub struct Config;
```

`src/git.rs`:
```rust
```

`src/scripts.rs`:
```rust
```

`src/commands/mod.rs`:
```rust
pub mod init;
pub mod list;
pub mod new;
pub mod open;
pub mod rm;
```

`src/commands/init.rs`, `src/commands/new.rs`, `src/commands/rm.rs`, `src/commands/list.rs`, `src/commands/open.rs` (all identical stubs):
```rust
pub fn run() -> anyhow::Result<()> {
    todo!()
}
```

Note: `new.rs`, `rm.rs`, `open.rs` stubs take `&str`:
```rust
pub fn run(_name: &str) -> anyhow::Result<()> {
    todo!()
}
```

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: Compiles with no errors (warnings about unused code are fine).

**Step 5: Verify CLI parses --help**

Run: `cargo run -- --help`
Expected: Shows help text with all subcommands listed.

**Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: scaffold project with CLI skeleton and stub modules"
```

---

### Task 2: Git Repo Detection

**Files:**
- Modify: `src/git.rs`
- Create: `tests/git_test.rs`

**Step 1: Write the failing test**

`tests/git_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

#[test]
fn find_repo_root_in_git_repo() {
    let dir = TempDir::new().unwrap();
    Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let result = wkspace::git::find_repo_root(dir.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), dir.path().canonicalize().unwrap());
}

#[test]
fn find_repo_root_outside_git_repo() {
    let dir = TempDir::new().unwrap();
    let result = wkspace::git::find_repo_root(dir.path());
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test git_test`
Expected: FAIL — `find_repo_root` doesn't exist.

**Step 3: Add lib.rs and implement find_repo_root**

Create `src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod git;
pub mod scripts;
```

`src/git.rs`:
```rust
use crate::error::WkspaceError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Find the root of the git repository containing `start_dir`.
pub fn find_repo_root(start_dir: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(WkspaceError::NotAGitRepo);
    }

    let path = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    Ok(PathBuf::from(path))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test git_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/lib.rs src/git.rs tests/git_test.rs
git commit -m "feat: add git repo root detection"
```

---

### Task 3: Config Parsing

**Files:**
- Modify: `src/config.rs`
- Create: `tests/config_test.rs`

**Step 1: Write the failing tests**

`tests/config_test.rs`:
```rust
use wkspace::config::Config;

#[test]
fn parse_full_config() {
    let toml_str = r#"
[worktree]
base_branch = "develop"
directory = ".trees"

[scripts]
setup = ["npm install", "cp .env.example .env"]
teardown = ["echo cleanup"]
"#;
    let config = Config::parse(toml_str).unwrap();
    assert_eq!(config.worktree.base_branch, "develop");
    assert_eq!(config.worktree.directory, ".trees");
    assert_eq!(config.scripts.setup, vec!["npm install", "cp .env.example .env"]);
    assert_eq!(config.scripts.teardown, vec!["echo cleanup"]);
}

#[test]
fn default_config_has_sensible_values() {
    let config = Config::default();
    assert_eq!(config.worktree.base_branch, "main");
    assert_eq!(config.worktree.directory, ".worktrees");
    assert!(config.scripts.setup.is_empty());
    assert!(config.scripts.teardown.is_empty());
}

#[test]
fn default_template_is_valid_toml() {
    let template = Config::default_template();
    assert!(template.contains("base_branch"));
    assert!(template.contains(".worktrees"));
    // Verify it parses (strip comments first isn't needed — TOML supports comments)
    let config = Config::parse(&template).unwrap();
    assert_eq!(config.worktree.base_branch, "main");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test config_test`
Expected: FAIL — Config struct is just a placeholder.

**Step 3: Implement Config**

`src/config.rs`:
```rust
use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub scripts: ScriptsConfig,
}

#[derive(Debug, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default = "default_base_branch")]
    pub base_branch: String,
    #[serde(default = "default_directory")]
    pub directory: String,
}

#[derive(Debug, Deserialize)]
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

impl Default for Config {
    fn default() -> Self {
        Self {
            worktree: WorktreeConfig::default(),
            scripts: ScriptsConfig::default(),
        }
    }
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            base_branch: default_base_branch(),
            directory: default_directory(),
        }
    }
}

impl Default for ScriptsConfig {
    fn default() -> Self {
        Self {
            setup: Vec::new(),
            teardown: Vec::new(),
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
            let contents = std::fs::read_to_string(&config_path)
                .context("Failed to read .wkspace.toml")?;
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

[scripts]
# Commands to run after creating a worktree (runs in worktree directory)
setup = []

# Commands to run before removing a worktree (runs in worktree directory)
teardown = []
"#
        .to_string()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test config_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/config.rs tests/config_test.rs
git commit -m "feat: implement config parsing with defaults and template"
```

---

### Task 4: Script Runner

**Files:**
- Modify: `src/scripts.rs`
- Create: `tests/scripts_test.rs`

**Step 1: Write the failing tests**

`tests/scripts_test.rs`:
```rust
use tempfile::TempDir;
use wkspace::scripts;

#[test]
fn run_scripts_empty_list_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = scripts::run_scripts(&[], dir.path());
    assert!(result.is_ok());
}

#[test]
fn run_scripts_successful_commands() {
    let dir = TempDir::new().unwrap();
    let commands = vec!["echo hello".to_string(), "true".to_string()];
    let result = scripts::run_scripts(&commands, dir.path());
    assert!(result.is_ok());
}

#[test]
fn run_scripts_stops_on_first_failure() {
    let dir = TempDir::new().unwrap();
    // "false" exits with code 1, "echo after" should never run
    let marker = dir.path().join("marker");
    let commands = vec![
        "false".to_string(),
        format!("touch {}", marker.display()),
    ];
    let result = scripts::run_scripts(&commands, dir.path());
    assert!(result.is_err());
    assert!(!marker.exists(), "Second command should not have run");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test scripts_test`
Expected: FAIL — `run_scripts` doesn't exist.

**Step 3: Implement run_scripts**

`src/scripts.rs`:
```rust
use crate::error::WkspaceError;
use std::path::Path;
use std::process::Command;

/// Run a list of shell commands sequentially in the given directory.
/// Stops on first failure.
pub fn run_scripts(commands: &[String], cwd: &Path) -> anyhow::Result<()> {
    for cmd in commands {
        println!("  Running: {cmd}");
        let status = Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .status()?;

        if !status.success() {
            anyhow::bail!(WkspaceError::ScriptFailed {
                command: cmd.clone(),
                exit_code: status.code(),
            });
        }
    }
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test scripts_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/scripts.rs tests/scripts_test.rs
git commit -m "feat: implement script runner with stop-on-failure"
```

---

### Task 5: Git Worktree & Branch Operations

**Files:**
- Modify: `src/git.rs`
- Create: `tests/git_worktree_test.rs`

**Step 1: Write the failing tests**

`tests/git_worktree_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

#[test]
fn add_worktree_creates_directory_and_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    let wt_dir = dir.path().join(".worktrees").join("my-feature");
    let result = wkspace::git::add_worktree(dir.path(), &wt_dir, "my-feature", "main");
    assert!(result.is_ok());
    assert!(wt_dir.exists());
}

#[test]
fn add_worktree_fails_if_branch_exists() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    // Create the branch first
    Command::new("git").args(["branch", "existing"]).current_dir(dir.path()).output().unwrap();

    let wt_dir = dir.path().join(".worktrees").join("existing");
    let result = wkspace::git::add_worktree(dir.path(), &wt_dir, "existing", "main");
    assert!(result.is_err());
}

#[test]
fn branch_exists_returns_true_for_existing_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    assert!(wkspace::git::branch_exists(dir.path(), "main").unwrap());
}

#[test]
fn branch_exists_returns_false_for_missing_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    assert!(!wkspace::git::branch_exists(dir.path(), "nope").unwrap());
}

#[test]
fn delete_branch_removes_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    Command::new("git").args(["branch", "to-delete"]).current_dir(dir.path()).output().unwrap();

    let result = wkspace::git::delete_branch(dir.path(), "to-delete");
    assert!(result.is_ok());
    assert!(!wkspace::git::branch_exists(dir.path(), "to-delete").unwrap());
}

#[test]
fn prune_worktrees_succeeds() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    let result = wkspace::git::prune_worktrees(dir.path());
    assert!(result.is_ok());
}

#[test]
fn list_worktrees_returns_entries() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    let wt_dir = dir.path().join(".worktrees").join("feat-a");
    wkspace::git::add_worktree(dir.path(), &wt_dir, "feat-a", "main").unwrap();

    let entries = wkspace::git::list_worktrees(dir.path()).unwrap();
    // Should include the main worktree and our new one
    assert!(entries.len() >= 2);
    let feat = entries.iter().find(|e| e.branch.as_deref() == Some("feat-a"));
    assert!(feat.is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test git_worktree_test`
Expected: FAIL — functions don't exist.

**Step 3: Implement git worktree operations**

`src/git.rs`:
```rust
use crate::error::WkspaceError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Find the root of the git repository containing `start_dir`.
pub fn find_repo_root(start_dir: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(WkspaceError::NotAGitRepo);
    }

    let path = String::from_utf8(output.stdout)?
        .trim()
        .to_string();
    Ok(PathBuf::from(path))
}

/// Check if a local branch exists.
pub fn branch_exists(repo_root: &Path, branch: &str) -> anyhow::Result<bool> {
    let output = Command::new("git")
        .args(["branch", "--list", branch])
        .current_dir(repo_root)
        .output()?;
    Ok(!String::from_utf8(output.stdout)?.trim().is_empty())
}

/// Create a new worktree with a new branch based on `base_branch`.
pub fn add_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    base_branch: &str,
) -> anyhow::Result<()> {
    if branch_exists(repo_root, branch)? {
        anyhow::bail!(WkspaceError::BranchExists(branch.to_string()));
    }

    // Ensure parent directory exists
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_path.to_string_lossy(),
            "-b",
            branch,
            base_branch,
        ])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// Force-delete a branch.
pub fn delete_branch(repo_root: &Path, branch: &str) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["branch", "-D", branch])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// Prune stale worktree references.
pub fn prune_worktrees(repo_root: &Path) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// A parsed worktree entry from `git worktree list --porcelain`.
#[derive(Debug)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub bare: bool,
}

/// List all worktrees in the repository.
pub fn list_worktrees(repo_root: &Path) -> anyhow::Result<Vec<WorktreeEntry>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut entries = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut current_bare = false;

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            // Save previous entry if any
            if let Some(p) = current_path.take() {
                entries.push(WorktreeEntry {
                    path: p,
                    branch: current_branch.take(),
                    bare: current_bare,
                });
                current_bare = false;
            }
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch_ref) = line.strip_prefix("branch ") {
            // branch refs/heads/main -> main
            current_branch = branch_ref.strip_prefix("refs/heads/").map(|s| s.to_string());
        } else if line == "bare" {
            current_bare = true;
        }
    }

    // Don't forget the last entry
    if let Some(p) = current_path {
        entries.push(WorktreeEntry {
            path: p,
            branch: current_branch,
            bare: current_bare,
        });
    }

    Ok(entries)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test git_worktree_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/git.rs src/lib.rs tests/git_worktree_test.rs
git commit -m "feat: implement git worktree and branch operations"
```

---

### Task 6: Init Command

**Files:**
- Modify: `src/commands/init.rs`
- Create: `tests/cmd_init_test.rs`

**Step 1: Write the failing tests**

`tests/cmd_init_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn init_creates_config_file() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".wkspace.toml").exists());

    let contents = std::fs::read_to_string(dir.path().join(".wkspace.toml")).unwrap();
    assert!(contents.contains("base_branch"));
}

#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Run init twice
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    let output = wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();

    assert!(output.status.success());
}

#[test]
fn init_fails_outside_git_repo() {
    let dir = TempDir::new().unwrap();

    let output = wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("git repository") || stderr.contains("Not inside"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_init_test`
Expected: FAIL — init panics with `todo!()`.

**Step 3: Implement init command**

`src/commands/init.rs`:
```rust
use crate::config::Config;
use crate::git;
use std::env;
use std::path::Path;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::find_repo_root(&cwd)?;
    create_config(&repo_root)
}

/// Create .wkspace.toml at the repo root if it doesn't exist.
/// Returns Ok(()) if file already exists (idempotent).
pub fn create_config(repo_root: &Path) -> anyhow::Result<()> {
    let config_path = repo_root.join(".wkspace.toml");
    if config_path.exists() {
        println!(".wkspace.toml already exists");
        return Ok(());
    }

    std::fs::write(&config_path, Config::default_template())?;
    println!("Created .wkspace.toml with defaults");
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_init_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/init.rs tests/cmd_init_test.rs
git commit -m "feat: implement init command"
```

---

### Task 7: Auto-Init Helper & Config Loading

**Files:**
- Create: `src/context.rs`
- Modify: `src/lib.rs`

This task creates a shared helper that all commands (except init) use: find the repo root, auto-init if needed, load config.

**Step 1: Write the failing test**

Add to `tests/config_test.rs`:
```rust
use tempfile::TempDir;
use std::process::Command;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

#[test]
fn resolve_context_auto_creates_config() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // No .wkspace.toml exists yet
    assert!(!dir.path().join(".wkspace.toml").exists());

    let ctx = wkspace::context::resolve(dir.path()).unwrap();
    assert_eq!(ctx.config.worktree.base_branch, "main");

    // Config file should now exist
    assert!(dir.path().join(".wkspace.toml").exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test config_test resolve_context`
Expected: FAIL — module doesn't exist.

**Step 3: Implement context module**

`src/context.rs`:
```rust
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
```

Update `src/lib.rs`:
```rust
pub mod config;
pub mod context;
pub mod error;
pub mod git;
pub mod scripts;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test config_test resolve_context`
Expected: PASS

**Step 5: Commit**

```bash
git add src/context.rs src/lib.rs tests/config_test.rs
git commit -m "feat: add context module with auto-init config loading"
```

---

### Task 8: New Command

**Files:**
- Modify: `src/commands/new.rs`
- Create: `tests/cmd_new_test.rs`

**Step 1: Write the failing tests**

`tests/cmd_new_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn new_creates_worktree_and_branch() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // new will try to spawn a subshell — set SHELL to /usr/bin/true to skip it
    let output = wkspace_bin()
        .args(["new", "my-feature"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1") // test escape hatch
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".worktrees").join("my-feature").exists());

    // Verify branch was created
    let branches = Command::new("git")
        .args(["branch", "--list", "my-feature"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&branches.stdout).trim().is_empty());
}

#[test]
fn new_runs_setup_scripts() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Write config with a setup script that creates a marker file
    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = ["touch setup-ran"]
teardown = []
"#,
    )
    .unwrap();

    let output = wkspace_bin()
        .args(["new", "with-setup"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".worktrees/with-setup/setup-ran").exists());
}

#[test]
fn new_fails_if_worktree_exists() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "dupe"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["new", "dupe"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("open"));
}

#[test]
fn new_auto_inits_config() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // No .wkspace.toml exists
    let output = wkspace_bin()
        .args(["new", "auto-init"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".wkspace.toml").exists());
    assert!(dir.path().join(".worktrees/auto-init").exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_new_test`
Expected: FAIL — panics with `todo!()`.

**Step 3: Implement new command**

`src/commands/new.rs`:
```rust
use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::scripts;
use std::env;
use std::process::Command;

pub fn run(name: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    // Check if worktree directory already exists
    if worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeExists(name.to_string()));
    }

    // Create worktree + branch
    println!("Creating worktree '{name}' from '{}'...", ctx.config.worktree.base_branch);
    git::add_worktree(
        &ctx.repo_root,
        &worktree_path,
        name,
        &ctx.config.worktree.base_branch,
    )?;

    // Run setup scripts
    if !ctx.config.scripts.setup.is_empty() {
        println!("Running setup scripts...");
        scripts::run_scripts(&ctx.config.scripts.setup, &worktree_path)?;
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        spawn_shell(&worktree_path)?;
    }

    Ok(())
}

fn spawn_shell(cwd: &std::path::Path) -> anyhow::Result<()> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    println!("Opening shell in {}...", cwd.display());
    let mut child = Command::new(&shell)
        .current_dir(cwd)
        .spawn()?;
    child.wait()?;
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_new_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/new.rs tests/cmd_new_test.rs
git commit -m "feat: implement new command with worktree creation and setup scripts"
```

---

### Task 9: Rm Command

**Files:**
- Modify: `src/commands/rm.rs`
- Create: `tests/cmd_rm_test.rs`

**Step 1: Write the failing tests**

`tests/cmd_rm_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn rm_removes_worktree_and_branch() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Create a worktree first
    wkspace_bin()
        .args(["new", "to-remove"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(dir.path().join(".worktrees/to-remove").exists());

    // Remove it
    let output = wkspace_bin()
        .args(["rm", "to-remove"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!dir.path().join(".worktrees/to-remove").exists());

    // Branch should be gone
    let branches = Command::new("git")
        .args(["branch", "--list", "to-remove"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&branches.stdout).trim().is_empty());
}

#[test]
fn rm_handles_untracked_files() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "dirty"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Add an untracked file (like .env)
    std::fs::write(dir.path().join(".worktrees/dirty/.env"), "SECRET=123").unwrap();

    let output = wkspace_bin()
        .args(["rm", "dirty"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!dir.path().join(".worktrees/dirty").exists());
}

#[test]
fn rm_runs_teardown_scripts() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Config with teardown that creates a marker in repo root
    let marker = dir.path().join("teardown-ran");
    std::fs::write(
        dir.path().join(".wkspace.toml"),
        format!(
            r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = []
teardown = ["touch {}"]
"#,
            marker.display()
        ),
    )
    .unwrap();

    wkspace_bin()
        .args(["new", "with-teardown"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    wkspace_bin()
        .args(["rm", "with-teardown"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(marker.exists());
}

#[test]
fn rm_fails_for_nonexistent_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["rm", "nope"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn rm_stops_on_teardown_failure() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = []
teardown = ["false"]
"#,
    )
    .unwrap();

    wkspace_bin()
        .args(["new", "fail-teardown"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["rm", "fail-teardown"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    // Worktree should still exist since teardown failed
    assert!(dir.path().join(".worktrees/fail-teardown").exists());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_rm_test`
Expected: FAIL — panics with `todo!()`.

**Step 3: Implement rm command**

`src/commands/rm.rs`:
```rust
use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::scripts;
use std::env;

pub fn run(name: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    // Validate worktree exists
    if !worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeNotFound(name.to_string()));
    }

    // Run teardown scripts (stop on failure)
    if !ctx.config.scripts.teardown.is_empty() {
        println!("Running teardown scripts...");
        scripts::run_scripts(&ctx.config.scripts.teardown, &worktree_path)?;
    }

    // Force-remove the worktree directory
    println!("Removing worktree '{name}'...");
    std::fs::remove_dir_all(&worktree_path)?;

    // Prune stale worktree references
    git::prune_worktrees(&ctx.repo_root)?;

    // Delete the branch
    git::delete_branch(&ctx.repo_root, name)?;

    println!("Worktree '{name}' removed");
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_rm_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/rm.rs tests/cmd_rm_test.rs
git commit -m "feat: implement rm command with teardown and force cleanup"
```

---

### Task 10: List Command

**Files:**
- Modify: `src/commands/list.rs`
- Create: `tests/cmd_list_test.rs`

**Step 1: Write the failing tests**

`tests/cmd_list_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn list_shows_no_worktrees_initially() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No worktrees") || stdout.trim().is_empty() || !stdout.contains("my-feature"));
}

#[test]
fn list_shows_created_worktrees() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "feat-a"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    wkspace_bin()
        .args(["new", "feat-b"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("feat-a"));
    assert!(stdout.contains("feat-b"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_list_test`
Expected: FAIL — panics with `todo!()`.

**Step 3: Implement list command**

`src/commands/list.rs`:
```rust
use crate::context;
use crate::git;
use std::env;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktrees_dir = ctx.worktrees_dir();

    let entries = git::list_worktrees(&ctx.repo_root)?;

    // Filter to only managed worktrees (those under the worktrees directory)
    let managed: Vec<_> = entries
        .iter()
        .filter(|e| e.path.starts_with(&worktrees_dir))
        .collect();

    if managed.is_empty() {
        println!("No worktrees");
        return Ok(());
    }

    for entry in &managed {
        let name = entry
            .path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        let branch = entry.branch.as_deref().unwrap_or("(detached)");
        println!("  {name}\t{branch}\t{}", entry.path.display());
    }

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_list_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/list.rs tests/cmd_list_test.rs
git commit -m "feat: implement list command showing managed worktrees"
```

---

### Task 11: Open Command

**Files:**
- Modify: `src/commands/open.rs`
- Create: `tests/cmd_open_test.rs`

**Step 1: Write the failing tests**

`tests/cmd_open_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn open_fails_for_nonexistent_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["open", "nope"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Worktree"));
}

#[test]
fn open_succeeds_for_existing_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "existing"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Use WKSPACE_NO_SHELL to avoid actually spawning a shell in tests
    let output = wkspace_bin()
        .args(["open", "existing"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_open_test`
Expected: FAIL — panics with `todo!()`.

**Step 3: Implement open command**

`src/commands/open.rs`:
```rust
use crate::context;
use crate::error::WkspaceError;
use std::env;
use std::process::Command;

pub fn run(name: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    if !worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeNotFound(name.to_string()));
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        println!("Opening shell in {}...", worktree_path.display());
        let mut child = Command::new(&shell)
            .current_dir(&worktree_path)
            .spawn()?;
        child.wait()?;
    }

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_open_test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/commands/open.rs tests/cmd_open_test.rs
git commit -m "feat: implement open command to shell into existing worktree"
```

---

### Task 12: Add .worktrees to .gitignore

**Files:**
- Modify: `src/commands/init.rs`
- Modify: `tests/cmd_init_test.rs`

**Step 1: Write the failing test**

Add to `tests/cmd_init_test.rs`:
```rust
#[test]
fn init_adds_worktrees_dir_to_gitignore() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".worktrees"));
}

#[test]
fn init_does_not_duplicate_gitignore_entry() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Run init twice
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    let count = gitignore.matches(".worktrees").count();
    assert_eq!(count, 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cmd_init_test init_adds_worktrees`
Expected: FAIL — no .gitignore handling.

**Step 3: Add .gitignore handling to init**

In `src/commands/init.rs`, add to the `create_config` function (after writing the config file), and also ensure the auto-init path in `context.rs` calls this:

`src/commands/init.rs` (full updated file):
```rust
use crate::config::Config;
use crate::git;
use std::env;
use std::path::Path;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let repo_root = git::find_repo_root(&cwd)?;
    create_config(&repo_root)?;
    ensure_gitignore(&repo_root, ".worktrees")?;
    Ok(())
}

/// Create .wkspace.toml at the repo root if it doesn't exist.
pub fn create_config(repo_root: &Path) -> anyhow::Result<()> {
    let config_path = repo_root.join(".wkspace.toml");
    if config_path.exists() {
        println!(".wkspace.toml already exists");
        return Ok(());
    }

    std::fs::write(&config_path, Config::default_template())?;
    println!("Created .wkspace.toml with defaults");
    Ok(())
}

/// Ensure the worktrees directory is in .gitignore.
pub fn ensure_gitignore(repo_root: &Path, entry: &str) -> anyhow::Result<()> {
    let gitignore_path = repo_root.join(".gitignore");
    let contents = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    if contents.lines().any(|line| line.trim() == entry) {
        return Ok(());
    }

    let new_contents = if contents.is_empty() || contents.ends_with('\n') {
        format!("{contents}{entry}\n")
    } else {
        format!("{contents}\n{entry}\n")
    };
    std::fs::write(&gitignore_path, new_contents)?;
    Ok(())
}
```

Also update `src/context.rs` to call `ensure_gitignore` during auto-init:

```rust
use crate::commands::init;
use crate::config::Config;
use crate::git;
use std::path::{Path, PathBuf};

pub struct Context {
    pub repo_root: PathBuf,
    pub config: Config,
}

impl Context {
    pub fn worktrees_dir(&self) -> PathBuf {
        self.repo_root.join(&self.config.worktree.directory)
    }

    pub fn worktree_path(&self, name: &str) -> PathBuf {
        self.worktrees_dir().join(name)
    }
}

pub fn resolve(start_dir: &Path) -> anyhow::Result<Context> {
    let repo_root = git::find_repo_root(start_dir)?;
    let config_path = repo_root.join(".wkspace.toml");

    if !config_path.exists() {
        std::fs::write(&config_path, Config::default_template())?;
        println!("Created .wkspace.toml with defaults");
    }

    let config = Config::load(&repo_root)?;
    init::ensure_gitignore(&repo_root, &config.worktree.directory)?;

    Ok(Context { repo_root, config })
}
```

Note: This creates a dependency from `context` → `commands::init`. To keep things clean, move `ensure_gitignore` to a shared location if this bothers you, but for this project size it's fine.

Update `src/main.rs` to add `mod commands` visibility. Since `commands` is already a module in `main.rs`, and `context.rs` needs it from the lib, add `commands` to `src/lib.rs`:

```rust
pub mod commands;
pub mod config;
pub mod context;
pub mod error;
pub mod git;
pub mod scripts;
```

And in `src/main.rs`, use the lib crate:

```rust
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
        Commands::Init => wkspace::commands::init::run(),
        Commands::New { name } => wkspace::commands::new::run(&name),
        Commands::Rm { name } => wkspace::commands::rm::run(&name),
        Commands::List => wkspace::commands::list::run(),
        Commands::Open { name } => wkspace::commands::open::run(&name),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test cmd_init_test`
Expected: PASS

**Step 5: Run all tests**

Run: `cargo test`
Expected: All tests PASS.

**Step 6: Commit**

```bash
git add src/ tests/
git commit -m "feat: add .worktrees to .gitignore on init and auto-init"
```

---

### Task 13: Final Integration Test & Polish

**Files:**
- Create: `tests/integration_test.rs`

**Step 1: Write end-to-end integration test**

`tests/integration_test.rs`:
```rust
use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

/// Full lifecycle: init → new → list → rm → list
#[test]
fn full_lifecycle() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // init
    let out = wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(dir.path().join(".wkspace.toml").exists());
    assert!(dir.path().join(".gitignore").exists());

    // new
    let out = wkspace_bin()
        .args(["new", "feat-x"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(dir.path().join(".worktrees/feat-x").exists());

    // list shows feat-x
    let out = wkspace_bin().args(["list"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("feat-x"));

    // rm
    let out = wkspace_bin().args(["rm", "feat-x"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(!dir.path().join(".worktrees/feat-x").exists());

    // list shows no worktrees
    let out = wkspace_bin().args(["list"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("feat-x"));
}
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: All PASS.

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add full lifecycle integration test"
```

---

### Task 14: Build Release Binary

**Step 1: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/wkspace`.

**Step 2: Verify binary runs**

Run: `./target/release/wkspace --help`
Expected: Shows help with all subcommands.

**Step 3: Commit any final changes and tag**

```bash
git tag v0.1.0
```
