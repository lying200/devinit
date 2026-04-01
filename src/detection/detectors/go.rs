use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

/// Detects Go projects from `go.mod`.
///
/// # Errors
///
/// Returns any I/O error produced while reading `go.mod`.
pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let go_mod = target_dir.join("go.mod");
    if !go_mod.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(go_mod)?;
    let version = parse_go_version(&content);
    let mut reasons = vec!["found go.mod".to_string()];
    if version.is_some() {
        reasons.push("found go version".to_string());
    }

    Ok(Some(LanguageCandidate {
        language: Language::Go {
            version,
            package: None,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}

fn parse_go_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(version) = trimmed.strip_prefix("go ") {
            let version = version.trim();
            if !version.is_empty() {
                return Some(normalize_go_version(version));
            }
        }
    }
    None
}

/// go.mod 中的版本可能只有 major.minor（如 `1.22`），
/// devenv 需要完整的 major.minor.patch 格式（如 `1.22.0`）。
fn normalize_go_version(version: &str) -> String {
    let dot_count = version.chars().filter(|&c| c == '.').count();
    if dot_count == 1 {
        format!("{version}.0")
    } else {
        version.to_string()
    }
}
