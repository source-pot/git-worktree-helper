use crate::context;
use crate::git;
use crate::scripts;
use std::collections::HashMap;
use std::env;

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let name = git::current_worktree_name(&cwd)?;
    let ctx = context::resolve_strict(&cwd)?;

    let mut script_env = HashMap::new();
    script_env.insert("WORKTREE_NAME".to_string(), name.to_string());

    if !ctx.config.scripts.teardown.is_empty() {
        println!("Running teardown scripts in worktree '{name}'...");
        scripts::run_scripts(&ctx.config.scripts.teardown, &cwd, &script_env)?;
    }

    Ok(())
}
