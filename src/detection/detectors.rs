use std::{io, path::Path};

use super::types::LanguageCandidate;

pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

pub fn run_detectors(_target_dir: &Path) -> io::Result<Vec<LanguageCandidate>> {
    let mut candidates = Vec::new();

    if let Some(candidate) = rust::detect(_target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = python::detect(_target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = go::detect(_target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = java::detect(_target_dir)? {
        candidates.push(candidate);
    }
    if let Some(candidate) = javascript::detect(_target_dir)? {
        candidates.push(candidate);
    }

    Ok(candidates)
}
