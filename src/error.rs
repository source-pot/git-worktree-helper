use std::fmt;

#[derive(Debug)]
pub enum WkspaceError {
    NotAGitRepo,
    WorktreeExists(String),
    WorktreeNotFound(String),
    BranchExists(String),
    ScriptFailed {
        command: String,
        exit_code: Option<i32>,
    },
    GitError(String),
    NotAWorktree,
    ConfigNotFound,
}

impl fmt::Display for WkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAGitRepo => write!(f, "Not inside a git repository"),
            Self::WorktreeExists(name) => {
                write!(
                    f,
                    "Worktree '{name}' already exists. Use `wkspace open {name}` to open it"
                )
            }
            Self::WorktreeNotFound(name) => write!(f, "Worktree '{name}' not found"),
            Self::BranchExists(name) => {
                write!(f, "Branch '{name}' already exists. Choose a different name")
            }
            Self::ScriptFailed { command, exit_code } => {
                write!(
                    f,
                    "Script failed: `{command}` (exit code: {})",
                    exit_code.map_or("unknown".to_string(), |c| c.to_string())
                )
            }
            Self::GitError(msg) => write!(f, "Git error: {msg}"),
            Self::NotAWorktree => {
                write!(f, "Current directory is not inside a git worktree")
            }
            Self::ConfigNotFound => {
                write!(f, "No .wkspace.toml found. Run 'wkspace init' first")
            }
        }
    }
}

impl std::error::Error for WkspaceError {}
