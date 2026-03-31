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

fn js_candidate() -> LanguageCandidate {
    LanguageCandidate {
        language: Language::JavaScript {
            package: None,
            package_manager: Some("pnpm".to_string()),
            corepack_enable: None,
        },
        confidence: DetectionConfidence::High,
        reasons: vec!["found package.json".to_string()],
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
fn explicit_langs_skip_detection() {
    let outcome = DetectionOutcome::Matches {
        candidates: vec![rust_candidate()],
    };

    let plan = plan_language_resolution(
        &[LanguageChoice::Java],
        outcome,
        &[],
    );

    assert_eq!(plan, ResolutionPlan::Explicit(vec![LanguageChoice::Java]));
}

#[test]
fn explicit_multiple_langs() {
    let plan = plan_language_resolution(
        &[LanguageChoice::Go, LanguageChoice::JavaScript],
        DetectionOutcome::NoMatch,
        &[],
    );

    assert_eq!(
        plan,
        ResolutionPlan::Explicit(vec![LanguageChoice::Go, LanguageChoice::JavaScript])
    );
}

#[test]
fn detection_acceptance_uses_detected_languages() {
    let candidate = rust_candidate();
    let outcome = DetectionOutcome::Matches {
        candidates: vec![candidate.clone()],
    };

    let plan = plan_language_resolution(&[], outcome, &[0]);

    assert_eq!(
        plan,
        ResolutionPlan::UseDetected(vec![candidate.language])
    );
}

#[test]
fn detection_acceptance_multiple_languages() {
    let rust = rust_candidate();
    let js = js_candidate();
    let outcome = DetectionOutcome::Matches {
        candidates: vec![rust.clone(), js.clone()],
    };

    let plan = plan_language_resolution(&[], outcome, &[0, 1]);

    assert_eq!(
        plan,
        ResolutionPlan::UseDetected(vec![rust.language, js.language])
    );
}

#[test]
fn detection_partial_selection() {
    let rust = rust_candidate();
    let js = js_candidate();
    let outcome = DetectionOutcome::Matches {
        candidates: vec![rust.clone(), js.clone()],
    };

    let plan = plan_language_resolution(&[], outcome, &[1]);

    assert_eq!(
        plan,
        ResolutionPlan::UseDetected(vec![js.language])
    );
}

#[test]
fn detection_rejection_falls_back_to_manual_flow() {
    let outcome = DetectionOutcome::Matches {
        candidates: vec![rust_candidate()],
    };

    let plan = plan_language_resolution(&[], outcome, &[]);

    assert_eq!(plan, ResolutionPlan::PromptManual);
}

#[test]
fn no_detection_match_falls_back_to_manual_flow() {
    let plan = plan_language_resolution(&[], DetectionOutcome::NoMatch, &[]);

    assert_eq!(plan, ResolutionPlan::PromptManual);
}
