use std::{fs, io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let pyproject = target_dir.join("pyproject.toml");
    let requirements = target_dir.join("requirements.txt");

    let (confidence, mut reasons) = if pyproject.exists() {
        (DetectionConfidence::High, vec!["found pyproject.toml".to_string()])
    } else if requirements.exists() {
        (
            DetectionConfidence::Medium,
            vec!["found requirements.txt".to_string()],
        )
    } else {
        return Ok(None);
    };

    let python_version = target_dir.join(".python-version");
    let version = if python_version.exists() {
        reasons.push("found .python-version".to_string());
        let value = fs::read_to_string(python_version)?;
        let value = value.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    } else {
        None
    };

    Ok(Some(LanguageCandidate {
        language: Language::Python {
            version,
            package: None,
            uv_enable: None,
            venv_enable: None,
            venv_quiet: None,
        },
        confidence,
        reasons,
    }))
}
