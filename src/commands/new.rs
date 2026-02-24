use crate::context;
use crate::error::WkspaceError;
use crate::git;
use crate::ports;
use crate::scripts;
use std::collections::HashMap;
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

    // Allocate ports before worktree creation (fail early)
    let port_env = ports::allocate_ports(&ctx.config.ports)?;
    if !port_env.is_empty() {
        println!("Allocated ports:");
        for (var, port) in &port_env {
            println!("  {var}={port}");
        }
    }

    // Build script environment: ports + worktree metadata
    let mut script_env = port_env;
    script_env.insert("WORKTREE_NAME".to_string(), name.to_string());

    // Create worktree + branch
    println!(
        "Creating worktree '{name}' from '{}'...",
        ctx.config.worktree.base_branch
    );
    git::add_worktree(
        &ctx.repo_root,
        &worktree_path,
        name,
        &ctx.config.worktree.base_branch,
    )?;

    // Run setup scripts
    if !ctx.config.scripts.setup.is_empty() {
        println!("Running setup scripts...");
        scripts::run_scripts(&ctx.config.scripts.setup, &worktree_path, &script_env)?;
    }

    // Spawn subshell (skip in tests via env var)
    if env::var("WKSPACE_NO_SHELL").is_err() {
        spawn_shell(&worktree_path, &script_env)?;
    }

    Ok(())
}

fn spawn_shell(cwd: &std::path::Path, extra_env: &HashMap<String, String>) -> anyhow::Result<()> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    println!("Opening shell in {}...", cwd.display());
    let mut child = Command::new(&shell)
        .current_dir(cwd)
        .envs(extra_env)
        .spawn()?;
    child.wait()?;
    Ok(())
}
