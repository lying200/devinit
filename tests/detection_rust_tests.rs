use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::rust::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-rust-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn rust_detector_returns_none_without_cargo_toml() {
    let dir = unique_test_dir("no-cargo");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn rust_detector_detects_rust_from_cargo_toml() {
    let dir = unique_test_dir("cargo");
    create_dir(&dir);
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Rust {
                channel: None,
                version: None,
                components: None,
                targets: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found Cargo.toml".to_string()],
        })
    );
}

#[test]
fn rust_detector_reads_channel_from_rust_toolchain_toml() {
    let dir = unique_test_dir("toolchain");
    create_dir(&dir);
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    fs::write(
        dir.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"stable\"\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Rust {
                channel: Some("stable".to_string()),
                version: None,
                components: None,
                targets: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found Cargo.toml".to_string(),
                "found rust-toolchain.toml".to_string(),
            ],
        })
    );
}
