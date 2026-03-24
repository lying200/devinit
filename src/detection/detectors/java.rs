use std::{io, path::Path};

use crate::detection::{DetectionConfidence, LanguageCandidate};
use crate::schema::Language;

pub fn detect(target_dir: &Path) -> io::Result<Option<LanguageCandidate>> {
    let pom = target_dir.join("pom.xml");
    let gradle = target_dir.join("build.gradle");
    let gradle_kts = target_dir.join("build.gradle.kts");

    let maven_enable = pom.exists().then_some(true);
    let gradle_enable = (gradle.exists() || gradle_kts.exists()).then_some(true);

    if maven_enable.is_none() && gradle_enable.is_none() {
        return Ok(None);
    }

    let mut reasons = Vec::new();
    if pom.exists() {
        reasons.push("found pom.xml".to_string());
    }
    if gradle.exists() {
        reasons.push("found build.gradle".to_string());
    }
    if gradle_kts.exists() {
        reasons.push("found build.gradle.kts".to_string());
    }

    Ok(Some(LanguageCandidate {
        language: Language::Java {
            jdk_package: None,
            gradle_enable,
            maven_enable,
        },
        confidence: DetectionConfidence::High,
        reasons,
    }))
}
