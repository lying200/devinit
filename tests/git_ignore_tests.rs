use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::{
    git_ignore::{IgnoreMode, apply_ignore_mode, find_git_repo_root},
    prompt::ignore_mode_from_selection,
};

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-{name}-{pid}-{nanos}"))
}

fn init_git_repo(path: &Path) {
    fs::create_dir_all(path).unwrap();

    let status = Command::new("git")
        .arg("init")
        .current_dir(path)
        .status()
        .unwrap();
    assert!(status.success(), "git init should succeed");
}

fn git(path: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} should succeed");
}

#[test]
fn apply_gitignore_rules_creates_gitignore_in_target_dir() {
    let repo_dir = unique_test_dir("gitignore-create");
    init_git_repo(&repo_dir);

    let outcome = apply_ignore_mode(&repo_dir, IgnoreMode::GitIgnore).unwrap();

    assert!(!outcome.skipped_for_missing_git);
    let gitignore_path = repo_dir.join(".gitignore");
    assert_eq!(
        outcome.destination.as_deref(),
        Some(gitignore_path.as_path())
    );

    let content = fs::read_to_string(gitignore_path).unwrap();
    assert!(content.contains(".devenv*"));
    assert!(content.contains("devenv.local.nix"));
    assert!(content.contains("devenv.local.yaml"));
    assert!(content.contains(".direnv"));
    assert!(!content.contains("devenv.nix"));
    assert!(!content.contains(".envrc"));
}

#[test]
fn apply_gitignore_rules_does_not_duplicate_existing_entries() {
    let repo_dir = unique_test_dir("gitignore-dedup");
    init_git_repo(&repo_dir);

    let gitignore_path = repo_dir.join(".gitignore");
    fs::write(
        &gitignore_path,
        "# devinit: devenv ignores\n.devenv*\ndevenv.local.nix\ndevenv.local.yaml\n.direnv\n",
    )
    .unwrap();

    let outcome = apply_ignore_mode(&repo_dir, IgnoreMode::GitIgnore).unwrap();

    assert!(!outcome.wrote_rules);
    let content = fs::read_to_string(gitignore_path).unwrap();
    assert_eq!(content.matches(".devenv*").count(), 1);
    assert_eq!(content.matches("devenv.local.nix").count(), 1);
    assert_eq!(content.matches("devenv.local.yaml").count(), 1);
    assert_eq!(content.matches(".direnv").count(), 1);
}

#[test]
fn apply_gitignore_rules_appends_only_missing_entries() {
    let repo_dir = unique_test_dir("gitignore-partial");
    init_git_repo(&repo_dir);

    let gitignore_path = repo_dir.join(".gitignore");
    fs::write(&gitignore_path, "node_modules\n.devenv*\n").unwrap();

    let outcome = apply_ignore_mode(&repo_dir, IgnoreMode::GitIgnore).unwrap();

    assert!(outcome.wrote_rules);
    let content = fs::read_to_string(gitignore_path).unwrap();
    // existing entries preserved, not duplicated
    assert_eq!(content.matches(".devenv*").count(), 1);
    assert!(content.contains("node_modules"));
    // missing entries added
    assert!(content.contains("devenv.local.nix"));
    assert!(content.contains("devenv.local.yaml"));
    assert!(content.contains(".direnv"));
}

#[test]
fn apply_local_exclude_rules_writes_repo_root_exclude() {
    let repo_dir = unique_test_dir("local-exclude");
    init_git_repo(&repo_dir);

    let outcome = apply_ignore_mode(&repo_dir, IgnoreMode::LocalExclude).unwrap();

    assert!(!outcome.skipped_for_missing_git);
    let exclude_path = repo_dir.join(".git/info/exclude");
    assert_eq!(outcome.destination.as_deref(), Some(exclude_path.as_path()));

    let content = fs::read_to_string(exclude_path).unwrap();
    assert!(content.contains(".devenv*"));
    assert!(content.contains("devenv.local.nix"));
    assert!(content.contains("devenv.local.yaml"));
    assert!(content.contains(".direnv"));
    assert!(content.contains("devenv.nix"));
    assert!(content.contains("devenv.yaml"));
    assert!(content.contains("devenv.lock"));
    assert!(content.contains(".envrc"));
}

#[test]
fn skip_ignore_handling_when_git_is_not_initialized() {
    let target_dir = unique_test_dir("skip-no-git");
    fs::create_dir_all(&target_dir).unwrap();

    let outcome = apply_ignore_mode(&target_dir, IgnoreMode::GitIgnore).unwrap();

    assert!(outcome.skipped_for_missing_git);
    assert!(outcome.destination.is_none());
    assert!(!target_dir.join(".gitignore").exists());
}

#[test]
fn tracked_files_are_reported_without_removing_them() {
    let repo_dir = unique_test_dir("tracked-files");
    init_git_repo(&repo_dir);
    fs::write(repo_dir.join("devenv.nix"), "test").unwrap();
    fs::write(repo_dir.join(".envrc"), "test").unwrap();
    git(&repo_dir, &["add", "devenv.nix", ".envrc"]);

    let outcome = apply_ignore_mode(&repo_dir, IgnoreMode::LocalExclude).unwrap();

    assert!(
        outcome
            .tracked_files
            .iter()
            .any(|path| path == "devenv.nix")
    );
    assert!(outcome.tracked_files.iter().any(|path| path == ".envrc"));

    let status = Command::new("git")
        .args(["status", "--short"])
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    assert!(status.status.success());
    let stdout = String::from_utf8(status.stdout).unwrap();
    assert!(stdout.contains("A  devenv.nix"));
    assert!(stdout.contains("A  .envrc"));
}

#[test]
fn find_git_repo_root_finds_parent_repository_for_nested_target() {
    let repo_dir = unique_test_dir("find-root");
    let nested_dir = repo_dir.join("deep/nested/path");
    init_git_repo(&repo_dir);
    fs::create_dir_all(&nested_dir).unwrap();

    let nested_root = find_git_repo_root(&nested_dir);
    let repo_root = find_git_repo_root(&repo_dir);

    assert_eq!(nested_root.as_deref(), Some(repo_dir.as_path()));
    assert_eq!(repo_root.as_deref(), Some(repo_dir.as_path()));
}

#[test]
fn local_exclude_uses_parent_repository_for_nested_target() {
    let repo_dir = unique_test_dir("local-exclude-skip");
    let nested_dir = repo_dir.join("nested/project");
    init_git_repo(&repo_dir);
    fs::create_dir_all(&nested_dir).unwrap();

    let outcome = apply_ignore_mode(&nested_dir, IgnoreMode::LocalExclude).unwrap();

    assert!(!outcome.skipped_for_missing_git);
    assert_eq!(
        outcome.destination.as_deref(),
        Some(repo_dir.join(".git/info/exclude").as_path())
    );
}

#[test]
fn ignore_mode_from_selection_maps_all_three_choices() {
    assert_eq!(ignore_mode_from_selection(0), IgnoreMode::None);
    assert_eq!(ignore_mode_from_selection(1), IgnoreMode::GitIgnore);
    assert_eq!(ignore_mode_from_selection(2), IgnoreMode::LocalExclude);
}
