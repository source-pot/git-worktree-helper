use std::process::Command;
use tempfile::TempDir;
use wkspace::config::Config;

#[test]
fn parse_full_config() {
    let toml_str = r#"
[worktree]
base_branch = "develop"
directory = ".trees"

[scripts]
setup = ["npm install", "cp .env.example .env"]
teardown = ["echo cleanup"]
"#;
    let config = Config::parse(toml_str).unwrap();
    assert_eq!(config.worktree.base_branch, "develop");
    assert_eq!(config.worktree.directory, ".trees");
    assert_eq!(config.scripts.setup, vec!["npm install", "cp .env.example .env"]);
    assert_eq!(config.scripts.teardown, vec!["echo cleanup"]);
}

#[test]
fn default_config_has_sensible_values() {
    let config = Config::default();
    assert_eq!(config.worktree.base_branch, "main");
    assert_eq!(config.worktree.directory, ".worktrees");
    assert!(config.scripts.setup.is_empty());
    assert!(config.scripts.teardown.is_empty());
}

#[test]
fn default_template_is_valid_toml() {
    let template = Config::default_template();
    assert!(template.contains("base_branch"));
    assert!(template.contains(".worktrees"));
    // Verify it parses (strip comments first isn't needed — TOML supports comments)
    let config = Config::parse(&template).unwrap();
    assert_eq!(config.worktree.base_branch, "main");
}

fn init_git_repo(dir: &std::path::Path) {
    Command::new("git").args(["init", "-b", "main"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(dir).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(dir).output().unwrap();
    std::fs::write(dir.join("README.md"), "# test").unwrap();
    Command::new("git").args(["add", "."]).current_dir(dir).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(dir).output().unwrap();
}

#[test]
fn resolve_context_auto_creates_config() {
    let dir = TempDir::new().unwrap();
    init_git_repo(dir.path());

    // No .wkspace.toml exists yet
    assert!(!dir.path().join(".wkspace.toml").exists());

    let ctx = wkspace::context::resolve(dir.path()).unwrap();
    assert_eq!(ctx.config.worktree.base_branch, "main");

    // Config file should now exist
    assert!(dir.path().join(".wkspace.toml").exists());
}
