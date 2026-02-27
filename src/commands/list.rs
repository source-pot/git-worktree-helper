use crate::context;
use crate::git;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

struct WorktreeRow {
    name: String,
    status: String,
    last_commit: String,
    description: String,
}

pub fn run() -> anyhow::Result<()> {
    let cwd = env::current_dir()?;
    let ctx = context::resolve(&cwd)?;
    let worktrees_dir = ctx.worktrees_dir();

    let entries = git::list_worktrees(&ctx.repo_root)?;

    // Filter to only managed worktrees (those under the worktrees directory)
    let managed: Vec<_> = entries
        .iter()
        .filter(|e| e.path.starts_with(&worktrees_dir))
        .collect();

    if managed.is_empty() {
        println!("No worktrees");
        return Ok(());
    }

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let stale_threshold = (ctx.config.worktree.stale_days * 86400) as i64;

    let mut rows = Vec::new();
    for entry in &managed {
        let name = entry
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let branch = entry.branch.as_deref().unwrap_or(&name);

        // Gather status
        let status = match git::get_worktree_status(&entry.path) {
            Ok(s) if s.uncommitted_count > 0 => format!("{} uncommitted", s.uncommitted_count),
            _ => "clean".to_string(),
        };

        // Gather last commit time and staleness
        let (last_commit, is_stale) = match git::get_last_commit_time(&ctx.repo_root, branch) {
            Some((relative, timestamp)) => {
                let stale = (now_secs - timestamp) >= stale_threshold;
                (relative, stale)
            }
            None => ("-".to_string(), false),
        };

        // Gather description, appending "stale" marker if needed
        let desc = git::get_branch_description(&ctx.repo_root, branch).unwrap_or_default();
        let description = if is_stale && desc.is_empty() {
            "stale".to_string()
        } else if is_stale {
            format!("{desc} (stale)")
        } else {
            desc
        };

        rows.push(WorktreeRow {
            name,
            status,
            last_commit,
            description,
        });
    }

    // Calculate column widths
    let name_w = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(4);
    let status_w = rows
        .iter()
        .map(|r| r.status.len())
        .max()
        .unwrap_or(0)
        .max(6);
    let commit_w = rows
        .iter()
        .map(|r| r.last_commit.len())
        .max()
        .unwrap_or(0)
        .max(11);

    // Print header
    println!(
        "{:<name_w$}  {:<status_w$}  {:<commit_w$}  DESCRIPTION",
        "NAME", "STATUS", "LAST COMMIT"
    );

    // Print rows
    for row in &rows {
        println!(
            "{:<name_w$}  {:<status_w$}  {:<commit_w$}  {}",
            row.name, row.status, row.last_commit, row.description
        );
    }

    Ok(())
}
