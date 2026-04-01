use devinit::detection::{
    DetectionConfidence, DetectionOutcome, LanguageCandidate, detect_project,
};
use devinit::schema::Language;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("devinit-detection-engine-{name}-{pid}-{nanos}"))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn detection_types_support_multiple_language_matches() {
    let candidate = LanguageCandidate {
        language: Language::Rust {
            channel: None,
            version: Some("1.76.0".to_string()),
            components: None,
            targets: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found Cargo.toml".to_string()],
    };

    let outcome = DetectionOutcome::Matches {
        candidates: vec![candidate],
    };

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert_eq!(candidates.len(), 1);
            assert_eq!(candidates[0].confidence, DetectionConfidence::High);
            assert_eq!(candidates[0].reasons, vec!["found Cargo.toml"]);
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_returns_no_match_for_empty_dir() {
    let dir = unique_test_dir("empty");
    create_dir(&dir);

    let outcome = detect_project(&dir).unwrap();

    assert_eq!(outcome, DetectionOutcome::NoMatch);
}

#[test]
fn detect_project_returns_rust_match_for_cargo_project() {
    let dir = unique_test_dir("cargo-project");
    create_dir(&dir);
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert!(!candidates.is_empty());
            assert!(matches!(candidates[0].language, Language::Rust { .. }));
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_returns_multiple_languages_for_mixed_project() {
    let dir = unique_test_dir("mixed-go-js");
    create_dir(&dir);
    fs::write(dir.join("go.mod"), "module example.com/demo\n\ngo 1.22\n").unwrap();
    fs::write(dir.join("package.json"), r#"{"name": "frontend"}"#).unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert_eq!(candidates.len(), 2);
            let names: Vec<&str> = candidates
                .iter()
                .map(|c| match &c.language {
                    Language::Go { .. } => "go",
                    Language::JavaScript { .. } => "javascript",
                    _ => "other",
                })
                .collect();
            assert!(names.contains(&"go"));
            assert!(names.contains(&"javascript"));
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_sorts_candidates_by_priority() {
    let dir = unique_test_dir("priority-rust-js");
    create_dir(&dir);
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
    fs::write(dir.join("package.json"), r#"{"name": "frontend"}"#).unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert!(candidates.len() >= 2);
            // Rust should come first (higher priority)
            assert!(matches!(candidates[0].language, Language::Rust { .. }));
            assert!(matches!(candidates[1].language, Language::JavaScript { .. }));
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_finds_languages_in_subdirectories() {
    let dir = unique_test_dir("monorepo-subdirs");
    create_dir(&dir);
    create_dir(&dir.join("backend"));
    create_dir(&dir.join("tool"));
    create_dir(&dir.join("ui"));
    fs::write(dir.join("backend/pom.xml"), "<project></project>").unwrap();
    fs::write(dir.join("tool/pyproject.toml"), "[project]\nname = \"tool\"\n").unwrap();
    fs::write(dir.join("ui/package.json"), r#"{"name": "ui"}"#).unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert_eq!(candidates.len(), 3);
            assert!(candidates.iter().any(|c| matches!(c.language, Language::Java { .. })));
            assert!(candidates.iter().any(|c| matches!(c.language, Language::Python { .. })));
            assert!(candidates.iter().any(|c| matches!(c.language, Language::JavaScript { .. })));
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_mixed_root_and_subdir() {
    let dir = unique_test_dir("mixed-root-subdir");
    create_dir(&dir);
    create_dir(&dir.join("frontend"));
    fs::write(dir.join("pyproject.toml"), "[project]\nname = \"app\"\n").unwrap();
    fs::write(dir.join("frontend/package.json"), r#"{"name": "frontend"}"#).unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            assert_eq!(candidates.len(), 2);
            assert!(candidates.iter().any(|c| matches!(c.language, Language::Python { .. })));
            assert!(candidates.iter().any(|c| matches!(c.language, Language::JavaScript { .. })));
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_dedupes_same_language() {
    let dir = unique_test_dir("workspace-dedup");
    create_dir(&dir);
    create_dir(&dir.join("crates/foo"));
    fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/foo\"]\n").unwrap();
    fs::write(dir.join("crates/foo/Cargo.toml"), "[package]\nname = \"foo\"\n").unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            let rust_count = candidates
                .iter()
                .filter(|c| matches!(c.language, Language::Rust { .. }))
                .count();
            assert_eq!(rust_count, 1, "should dedupe to one Rust candidate");
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_skips_noise_directories() {
    let dir = unique_test_dir("noise-dirs");
    create_dir(&dir);
    create_dir(&dir.join("node_modules"));
    create_dir(&dir.join("target"));
    create_dir(&dir.join(".hidden"));
    fs::write(dir.join("node_modules/package.json"), r#"{"name": "dep"}"#).unwrap();
    fs::write(dir.join("target/Cargo.toml"), "[package]\nname = \"out\"\n").unwrap();
    fs::write(dir.join(".hidden/pyproject.toml"), "[project]\nname = \"secret\"\n").unwrap();

    let outcome = detect_project(&dir).unwrap();

    assert_eq!(outcome, DetectionOutcome::NoMatch);
}

#[test]
fn detect_project_keeps_higher_confidence_on_merge() {
    let dir = unique_test_dir("confidence-merge");
    create_dir(&dir);
    create_dir(&dir.join("lib"));
    // root: requirements.txt → Medium confidence
    fs::write(dir.join("requirements.txt"), "requests\n").unwrap();
    // subdir: pyproject.toml → High confidence
    fs::write(dir.join("lib/pyproject.toml"), "[project]\nname = \"lib\"\n").unwrap();

    let outcome = detect_project(&dir).unwrap();

    match outcome {
        DetectionOutcome::Matches { candidates } => {
            let python_candidates: Vec<_> = candidates
                .iter()
                .filter(|c| matches!(c.language, Language::Python { .. }))
                .collect();
            assert_eq!(python_candidates.len(), 1);
            assert_eq!(python_candidates[0].confidence, DetectionConfidence::High);
        }
        DetectionOutcome::NoMatch => panic!("expected matches"),
    }
}

#[test]
fn detect_project_empty_subdirs_no_match() {
    let dir = unique_test_dir("empty-subdirs");
    create_dir(&dir);
    create_dir(&dir.join("a"));
    create_dir(&dir.join("b"));

    let outcome = detect_project(&dir).unwrap();

    assert_eq!(outcome, DetectionOutcome::NoMatch);
}
