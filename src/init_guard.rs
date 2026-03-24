use std::{fs, io, path::Path};

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
