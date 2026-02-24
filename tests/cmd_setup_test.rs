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
fn setup_runs_scripts_in_worktree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Write config with a setup script that creates a marker file
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

    // Create worktree via `wkspace new`
    let output = wkspace_bin()
        .args(["new", "setup-test"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let worktree_dir = dir.path().join(".worktrees/setup-test");

    // Remove the marker file created by `new`
    std::fs::remove_file(worktree_dir.join("setup-ran")).unwrap();
    assert!(!worktree_dir.join("setup-ran").exists());

    // Run `wkspace setup` from inside the worktree
    let output = wkspace_bin()
        .arg("setup")
        .current_dir(&worktree_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(worktree_dir.join("setup-ran").exists());
}

#[test]
fn setup_fails_in_main_working_tree() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
"#,
    )
    .unwrap();

    let output = wkspace_bin()
        .arg("setup")
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not inside a git worktree"),
        "expected worktree error, got: {stderr}"
    );
}

#[test]
fn setup_fails_without_config() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Create worktree via raw git (no .wkspace.toml)
    let worktree_dir = dir.path().join(".worktrees/no-config");
    Command::new("git")
        .args([
            "worktree",
            "add",
            &worktree_dir.to_string_lossy(),
            "-b",
            "no-config",
        ])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let output = wkspace_bin()
        .arg("setup")
        .current_dir(&worktree_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(".wkspace.toml"),
        "expected config error, got: {stderr}"
    );
}

#[test]
fn setup_injects_port_env_vars() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = ["echo $MY_TEST_PORT > port.txt"]
teardown = []

[ports]
test_port = "MY_TEST_PORT"
"#,
    )
    .unwrap();

    let output = wkspace_bin()
        .args(["new", "port-setup"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let worktree_dir = dir.path().join(".worktrees/port-setup");

    // Remove port.txt from `new`, then re-run setup
    std::fs::remove_file(worktree_dir.join("port.txt")).unwrap();

    let output = wkspace_bin()
        .arg("setup")
        .current_dir(&worktree_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let contents = std::fs::read_to_string(worktree_dir.join("port.txt"))
        .unwrap()
        .trim()
        .to_string();
    let port: u16 = contents
        .parse()
        .expect("port.txt should contain a valid port number");
    assert!(
        (10000..=11000).contains(&port),
        "Port {port} should be in range 10000..=11000"
    );
}

#[test]
fn setup_injects_worktree_name() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    std::fs::write(
        dir.path().join(".wkspace.toml"),
        r#"
[worktree]
base_branch = "main"
directory = ".worktrees"

[scripts]
setup = ["echo $WORKTREE_NAME > name.txt"]
teardown = []
"#,
    )
    .unwrap();

    let output = wkspace_bin()
        .args(["new", "name-test"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let worktree_dir = dir.path().join(".worktrees/name-test");
    std::fs::remove_file(worktree_dir.join("name.txt")).unwrap();

    let output = wkspace_bin()
        .arg("setup")
        .current_dir(&worktree_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let name = std::fs::read_to_string(worktree_dir.join("name.txt"))
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(name, "name-test");
}
