use crate::schema::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCandidate {
    pub language: Language,
    pub confidence: DetectionConfidence,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionOutcome {
    NoMatch,
    Matches { candidates: Vec<LanguageCandidate> },
}
