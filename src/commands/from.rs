use crate::commands::new;
use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::ports;
use crate::scripts;
use dialoguer::FuzzySelect;
use std::collections::HashSet;
use std::env;

/// Show an interactive picker to select a branch for worktree creation.
pub fn pick_branch() -> anyhow::Result<String> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;

    // Fetch latest from remote
    println!("Fetching from remote...");
    git::fetch_and_update_branch(&ctx.repo_root, &ctx.config.worktree.base_branch);

    // Get all branches and filter out those already attached to worktrees
    let all_branches = git::list_branches(&ctx.repo_root)?;
    let worktrees = git::list_worktrees(&ctx.repo_root)?;
    let attached: HashSet<String> = worktrees.iter().filter_map(|w| w.branch.clone()).collect();

    let available: Vec<String> = all_branches
        .into_iter()
        .filter(|b| !attached.contains(b))
        .collect();

    if available.is_empty() {
        anyhow::bail!("No branches available (all are already attached to worktrees)");
    }

    let selection = FuzzySelect::new()
        .with_prompt("Select branch")
        .items(&available)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(i) => Ok(available[i].clone()),
        None => std::process::exit(0),
    }
}

pub fn run(branch: &str) -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;

    if branch == ctx.config.worktree.base_branch {
        anyhow::bail!("Cannot create worktree from base branch — use `wkspace new <name>` instead");
    }

    // Check if the branch is already checked out in another worktree
    let worktrees = git::list_worktrees(&ctx.repo_root)?;
    let attached: HashSet<String> = worktrees.iter().filter_map(|w| w.branch.clone()).collect();
    if attached.contains(branch) {
        anyhow::bail!(WkspaceError::BranchAlreadyCheckedOut(branch.to_string()));
    }

    // Derive worktree directory name: replace / with - for path safety
    let worktree_name = branch.replace('/', "-");
    let worktree_path = ctx.worktree_path(&worktree_name);

    if worktree_path.exists() {
        anyhow::bail!(WkspaceError::WorktreeExists(worktree_name.clone()));
    }

    // Allocate ports before worktree creation (fail early)
    let port_env = ports::allocate_ports(&ctx.config.ports)?;
    if !port_env.is_empty() {
        println!("Allocated ports:");
        for (var, port) in &port_env {
            println!("  {var}={port}");
        }
    }

    // Build script environment
    let mut script_env = port_env;
    script_env.insert("WORKTREE_NAME".to_string(), worktree_name.clone());

    // Fetch latest from remote
    println!("Fetching from remote...");
    git::fetch_and_update_branch(&ctx.repo_root, &ctx.config.worktree.base_branch);

    // Check out existing branch into worktree
    println!("Creating worktree '{worktree_name}' from branch '{branch}'...");
    git::checkout_worktree(&ctx.repo_root, &worktree_path, branch)?;

    // Run setup scripts
    if !ctx.config.scripts.setup.is_empty() {
        println!("Running setup scripts...");
        scripts::run_scripts(&ctx.config.scripts.setup, &worktree_path, &script_env)?;
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        new::spawn_shell(&worktree_path, &script_env)?;
    }

    Ok(())
}
