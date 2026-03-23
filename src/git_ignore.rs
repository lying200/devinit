use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreMode {
    None,
    GitIgnore,
    LocalExclude,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct IgnoreOutcome {
    pub skipped_for_missing_git: bool,
    pub destination: Option<PathBuf>,
    pub wrote_rules: bool,
    pub tracked_files: Vec<String>,
}

const SHARED_IGNORE_LINES: &[&str] = &[
    "# devinit: devenv ignores",
    ".devenv*",
    "devenv.local.nix",
    "devenv.local.yaml",
    ".direnv",
];

const LOCAL_EXCLUDE_LINES: &[&str] = &[
    "# devinit: local devenv mechanism ignores",
    ".devenv*",
    "devenv.local.nix",
    "devenv.local.yaml",
    ".direnv",
    "devenv.nix",
    "devenv.yaml",
    "devenv.lock",
    ".envrc",
];

pub fn find_git_repo_root(target_dir: &Path) -> Option<PathBuf> {
    if target_dir.join(".git").exists() {
        Some(target_dir.to_path_buf())
    } else {
        None
    }
}

pub fn apply_ignore_mode(target_dir: &Path, mode: IgnoreMode) -> io::Result<IgnoreOutcome> {
    if mode == IgnoreMode::None {
        return Ok(IgnoreOutcome::default());
    }

    let Some(repo_root) = find_git_repo_root(target_dir) else {
        return Ok(IgnoreOutcome {
            skipped_for_missing_git: true,
            ..IgnoreOutcome::default()
        });
    };

    let destination = match mode {
        IgnoreMode::None => None,
        IgnoreMode::GitIgnore => Some(target_dir.join(".gitignore")),
        IgnoreMode::LocalExclude => Some(repo_root.join(".git/info/exclude")),
    };

    let tracked_files = tracked_ignore_files(&repo_root, target_dir)?;
    let wrote_rules = if let Some(path) = destination.as_ref() {
        write_missing_ignore_lines(path, ignore_lines_for_mode(mode))?
    } else {
        false
    };

    Ok(IgnoreOutcome {
        skipped_for_missing_git: false,
        destination,
        wrote_rules,
        tracked_files,
    })
}

fn ignore_lines_for_mode(mode: IgnoreMode) -> &'static [&'static str] {
    match mode {
        IgnoreMode::None => &[],
        IgnoreMode::GitIgnore => SHARED_IGNORE_LINES,
        IgnoreMode::LocalExclude => LOCAL_EXCLUDE_LINES,
    }
}

fn write_missing_ignore_lines(path: &Path, lines: &[&str]) -> io::Result<bool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };

    let existing_lines: HashSet<&str> = existing.lines().collect();
    let missing_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !existing_lines.contains(line))
        .collect();

    if missing_lines.is_empty() {
        return Ok(false);
    }

    let mut updated = existing;
    if !updated.is_empty() {
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push('\n');
    }
    updated.push_str(&missing_lines.join("\n"));
    updated.push('\n');

    std::fs::write(path, updated)?;
    Ok(true)
}

fn tracked_ignore_files(repo_root: &Path, target_dir: &Path) -> io::Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "-z"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git ls-files failed with status {}",
            output.status
        )));
    }

    let mut tracked_files = Vec::new();
    for entry in output.stdout.split(|byte| *byte == 0) {
        if entry.is_empty() {
            continue;
        }

        let repo_relative = PathBuf::from(String::from_utf8_lossy(entry).into_owned());
        let absolute = repo_root.join(&repo_relative);
        let Ok(target_relative) = absolute.strip_prefix(target_dir) else {
            continue;
        };

        if is_relevant_ignore_path(target_relative) {
            tracked_files.push(target_relative.to_string_lossy().into_owned());
        }
    }

    tracked_files.sort();
    Ok(tracked_files)
}

fn is_relevant_ignore_path(path: &Path) -> bool {
    let Some(first) = path.iter().next() else {
        return false;
    };

    let first = first.to_string_lossy();
    if first.starts_with(".devenv") || first == ".direnv" {
        return true;
    }

    matches!(
        path.to_string_lossy().as_ref(),
        "devenv.local.nix"
            | "devenv.local.yaml"
            | "devenv.nix"
            | "devenv.yaml"
            | "devenv.lock"
            | ".envrc"
    )
}
