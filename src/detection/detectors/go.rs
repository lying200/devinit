use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

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
            let version = version.trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }
    None
}
