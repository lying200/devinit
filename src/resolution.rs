use crate::{
    cli::LanguageChoice,
    detection::DetectionOutcome,
    schema::Language,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionPlan {
    Explicit(LanguageChoice),
    UseDetected(Language),
    PromptManual,
}

pub fn plan_language_resolution(
    cli_lang: Option<LanguageChoice>,
    detection: DetectionOutcome,
    use_detected: bool,
) -> ResolutionPlan {
    if let Some(choice) = cli_lang {
        return ResolutionPlan::Explicit(choice);
    }

    match detection {
        DetectionOutcome::NoMatch => ResolutionPlan::PromptManual,
        DetectionOutcome::Match { candidate } if use_detected => {
            ResolutionPlan::UseDetected(candidate.language)
        }
        DetectionOutcome::Match { .. } => ResolutionPlan::PromptManual,
    }
}
