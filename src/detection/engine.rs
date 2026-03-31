use std::{io, path::Path};

use super::detectors::run_detectors;
use super::types::DetectionOutcome;
use crate::schema::Language;

pub fn detect_project(target_dir: &Path) -> io::Result<DetectionOutcome> {
    let mut candidates = run_detectors(target_dir)?;
    if candidates.is_empty() {
        Ok(DetectionOutcome::NoMatch)
    } else {
        candidates.sort_by_key(priority_key);
        Ok(DetectionOutcome::Matches { candidates })
    }
}

fn priority_key(candidate: &super::types::LanguageCandidate) -> usize {
    match candidate.language {
        Language::Rust { .. } => 0,
        Language::Python { .. } => 1,
        Language::Go { .. } => 2,
        Language::Java { .. } => 3,
        Language::JavaScript { .. } => 4,
    }
}
