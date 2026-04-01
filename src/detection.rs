pub mod detectors;
pub mod engine;
pub mod services;
pub mod types;

pub use engine::detect_project;
pub use services::{ServiceCandidate, detect_services};
pub use types::{DetectionConfidence, DetectionOutcome, LanguageCandidate};
