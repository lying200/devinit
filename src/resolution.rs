use crate::{
    cli::LanguageChoice,
    detection::DetectionOutcome,
    schema::Language,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionPlan {
    Explicit(Vec<LanguageChoice>),
    UseDetected(Vec<Language>),
    PromptManual,
}

pub fn plan_language_resolution(
    cli_langs: &[LanguageChoice],
    detection: DetectionOutcome,
    confirmed_indices: &[usize],
) -> ResolutionPlan {
    if !cli_langs.is_empty() {
        return ResolutionPlan::Explicit(cli_langs.to_vec());
    }

    match detection {
        DetectionOutcome::NoMatch => ResolutionPlan::PromptManual,
        DetectionOutcome::Matches { candidates } => {
            let confirmed: Vec<Language> = confirmed_indices
                .iter()
                .filter_map(|&i| candidates.get(i).map(|c| c.language.clone()))
                .collect();
            if confirmed.is_empty() {
                ResolutionPlan::PromptManual
            } else {
                ResolutionPlan::UseDetected(confirmed)
            }
        }
    }
}
