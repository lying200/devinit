use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let package_json = target_dir.join("package.json");
    if !package_json.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(package_json)?;
    let package_manager = parse_package_manager(&content);

    let mut reasons = vec!["found package.json".to_string()];
    if package_manager.is_some() {
        reasons.push("found packageManager".to_string());
    }

    Ok(Some(LanguageCandidate {
        language: Language::JavaScript {
            package: None,
            package_manager,
            corepack_enable: None,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}

fn parse_package_manager(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("\"packageManager\":") {
            let value = value.trim().trim_end_matches(',').trim().trim_matches('"');
            let manager = value.split('@').next().unwrap_or("").trim();
            if matches!(manager, "npm" | "pnpm" | "yarn" | "bun") {
                return Some(manager.to_string());
            }
        }
    }
    None
}
