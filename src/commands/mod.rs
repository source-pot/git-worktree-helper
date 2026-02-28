use crate::context;
use crate::git;
use dialoguer::Select;
use std::env;

pub mod from;
pub mod init;
pub mod list;
pub mod new;
pub mod open;
pub mod rm;
pub mod setup;

/// Show an interactive picker to select a managed worktree.
pub fn pick_worktree(prompt: &str) -> anyhow::Result<String> {
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
        anyhow::bail!("No managed worktrees found");
    }

    let selection = Select::new()
        .with_prompt(prompt)
        .items(&names)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(i) => Ok(names[i].clone()),
        None => std::process::exit(0),
    }
}
