use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::javascript::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-javascript-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn javascript_detector_returns_none_without_package_json() {
    let dir = unique_test_dir("no-package-json");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn javascript_detector_detects_javascript_from_package_json() {
    let dir = unique_test_dir("package-json");
    create_dir(&dir);
    fs::write(dir.join("package.json"), "{\n  \"name\": \"demo\"\n}\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: None,
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found package.json".to_string()],
        })
    );
}

#[test]
fn javascript_detector_reads_package_manager_from_package_json() {
    let dir = unique_test_dir("package-manager");
    create_dir(&dir);
    fs::write(
        dir.join("package.json"),
        "{\n  \"name\": \"demo\",\n  \"packageManager\": \"pnpm@9.0.0\"\n}\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::JavaScript {
                package: None,
                package_manager: Some("pnpm".to_string()),
                corepack_enable: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found package.json".to_string(),
                "found packageManager".to_string(),
            ],
        })
    );
}
