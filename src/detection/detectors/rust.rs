use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

/// Detects Rust projects from Cargo and toolchain files.
///
/// # Errors
///
/// Returns any I/O error produced while reading Rust toolchain files.
pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let cargo_toml = target_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(None);
    }

    let mut reasons = vec!["found Cargo.toml".to_string()];
    let (channel, version) = detect_toolchain(target_dir, &mut reasons)?;

    Ok(Some(LanguageCandidate {
        language: Language::Rust {
            channel,
            version,
            components: None,
            targets: None,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}

fn detect_toolchain(
    target_dir: &Path,
    reasons: &mut Vec<String>,
) -> io::Result<(Option<String>, Option<String>)> {
    let toolchain_toml = target_dir.join("rust-toolchain.toml");
    if toolchain_toml.exists() {
        reasons.push("found rust-toolchain.toml".to_string());
        let content = fs::read_to_string(toolchain_toml)?;
        if let Some(value) = parse_channel_assignment(&content) {
            if is_channel_name(&value) {
                return Ok((Some(value), None));
            }
            return Ok((None, Some(value)));
        }
    }

    let toolchain = target_dir.join("rust-toolchain");
    if toolchain.exists() {
        reasons.push("found rust-toolchain".to_string());
        let content = fs::read_to_string(toolchain)?;
        let value = content.trim().to_string();
        if !value.is_empty() {
            if is_channel_name(&value) {
                return Ok((Some(value), None));
            }
            return Ok((None, Some(value)));
        }
    }

    Ok((None, None))
}

fn parse_channel_assignment(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("channel =") {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn is_channel_name(value: &str) -> bool {
    matches!(value, "stable" | "beta" | "nightly")
}
