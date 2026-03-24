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
fn go_detector_reads_version_from_go_mod_go_directive() {
    let dir = unique_test_dir("go-version");
    create_dir(&dir);
    fs::write(
        dir.join("go.mod"),
        "module example.com/demo\n\ngo 1.24.0\n",
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
