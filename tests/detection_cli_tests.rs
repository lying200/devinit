use devinit::cli::LanguageChoice;
use devinit::detection::{DetectionConfidence, DetectionOutcome, LanguageCandidate};
use devinit::prompt::format_detected_summary;
use devinit::resolution::{ResolutionPlan, plan_language_resolution};
use devinit::schema::Language;

fn rust_candidate() -> LanguageCandidate {
    LanguageCandidate {
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
    }
}

#[test]
fn detected_summary_includes_language_and_reasons() {
    let summary = format_detected_summary(&rust_candidate());

    assert!(summary.contains("detected language: Rust"));
    assert!(summary.contains("detected channel: stable"));
    assert!(summary.contains("found Cargo.toml"));
}

#[test]
fn explicit_lang_skips_detection() {
    let outcome = DetectionOutcome::Match {
        candidate: rust_candidate(),
    };

    let plan = plan_language_resolution(Some(LanguageChoice::Java), outcome, true);

    assert_eq!(plan, ResolutionPlan::Explicit(LanguageChoice::Java));
}

#[test]
fn detection_acceptance_uses_detected_language() {
    let candidate = rust_candidate();
    let outcome = DetectionOutcome::Match {
        candidate: candidate.clone(),
    };

    let plan = plan_language_resolution(None, outcome, true);

    assert_eq!(plan, ResolutionPlan::UseDetected(candidate.language));
}

#[test]
fn detection_rejection_falls_back_to_manual_flow() {
    let outcome = DetectionOutcome::Match {
        candidate: rust_candidate(),
    };

    let plan = plan_language_resolution(None, outcome, false);

    assert_eq!(plan, ResolutionPlan::PromptManual);
}

#[test]
fn no_detection_match_falls_back_to_manual_flow() {
    let plan = plan_language_resolution(None, DetectionOutcome::NoMatch, false);

    assert_eq!(plan, ResolutionPlan::PromptManual);
}
