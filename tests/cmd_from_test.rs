use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir)
        .output()
        .unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn create_branch(dir: &std::path::Path, name: &str) {
    Command::new("git")
        .args(["branch", name])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn from_creates_worktree_from_existing_branch() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "feat-login");

    let output = wkspace_bin()
        .args(["from", "feat-login"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join(".worktrees/feat-login").exists());

    // Verify the worktree is on the correct branch
    let head = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir.path().join(".worktrees/feat-login"))
        .output()
        .unwrap();
    let branch = String::from_utf8_lossy(&head.stdout).trim().to_string();
    assert_eq!(branch, "feat-login");
}

#[test]
fn from_sanitizes_slash_in_branch_name() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "feat/login");

    let output = wkspace_bin()
        .args(["from", "feat/login"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Directory name should have / replaced with -
    assert!(dir.path().join(".worktrees/feat-login").exists());

    // But the branch inside should still be feat/login
    let head = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir.path().join(".worktrees/feat-login"))
        .output()
        .unwrap();
    let branch = String::from_utf8_lossy(&head.stdout).trim().to_string();
    assert_eq!(branch, "feat/login");
}

#[test]
fn from_base_branch_errors() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["from", "main"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot create worktree from base branch"));
}

#[test]
fn from_runs_setup_scripts() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "scripted");

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = ["touch setup-ran"]
teardown = []
"#,
    )
    .unwrap();

    let output = wkspace_bin()
        .args(["from", "scripted"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join(".worktrees/scripted/setup-ran").exists());
}

#[test]
fn rm_works_for_branches_with_slash() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "feat/login");

    // Create worktree from branch with /
    let output = wkspace_bin()
        .args(["from", "feat/login"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "from stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(dir.path().join(".worktrees/feat-login").exists());

    // Remove using the sanitized directory name
    let output = wkspace_bin()
        .args(["rm", "feat-login"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "rm stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.path().join(".worktrees/feat-login").exists());

    // Verify the real branch feat/login was deleted
    let branch_check = Command::new("git")
        .args(["branch", "--list", "feat/login"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&branch_check.stdout);
    assert!(
        branches.trim().is_empty(),
        "branch feat/login should be deleted, got: {branches}"
    );
}

#[test]
fn from_fails_if_worktree_already_exists() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "dupe-branch");

    // Create worktree first time
    wkspace_bin()
        .args(["from", "dupe-branch"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Second time should fail
    let output = wkspace_bin()
        .args(["from", "dupe-branch"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already checked out"),
        "expected 'already checked out' error, got: {stderr}"
    );
}

#[test]
fn from_fails_if_branch_already_checked_out() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());
    create_branch(dir.path(), "shared-branch");

    // Create worktree from the branch
    let output = wkspace_bin()
        .args(["from", "shared-branch"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "first from stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Remove the worktree directory but leave the branch checked out
    // (simulate the branch being attached to a worktree elsewhere)
    // Actually, the worktree still exists with the branch attached,
    // so trying `from` again with the same branch should fail with
    // "already checked out" even if the directory name would differ.
    let output = wkspace_bin()
        .args(["from", "shared-branch"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already checked out"),
        "expected 'already checked out' error, got: {stderr}"
    );
}
