use crate::context;
use crate::git;
use crate::ports;
use crate::scripts;
use std::env;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let name = git::current_worktree_name(&cwd)?;
    let ctx = context::resolve_strict(&cwd)?;

    // Allocate ports
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

    // Run setup scripts
    if !ctx.config.scripts.setup.is_empty() {
        println!("Running setup scripts in worktree '{name}'...");
        scripts::run_scripts(&ctx.config.scripts.setup, &cwd, &script_env)?;
    }

    Ok(())
}
