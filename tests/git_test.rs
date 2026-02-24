use std::process::Command;
use tempfile::TempDir;

#[test]
fn find_repo_root_in_git_repo() {
    let dir = TempDir::new().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let result = wkspace::git::find_repo_root(dir.path());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), dir.path().canonicalize().unwrap());
}

#[test]
fn find_repo_root_outside_git_repo() {
    let dir = TempDir::new().unwrap();
    let result = wkspace::git::find_repo_root(dir.path());
    assert!(result.is_err());
}
