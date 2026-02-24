use std::process::Command;
use tempfile::TempDir;

fn init_repo(dir: &std::path::Path) {
    Command::new("git").args(["init", "-b", "main"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

#[test]
fn add_worktree_creates_directory_and_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    let wt_dir = dir.path().join(".worktrees").join("my-feature");
    let result = wkspace::git::add_worktree(dir.path(), &wt_dir, "my-feature", "main");
    assert!(result.is_ok());
    assert!(wt_dir.exists());
}

#[test]
fn add_worktree_fails_if_branch_exists() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    // Create the branch first
    Command::new("git").args(["branch", "existing"]).current_dir(dir.path()).output().unwrap();

    let wt_dir = dir.path().join(".worktrees").join("existing");
    let result = wkspace::git::add_worktree(dir.path(), &wt_dir, "existing", "main");
    assert!(result.is_err());
}

#[test]
fn branch_exists_returns_true_for_existing_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    assert!(wkspace::git::branch_exists(dir.path(), "main").unwrap());
}

#[test]
fn branch_exists_returns_false_for_missing_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    assert!(!wkspace::git::branch_exists(dir.path(), "nope").unwrap());
}

#[test]
fn delete_branch_removes_branch() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    Command::new("git").args(["branch", "to-delete"]).current_dir(dir.path()).output().unwrap();

    let result = wkspace::git::delete_branch(dir.path(), "to-delete");
    assert!(result.is_ok());
    assert!(!wkspace::git::branch_exists(dir.path(), "to-delete").unwrap());
}

#[test]
fn prune_worktrees_succeeds() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());
    let result = wkspace::git::prune_worktrees(dir.path());
    assert!(result.is_ok());
}

#[test]
fn list_worktrees_returns_entries() {
    let dir = TempDir::new().unwrap();
    init_repo(dir.path());

    let wt_dir = dir.path().join(".worktrees").join("feat-a");
    wkspace::git::add_worktree(dir.path(), &wt_dir, "feat-a", "main").unwrap();

    let entries = wkspace::git::list_worktrees(dir.path()).unwrap();
    // Should include the main worktree and our new one
    assert!(entries.len() >= 2);
    let feat = entries.iter().find(|e| e.branch.as_deref() == Some("feat-a"));
    assert!(feat.is_some());
}
