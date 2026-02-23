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
