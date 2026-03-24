use std::{io, path::Path};

use super::detectors::run_detectors;
use super::types::{DetectionOutcome, LanguageCandidate};
use crate::schema::Language;

pub fn detect_project(target_dir: &Path) -> io::Result<DetectionOutcome> {
    let candidates = run_detectors(target_dir)?;
    Ok(select_primary_candidate(candidates))
}

pub fn select_primary_candidate(candidates: Vec<LanguageCandidate>) -> DetectionOutcome {
    let mut candidates = candidates;
    candidates.sort_by_key(priority_key);

    match candidates.into_iter().next() {
        Some(candidate) => DetectionOutcome::Match { candidate },
        None => DetectionOutcome::NoMatch,
    }
}

fn priority_key(candidate: &LanguageCandidate) -> usize {
    match candidate.language {
        Language::Rust { .. } => 0,
        Language::Python { .. } => 1,
        Language::Go { .. } => 2,
        Language::Java { .. } => 3,
        Language::JavaScript { .. } => 4,
    }
}
