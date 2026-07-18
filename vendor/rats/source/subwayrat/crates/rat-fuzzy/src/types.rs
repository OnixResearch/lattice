//! Candidate and data source types.

/// A single searchable candidate.
#[derive(Debug, Clone)]
pub struct FuzzyCandidate {
    pub id: String,
    pub text: String,
    pub context: Option<String>,
    pub icon: Option<char>,
}

/// Trait for providing candidate data. Object-safe.
pub trait FuzzySource {
    fn candidates(&self) -> &[FuzzyCandidate];
}

impl FuzzySource for Vec<FuzzyCandidate> {
    fn candidates(&self) -> &[FuzzyCandidate] {
        self.as_slice()
    }
}
