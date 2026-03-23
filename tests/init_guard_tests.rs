use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::init_guard::{detect_existing_environment, has_local_git_dir, target_dir_was_empty};

fn unique_test_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "devinit-init-guard-{}-{}-{}",
        name,
        std::process::id(),
        nanos
    ))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn detect_existing_environment_finds_devenv_nix() {
    let dir = unique_test_dir("devenv-nix");
    create_dir(&dir);
    fs::write(dir.join("devenv.nix"), "{}").unwrap();

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found.as_deref(), Some("devenv.nix"));
}

#[test]
fn detect_existing_environment_finds_envrc() {
    let dir = unique_test_dir("envrc");
    create_dir(&dir);
    fs::write(dir.join(".envrc"), "use nix").unwrap();

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found.as_deref(), Some(".envrc"));
}

#[test]
fn detect_existing_environment_finds_flake_nix() {
    let dir = unique_test_dir("flake");
    create_dir(&dir);
    fs::write(dir.join("flake.nix"), "{}").unwrap();

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found.as_deref(), Some("flake.nix"));
}

#[test]
fn detect_existing_environment_finds_dot_devenv_entries() {
    let dir = unique_test_dir("dot-devenv");
    create_dir(&dir);
    create_dir(&dir.join(".devenv"));

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found.as_deref(), Some(".devenv"));
}

#[test]
fn detect_existing_environment_finds_dot_direnv() {
    let dir = unique_test_dir("dot-direnv");
    create_dir(&dir);
    create_dir(&dir.join(".direnv"));

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found.as_deref(), Some(".direnv"));
}

#[test]
fn detect_existing_environment_returns_none_for_clean_directory() {
    let dir = unique_test_dir("clean");
    create_dir(&dir);
    fs::write(dir.join("README.md"), "# clean").unwrap();

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found, None);
}

#[test]
fn detect_existing_environment_returns_none_for_missing_directory() {
    let dir = unique_test_dir("missing");

    let found = detect_existing_environment(&dir).unwrap();

    assert_eq!(found, None);
}

#[test]
fn target_dir_was_empty_returns_true_for_missing_directory() {
    let dir = unique_test_dir("missing-empty");

    let is_empty = target_dir_was_empty(&dir, false).unwrap();

    assert!(is_empty);
}

#[test]
fn target_dir_was_empty_returns_true_for_existing_empty_directory() {
    let dir = unique_test_dir("existing-empty");
    create_dir(&dir);

    let is_empty = target_dir_was_empty(&dir, true).unwrap();

    assert!(is_empty);
}

#[test]
fn target_dir_was_empty_returns_false_for_non_empty_directory() {
    let dir = unique_test_dir("non-empty");
    create_dir(&dir);
    fs::write(dir.join("README.md"), "# devinit").unwrap();

    let is_empty = target_dir_was_empty(&dir, true).unwrap();

    assert!(!is_empty);
}

#[test]
fn has_local_git_dir_only_checks_target_directory() {
    let parent = unique_test_dir("git-parent");
    let child = parent.join("child");
    create_dir(&parent.join(".git"));
    create_dir(&child);

    assert!(has_local_git_dir(&parent));
    assert!(!has_local_git_dir(&child));
}

#[test]
fn devinit_skips_initialization_when_existing_environment_is_detected() {
    let dir = unique_test_dir("skip-binary");
    create_dir(&dir);
    fs::write(dir.join("flake.nix"), "{ }").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_devinit"))
        .arg("--lang")
        .arg("rust")
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("existing direnv/devenv/nix environment detected"));
    assert!(!dir.join("devenv.nix").exists());
    assert!(!dir.join("devenv.yaml").exists());
    assert!(!dir.join(".envrc").exists());
}

#[test]
fn devinit_creates_missing_target_dir_before_prompting() {
    let dir = unique_test_dir("create-missing");

    let output = Command::new(env!("CARGO_BIN_EXE_devinit"))
        .arg("--lang")
        .arg("rust")
        .arg(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(dir.exists());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("create init target err"));
}

#[test]
fn devinit_initializes_git_for_missing_directory() {
    let dir = unique_test_dir("git-init-missing");

    let output = Command::new(env!("CARGO_BIN_EXE_devinit"))
        .arg("--lang")
        .arg("rust")
        .arg(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(dir.join(".git").exists());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("git init"));
}

#[test]
fn devinit_initializes_git_for_existing_empty_directory() {
    let dir = unique_test_dir("git-init-empty");
    create_dir(&dir);

    let output = Command::new(env!("CARGO_BIN_EXE_devinit"))
        .arg("--lang")
        .arg("rust")
        .arg(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(dir.join(".git").exists());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("git init"));
}

#[test]
fn devinit_does_not_initialize_git_for_non_empty_directory() {
    let dir = unique_test_dir("git-init-non-empty");
    create_dir(&dir);
    fs::write(dir.join("README.md"), "# devinit").unwrap();

    let _output = Command::new(env!("CARGO_BIN_EXE_devinit"))
        .arg("--lang")
        .arg("rust")
        .arg(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert!(!dir.join(".git").exists());
}
