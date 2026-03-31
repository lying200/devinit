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

#[test]
fn python_detector_reads_version_from_requires_python() {
    let dir = unique_test_dir("requires-python");
    create_dir(&dir);
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nrequires-python = \">=3.11\"\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Python {
                version: Some("3.11".to_string()),
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found pyproject.toml".to_string(),
                "found requires-python".to_string(),
            ],
        })
    );
}

#[test]
fn python_detector_reads_version_from_requires_python_with_patch() {
    let dir = unique_test_dir("requires-python-patch");
    create_dir(&dir);
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nrequires-python = \">=3.11.0\"\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Python {
                version: Some("3.11.0".to_string()),
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found pyproject.toml".to_string(),
                "found requires-python".to_string(),
            ],
        })
    );
}

#[test]
fn python_detector_reads_version_from_requires_python_exact() {
    let dir = unique_test_dir("requires-python-exact");
    create_dir(&dir);
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nrequires-python = \"==3.12.1\"\n",
    )
    .unwrap();

    let result = detect(&dir).unwrap();

    assert_eq!(
        result,
        Some(LanguageCandidate {
            language: Language::Python {
                version: Some("3.12.1".to_string()),
                package: None,
                uv_enable: None,
                venv_enable: None,
                venv_quiet: None,
            },
            confidence: DetectionConfidence::High,
            reasons: vec![
                "found pyproject.toml".to_string(),
                "found requires-python".to_string(),
            ],
        })
    );
}

#[test]
fn python_detector_ignores_complex_requires_python_range() {
    let dir = unique_test_dir("requires-python-complex");
    create_dir(&dir);
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nrequires-python = \">=3.8,<4.0\"\n",
    )
    .unwrap();

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
fn python_detector_dot_python_version_takes_precedence() {
    let dir = unique_test_dir("precedence");
    create_dir(&dir);
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"demo\"\nrequires-python = \">=3.11\"\n",
    )
    .unwrap();
    fs::write(dir.join(".python-version"), "3.12.2\n").unwrap();

    let result = detect(&dir).unwrap();

    // .python-version 优先于 requires-python
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
