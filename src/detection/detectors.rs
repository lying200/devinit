use std::path::{Path, PathBuf};
use std::{fs, io, mem};

use super::types::{DetectionConfidence, LanguageCandidate};

pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "vendor",
    "__pycache__",
    "dist",
    "build",
];

/// Runs all language detectors for `target_dir` and its immediate subdirectories.
///
/// # Errors
///
/// Returns any I/O error produced by an individual detector or directory scan.
pub fn run_detectors(target_dir: &Path) -> io::Result<Vec<LanguageCandidate>> {
    let dirs = scannable_dirs(target_dir)?;
    let mut all_candidates = Vec::new();

    for dir in &dirs {
        detect_in_dir(dir, &mut all_candidates)?;
    }

    Ok(merge_candidates(all_candidates))
}

fn detect_in_dir(dir: &Path, candidates: &mut Vec<LanguageCandidate>) -> io::Result<()> {
    if let Some(c) = rust::detect(dir)? {
        candidates.push(c);
    }
    if let Some(c) = python::detect(dir)? {
        candidates.push(c);
    }
    if let Some(c) = go::detect(dir)? {
        candidates.push(c);
    }
    if let Some(c) = java::detect(dir)? {
        candidates.push(c);
    }
    if let Some(c) = javascript::detect(dir)? {
        candidates.push(c);
    }
    Ok(())
}

fn scannable_dirs(target_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut dirs = vec![target_dir.to_path_buf()];

    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if name_str.starts_with('.') || SKIP_DIRS.contains(&name_str) {
            continue;
        }
        dirs.push(entry.path());
    }

    Ok(dirs)
}

fn confidence_rank(c: DetectionConfidence) -> u8 {
    match c {
        DetectionConfidence::High => 2,
        DetectionConfidence::Medium => 1,
        DetectionConfidence::Low => 0,
    }
}

fn merge_candidates(all: Vec<LanguageCandidate>) -> Vec<LanguageCandidate> {
    let mut merged: Vec<LanguageCandidate> = Vec::new();

    for candidate in all {
        if let Some(existing) = merged
            .iter_mut()
            .find(|m| mem::discriminant(&m.language) == mem::discriminant(&candidate.language))
        {
            if confidence_rank(candidate.confidence) > confidence_rank(existing.confidence) {
                existing.reasons.extend(candidate.reasons);
                existing.confidence = candidate.confidence;
                existing.language = candidate.language;
            } else {
                existing.reasons.extend(candidate.reasons);
            }
        } else {
            merged.push(candidate);
        }
    }

    merged
}
