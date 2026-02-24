use std::process::Command;
use tempfile::TempDir;

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init"]).current_dir(dir).output().unwrap();
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
fn new_creates_worktree_and_branch() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // new will try to spawn a subshell — set WKSPACE_NO_SHELL to skip it
    let output = wkspace_bin()
        .args(["new", "my-feature"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".worktrees").join("my-feature").exists());

    // Verify branch was created
    let branches = Command::new("git")
        .args(["branch", "--list", "my-feature"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&branches.stdout).trim().is_empty());
}

#[test]
fn new_runs_setup_scripts() {
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

    let output = wkspace_bin()
        .args(["new", "with-setup"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".worktrees/with-setup/setup-ran").exists());
}

#[test]
fn new_fails_if_worktree_exists() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    wkspace_bin()
        .args(["new", "dupe"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    let output = wkspace_bin()
        .args(["new", "dupe"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists") || stderr.contains("open"));
}

#[test]
fn new_injects_port_env_vars_into_setup_scripts() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // Config with a port and a setup script that writes the env var to a file
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
        .args(["new", "port-test"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let port_file = dir.path().join(".worktrees/port-test/port.txt");
    assert!(port_file.exists(), "port.txt should have been created by setup script");

    let contents = std::fs::read_to_string(&port_file).unwrap().trim().to_string();
    let port: u16 = contents.parse().expect("port.txt should contain a valid port number");
    assert!(
        (10000..=11000).contains(&port),
        "Port {port} should be in range 10000..=11000"
    );
}

#[test]
fn new_auto_inits_config() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // No .wkspace.toml exists
    let output = wkspace_bin()
        .args(["new", "auto-init"])
        .current_dir(dir.path())
        .env("WKSPACE_NO_SHELL", "1")
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert!(dir.path().join(".wkspace.toml").exists());
    assert!(dir.path().join(".worktrees/auto-init").exists());
}
