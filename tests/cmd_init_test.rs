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
fn init_creates_config_file() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".wkspace.toml").exists());

    let contents = std::fs::read_to_string(dir.path().join(".wkspace.toml")).unwrap();
    assert!(contents.contains("base_branch"));
}

#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Run init twice
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    let output = wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();

    assert!(output.status.success());
}

#[test]
fn init_fails_outside_git_repo() {
    let dir = TempDir::new().unwrap();

    let output = wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("git repository") || stderr.contains("Not inside"));
}

#[test]
fn init_adds_worktrees_dir_to_gitignore() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(gitignore.contains(".worktrees"));
}

#[test]
fn init_does_not_duplicate_gitignore_entry() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Run init twice
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();
    wkspace_bin().args(["init"]).current_dir(dir.path()).output().unwrap();

    let gitignore = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    let count = gitignore.matches(".worktrees").count();
    assert_eq!(count, 1);
}
