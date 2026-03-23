use std::{fs, io, path::Path, process::Command};

const DIRECT_MARKERS: &[&str] = &[
    "devenv.nix",
    "devenv.yaml",
    "devenv.lock",
    ".envrc",
    "shell.nix",
    "default.nix",
    "flake.nix",
    "flake.lock",
    ".direnv",
];

pub fn detect_existing_environment(target_dir: &Path) -> io::Result<Option<String>> {
    if !target_dir.exists() {
        return Ok(None);
    }

    for marker in DIRECT_MARKERS {
        if target_dir.join(marker).exists() {
            return Ok(Some((*marker).to_string()));
        }
    }

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name.starts_with(".devenv") {
            return Ok(Some(file_name.into_owned()));
        }
    }

    Ok(None)
}

pub fn has_local_git_dir(target_dir: &Path) -> bool {
    target_dir.join(".git").exists()
}

pub fn target_dir_was_empty(target_dir: &Path, existed_before: bool) -> io::Result<bool> {
    if !existed_before {
        return Ok(true);
    }

    Ok(fs::read_dir(target_dir)?.next().is_none())
}

pub fn initialize_git_repository(target_dir: &Path) -> io::Result<()> {
    let output = Command::new("git").arg("init").arg(target_dir).output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("git init failed with status {}", output.status)
        } else {
            stderr
        };
        Err(io::Error::other(message))
    }
}
