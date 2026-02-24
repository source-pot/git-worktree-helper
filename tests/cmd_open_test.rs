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
fn open_fails_for_nonexistent_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["open", "nope"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Worktree"));
}

#[test]
fn open_succeeds_for_existing_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "existing"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Use WKSPACE_NO_SHELL to avoid actually spawning a shell in tests
    let output = wkspace_bin()
        .args(["open", "existing"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}
