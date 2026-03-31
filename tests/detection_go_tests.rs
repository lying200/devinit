use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::go::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-go-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn go_detector_returns_none_without_go_mod() {
    let dir = unique_test_dir("no-go-mod");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn go_detector_detects_go_from_go_mod() {
    let dir = unique_test_dir("go-mod");
    create_dir(&dir);
    fs::write(dir.join("go.mod"), "module example.com/demo\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Go {
                version: None,
                package: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found go.mod".to_string()],
        })
    );
}

#[test]
fn go_detector_normalizes_two_segment_version_to_three() {
    let dir = unique_test_dir("go-version-two-seg");
    create_dir(&dir);
    fs::write(
        dir.join("go.mod"),
        "module example.com/demo\n\ngo 1.24\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Go {
                version: Some("1.24.0".to_string()),
                package: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found go.mod".to_string(), "found go version".to_string()],
        })
    );
}

#[test]
fn go_detector_empty_go_mod_has_no_version() {
    let dir = unique_test_dir("go-empty");
    create_dir(&dir);
    fs::write(dir.join("go.mod"), "").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Go {
                version: None,
                package: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found go.mod".to_string()],
        })
    );
}

#[test]
fn go_detector_keeps_three_segment_version_as_is() {
    let dir = unique_test_dir("go-version-three-seg");
    create_dir(&dir);
    fs::write(
        dir.join("go.mod"),
        "module example.com/demo\n\ngo 1.22.5\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Go {
                version: Some("1.22.5".to_string()),
                package: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found go.mod".to_string(), "found go version".to_string()],
        })
    );
}
