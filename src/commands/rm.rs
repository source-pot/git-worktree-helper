use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::scripts;
use std::collections::HashMap;
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
        scripts::run_scripts(&ctx.config.scripts.teardown, &worktree_path, &HashMap::new())?;
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
