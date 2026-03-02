use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::scripts;
use std::collections::HashMap;
use std::env;

pub fn run(name: &str, force: bool) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktree_path = ctx.worktree_path(name);

    // Validate worktree exists
    if !worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeNotFound(name.to_string()));
    }

    // Warn if worktree has uncommitted changes
    let status = git::get_worktree_status(&worktree_path)?;
    if !force && status.uncommitted_count > 0 {
        println!("Worktree '{name}' has uncommitted changes:");
        for file in &status.files {
            println!("  {file}");
        }
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to remove this worktree?")
            .default(false)
            .interact()?;
        if !confirm {
            return Ok(());
        }
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

    // Remove the worktree directory and git metadata in one step
    println!("Removing worktree '{name}'...");
    git::remove_worktree(&ctx.repo_root, &worktree_path)?;

    // Delete the branch
    git::delete_branch(&ctx.repo_root, &branch)?;

    println!("Worktree '{name}' removed");
    Ok(())
}
