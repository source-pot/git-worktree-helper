use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::scripts;
use dialoguer::Select;
use std::collections::HashMap;
use std::env;

/// Show an interactive picker to select a managed worktree.
pub fn pick_worktree() -> anyhow::Result<String> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktrees_dir = ctx.worktrees_dir();

    let entries = git::list_worktrees(&ctx.repo_root)?;
    let names: Vec<String> = entries
        .iter()
        .filter(|e| e.path.starts_with(&worktrees_dir))
        .filter_map(|e| e.path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();

    if names.is_empty() {
        anyhow::bail!("No worktrees to remove");
    }

    let selection = Select::new()
        .with_prompt("Select worktree to remove")
        .items(&names)
        .default(0)
        .interact()?;

    Ok(names[selection].clone())
}

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
        let mut script_env = HashMap::new();
        script_env.insert("WORKTREE_NAME".to_string(), name.to_string());
        scripts::run_scripts(&ctx.config.scripts.teardown, &worktree_path, &script_env)?;
    }

    // Resolve the real branch name from worktree metadata before removal.
    // For worktrees created via `wkspace from`, the directory name may differ
    // from the branch name (e.g. "feat-login" dir for "feat/login" branch).
    let branch = git::list_worktrees(&ctx.repo_root)?
        .into_iter()
        .find(|e| e.path == worktree_path)
        .and_then(|e| e.branch)
        .unwrap_or_else(|| name.to_string());

    // Force-remove the worktree directory
    println!("Removing worktree '{name}'...");
    std::fs::remove_dir_all(&worktree_path)?;

    // Prune stale worktree references
    git::prune_worktrees(&ctx.repo_root)?;

    // Delete the branch
    git::delete_branch(&ctx.repo_root, &branch)?;

    println!("Worktree '{name}' removed");
    Ok(())
}
