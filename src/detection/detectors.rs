use std::{io, path::Path};

use super::types::LanguageCandidate;

pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

/// Runs all language detectors for `target_dir`.
///
/// # Errors
///
/// Returns any I/O error produced by an individual detector.
pub fn run_detectors(target_dir: &Path) -> io::Result<Vec<LanguageCandidate>> {
    let mut candidates = Vec::new();

    if let Some(candidate) = rust::detect(target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = python::detect(target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = go::detect(target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = java::detect(target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = javascript::detect(target_dir)? {
        candidates.push(candidate);
    }

    Ok(candidates)
}
