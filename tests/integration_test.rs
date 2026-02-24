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

/// Full lifecycle: init → new → list → rm → list
#[test]
fn full_lifecycle() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // init
    let out = wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(dir.path().join(".wkspace.toml").exists());
    assert!(dir.path().join(".gitignore").exists());

    // new
    let out = wkspace_bin()
        .args(["new", "feat-x"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(dir.path().join(".worktrees/feat-x").exists());

    // list shows feat-x
    let out = wkspace_bin().args(["list"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("feat-x"));

    // rm
    let out = wkspace_bin().args(["rm", "feat-x"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(!dir.path().join(".worktrees/feat-x").exists());

    // list shows no worktrees
    let out = wkspace_bin().args(["list"]).current_dir(dir.path()).output().unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stdout).contains("feat-x"));
}
