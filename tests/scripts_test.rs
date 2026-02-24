use std::collections::HashMap;
use tempfile::TempDir;
use wkspace::scripts;

#[test]
fn run_scripts_empty_list_succeeds() {
    let dir = TempDir::new().unwrap();
    let result = scripts::run_scripts(&[], dir.path(), &HashMap::new());
    assert!(result.is_ok());
}

#[test]
fn run_scripts_successful_commands() {
    let dir = TempDir::new().unwrap();
    let commands = vec!["echo hello".to_string(), "true".to_string()];
    let result = scripts::run_scripts(&commands, dir.path(), &HashMap::new());
    assert!(result.is_ok());
}

#[test]
fn run_scripts_stops_on_first_failure() {
    let dir = TempDir::new().unwrap();
    // "false" exits with code 1, "echo after" should never run
    let marker = dir.path().join("marker");
    let commands = vec![
        "false".to_string(),
        format!("touch {}", marker.display()),
    ];
    let result = scripts::run_scripts(&commands, dir.path(), &HashMap::new());
    assert!(result.is_err());
    assert!(!marker.exists(), "Second command should not have run");
}
