pub mod engine;
pub mod detectors;
pub mod types;

pub use engine::detect_project;
pub use types::{DetectionConfidence, DetectionOutcome, LanguageCandidate};
