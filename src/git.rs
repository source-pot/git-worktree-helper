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

    let path = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(path))
}

/// Find the main repository root (not the worktree toplevel).
/// Uses git-common-dir which points to the shared .git directory.
pub fn find_main_repo_root(start_dir: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(start_dir)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(WkspaceError::NotAGitRepo);
    }

    let git_common_dir = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    // git-common-dir returns the .git directory; parent is the repo root
    git_common_dir
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!(WkspaceError::NotAGitRepo))
}

/// Get the current worktree name, or error if not inside a worktree.
/// Inside a worktree, `git rev-parse --git-dir` returns `.git/worktrees/<name>`.
/// In the main working tree, it returns `.git`.
pub fn current_worktree_name(cwd: &Path) -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-dir"])
        .current_dir(cwd)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(WkspaceError::NotAGitRepo);
    }

    let git_dir = PathBuf::from(String::from_utf8(output.stdout)?.trim());

    // In a worktree: <repo>/.git/worktrees/<name>
    // In main tree: <repo>/.git
    let file_name = git_dir.file_name().and_then(|f| f.to_str()).unwrap_or("");

    if file_name == ".git" {
        anyhow::bail!(WkspaceError::NotAWorktree);
    }

    // Verify parent is "worktrees" directory
    let parent_name = git_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|f| f.to_str())
        .unwrap_or("");

    if parent_name != "worktrees" {
        anyhow::bail!(WkspaceError::NotAWorktree);
    }

    Ok(file_name.to_string())
}

/// Fetch the latest state of a branch from origin and fast-forward the local ref.
///
/// Skips the update if the local branch has unpushed commits (is ahead of remote).
/// If the fetch fails (no remote, offline), prints a warning and continues.
pub fn fetch_and_update_branch(repo_root: &Path, branch: &str) {
    // Fetch latest from origin
    let fetch = Command::new("git")
        .args(["fetch", "origin", branch])
        .current_dir(repo_root)
        .output();

    let Ok(fetch_output) = fetch else {
        eprintln!("Warning: failed to fetch '{branch}' from origin, using local branch");
        return;
    };

    if !fetch_output.status.success() {
        eprintln!("Warning: failed to fetch '{branch}' from origin, using local branch");
        return;
    }

    // Check if local branch is ahead of remote (has unpushed work)
    let ancestor_check = Command::new("git")
        .args([
            "merge-base",
            "--is-ancestor",
            &format!("origin/{branch}"),
            branch,
        ])
        .current_dir(repo_root)
        .output();

    if let Ok(output) = ancestor_check {
        if output.status.success() {
            // origin/branch is ancestor of local branch — local is ahead or equal, skip update
            return;
        }
    }

    // Fast-forward local branch to match remote
    let remote_ref = format!("origin/{branch}");
    let update = Command::new("git")
        .args(["update-ref", &format!("refs/heads/{branch}"), &remote_ref])
        .current_dir(repo_root)
        .output();

    match update {
        Ok(output) if output.status.success() => {
            println!("Updated '{branch}' to match origin.");
        }
        _ => {
            eprintln!("Warning: failed to update local '{branch}', using current state");
        }
    }
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

/// Remove a worktree directory and its git metadata in one step.
/// Retries up to 3 times to handle transient macOS filesystem errors
/// (Spotlight indexing, .DS_Store creation during deletion).
pub fn remove_worktree(repo_root: &Path, worktree_path: &Path) -> anyhow::Result<()> {
    let path_str = worktree_path.to_string_lossy();
    let args = ["worktree", "remove", "--force", &path_str];

    for attempt in 0..3 {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_root)
            .output()?;

        if output.status.success() {
            return Ok(());
        }

        if attempt < 2 {
            std::thread::sleep(std::time::Duration::from_millis(500));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
        }
    }
    unreachable!()
}

/// Set the description for a branch using git config.
pub fn set_branch_description(repo_root: &Path, branch: &str, desc: &str) -> anyhow::Result<()> {
    let output = Command::new("git")
        .args(["config", &format!("branch.{branch}.description"), desc])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }
    Ok(())
}

/// Get the description for a branch, if set.
pub fn get_branch_description(repo_root: &Path, branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", &format!("branch.{branch}.description")])
        .current_dir(repo_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let desc = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if desc.is_empty() {
        None
    } else {
        Some(desc)
    }
}

/// Status summary for a worktree's working directory.
pub struct WorktreeStatus {
    pub uncommitted_count: usize,
    pub files: Vec<String>,
}

/// Get the working directory status for a worktree path.
pub fn get_worktree_status(worktree_path: &Path) -> anyhow::Result<WorktreeStatus> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let files: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l[3..].to_string())
        .collect();
    Ok(WorktreeStatus {
        uncommitted_count: files.len(),
        files,
    })
}

/// Get the last commit time for a branch.
/// Returns (relative_time, unix_timestamp) e.g. ("3 hours ago", 1700000000).
pub fn get_last_commit_time(repo_root: &Path, branch: &str) -> Option<(String, i64)> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cr|%ct", branch])
        .current_dir(repo_root)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?.trim().to_string();
    let (relative, timestamp_str) = stdout.split_once('|')?;
    let timestamp = timestamp_str.parse::<i64>().ok()?;
    Some((relative.to_string(), timestamp))
}

/// List all local and remote branches, deduplicated.
///
/// If a local branch tracks a remote (e.g. `feat/x` and `origin/feat/x`),
/// only the local name is kept. Remote-only branches have their `origin/` prefix stripped.
/// Returns a sorted list of unique branch names.
pub fn list_branches(repo_root: &Path) -> anyhow::Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "-a", "--format=%(refname:short)"])
        .current_dir(repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(WkspaceError::GitError(stderr.trim().to_string()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut local_branches = std::collections::HashSet::new();
    let mut remote_only = Vec::new();

    for line in stdout.lines() {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }
        if let Some(remote_name) = name.strip_prefix("origin/") {
            // Skip HEAD pointer
            if remote_name == "HEAD" {
                continue;
            }
            remote_only.push(remote_name.to_string());
        } else {
            local_branches.insert(name.to_string());
        }
    }

    // Merge: keep local names, add remote-only branches (not already local)
    for name in remote_only {
        local_branches.insert(name);
    }

    let mut branches: Vec<String> = local_branches.into_iter().collect();
    branches.sort();
    Ok(branches)
}

/// Check out an existing branch into a new worktree (no `-b` flag).
///
/// For remote-only branches, git automatically creates a local tracking branch.
pub fn checkout_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
) -> anyhow::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new("git")
        .args(["worktree", "add", &worktree_path.to_string_lossy(), branch])
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
            current_branch = branch_ref
                .strip_prefix("refs/heads/")
                .map(|s| s.to_string());
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
