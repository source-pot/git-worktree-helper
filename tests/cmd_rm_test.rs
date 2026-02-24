use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init", "-b", "main"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn rm_removes_worktree_and_branch() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Create a worktree first
    wkspace_bin()
        .args(["new", "to-remove"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(dir.path().join(".worktrees/to-remove").exists());

    // Remove it
    let output = wkspace_bin()
        .args(["rm", "to-remove"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!dir.path().join(".worktrees/to-remove").exists());

    // Branch should be gone
    let branches = Command::new("git")
        .args(["branch", "--list", "to-remove"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&branches.stdout).trim().is_empty());
}

#[test]
fn rm_handles_untracked_files() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "dirty"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Add an untracked file (like .env)
    std::fs::write(dir.path().join(".worktrees/dirty/.env"), "SECRET=123").unwrap();

    let output = wkspace_bin()
        .args(["rm", "dirty"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!dir.path().join(".worktrees/dirty").exists());
}

#[test]
fn rm_runs_teardown_scripts() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Config with teardown that creates a marker in repo root
    let marker = dir.path().join("teardown-ran");
    std::fs::write(
        dir.path().join(".wkspace.toml"),
        format!(
            r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = []
teardown = ["touch {}"]
"#,
            marker.display()
        ),
    )
    .unwrap();

    wkspace_bin()
        .args(["new", "with-teardown"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    wkspace_bin()
        .args(["rm", "with-teardown"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(marker.exists());
}

#[test]
fn rm_fails_for_nonexistent_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["rm", "nope"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
}

#[test]
fn rm_stops_on_teardown_failure() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = []
teardown = ["false"]
"#,
    )
    .unwrap();

    wkspace_bin()
        .args(["new", "fail-teardown"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["rm", "fail-teardown"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    // Worktree should still exist since teardown failed
    assert!(dir.path().join(".worktrees/fail-teardown").exists());
}
