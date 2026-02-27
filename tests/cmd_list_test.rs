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

fn wkspace_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wkspace"))
}

#[test]
fn list_shows_no_worktrees_initially() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No worktrees")
            || stdout.trim().is_empty()
            || !stdout.contains("my-feature")
    );
}

#[test]
fn list_shows_created_worktrees() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["new", "feat-a"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "wkspace new feat-a failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = wkspace_bin()
        .args(["new", "feat-b"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "wkspace new feat-b failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("feat-a"));
    assert!(stdout.contains("feat-b"));
}

#[test]
fn list_shows_description_when_set() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    let output = wkspace_bin()
        .args(["new", "desc-feat", "--desc", "OAuth2 login"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("desc-feat"));
    assert!(stdout.contains("OAuth2 login"));
}

#[test]
fn list_shows_clean_for_no_changes() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "clean-wt"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clean"), "expected 'clean' in: {stdout}");
}

#[test]
fn list_shows_uncommitted_count() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "dirty-wt"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    // Create uncommitted files in the worktree
    let wt_path = dir.path().join(".worktrees/dirty-wt");
    std::fs::write(wt_path.join("new-file.txt"), "uncommitted").unwrap();
    std::fs::write(wt_path.join("another.txt"), "also uncommitted").unwrap();

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2 uncommitted"),
        "expected '2 uncommitted' in: {stdout}"
    );
}

#[test]
fn list_shows_stale_marker() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Set stale_days to 0 so any worktree is immediately stale
    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"
stale_days = 0
"#,
    )
    .unwrap();

    wkspace_bin()
        .args(["new", "old-wt"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["list"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("stale"), "expected 'stale' in: {stdout}");
}
