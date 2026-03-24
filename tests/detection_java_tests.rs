use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::java::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-java-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn java_detector_returns_none_without_java_signals() {
    let dir = unique_test_dir("no-signals");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn java_detector_detects_java_from_pom_xml() {
    let dir = unique_test_dir("maven");
    create_dir(&dir);
    fs::write(dir.join("pom.xml"), "<project/>\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Java {
                jdk_package: None,
                gradle_enable: None,
                maven_enable: Some(true),
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found pom.xml".to_string()],
        })
    );
}

#[test]
fn java_detector_detects_java_from_gradle_file() {
    let dir = unique_test_dir("gradle");
    create_dir(&dir);
    fs::write(dir.join("build.gradle"), "plugins {}\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Java {
                jdk_package: None,
                gradle_enable: Some(true),
                maven_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found build.gradle".to_string()],
        })
    );
}
