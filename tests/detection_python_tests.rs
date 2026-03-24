use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use devinit::detection::{DetectionConfidence, LanguageCandidate};
use devinit::detection::detectors::python::detect;
use devinit::schema::Language;

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-python-detect-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn python_detector_returns_none_without_python_signals() {
    let dir = unique_test_dir("no-signals");
    create_dir(&dir);

    let result = detect(&dir).unwrap();

    assert_eq!(result, None);
}

#[test]
fn python_detector_detects_python_from_pyproject_toml() {
    let dir = unique_test_dir("pyproject");
    create_dir(&dir);
    fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Python {
                version: None,
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec!["found pyproject.toml".to_string()],
        })
    );
}

#[test]
fn python_detector_reads_version_from_dot_python_version() {
    let dir = unique_test_dir("python-version");
    create_dir(&dir);
    fs::write(dir.join("pyproject.toml"), "[project]\nname = \"demo\"\n").unwrap();
    fs::write(dir.join(".python-version"), "3.12.2\n").unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Python {
                version: Some("3.12.2".to_string()),
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found pyproject.toml".to_string(),
                "found .python-version".to_string(),
            ],
        })
    );
}
