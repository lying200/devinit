use devinit::detection::{
    DetectionConfidence, DetectionOutcome, LanguageCandidate, detect_project,
    engine::select_primary_candidate,
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
fn detection_types_support_single_language_match() {
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

    let outcome = DetectionOutcome::Match { candidate };

    match outcome {
        DetectionOutcome::Match { candidate } => {
            assert_eq!(candidate.confidence, DetectionConfidence::High);
            assert_eq!(candidate.reasons, vec!["found Cargo.toml"]);
        }
        DetectionOutcome::NoMatch => panic!("expected match"),
    }
}

#[test]
fn select_primary_candidate_returns_no_match_for_empty_list() {
    let outcome = select_primary_candidate(Vec::new());

    assert_eq!(outcome, DetectionOutcome::NoMatch);
}

#[test]
fn select_primary_candidate_returns_single_candidate() {
    let candidate = LanguageCandidate {
        language: Language::Go {
            version: Some("1.24".to_string()),
            package: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found go.mod".to_string()],
    };

    let outcome = select_primary_candidate(vec![candidate.clone()]);

    assert_eq!(outcome, DetectionOutcome::Match { candidate });
}

#[test]
fn select_primary_candidate_prefers_rust_over_javascript() {
    let rust = LanguageCandidate {
        language: Language::Rust {
            channel: None,
            version: Some("1.76.0".to_string()),
            components: None,
            targets: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found Cargo.toml".to_string()],
    };
    let javascript = LanguageCandidate {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("pnpm".to_string()),
            corepack_enable: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found package.json".to_string()],
    };

    let outcome = select_primary_candidate(vec![javascript, rust.clone()]);

    assert_eq!(outcome, DetectionOutcome::Match { candidate: rust });
}

#[test]
fn detect_project_returns_rust_match_for_cargo_project() {
    let dir = unique_test_dir("cargo-project");
    create_dir(&dir);
    fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();

    let outcome = detect_project(&dir).unwrap();

    assert_eq!(
        outcome,
        DetectionOutcome::Match {
            candidate: LanguageCandidate {
                language: Language::Rust {
                    channel: None,
                    version: None,
                    components: None,
                    targets: None,
                },
                confidence: DetectionConfidence::High,
                reasons: vec!["found Cargo.toml".to_string()],
            }
        }
    );
}
