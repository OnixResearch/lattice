/// Bounded explanation for a status selection decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionExplanation {
    target_id: String,
    selected_authority: Option<super::types::AuthoritySummary>,
    rejected_candidates: Vec<super::types::RejectedCandidate>,
    freshness: super::types::Freshness,
    fallback_reason: Option<super::types::FallbackReason>,
    evidence_hashes: Vec<String>,
    evidence_summaries: Vec<super::safe::EvidenceSummary>,
}

impl DecisionExplanation {
    pub(super) fn new(parts: DecisionExplanationParts<'_>) -> Self {
        Self {
            target_id: String::from(parts.target_id),
            selected_authority: parts.selected_authority,
            rejected_candidates: parts.rejected_candidates,
            freshness: parts.freshness,
            fallback_reason: parts.fallback_reason,
            evidence_hashes: parts.evidence_hashes,
            evidence_summaries: vec![parts.evidence_summary],
        }
    }

    /// Returns the target identity for this explanation.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.target_id.as_str()
    }

    /// Returns the selected authority summary, if any.
    #[must_use]
    pub const fn selected_authority(&self) -> Option<&super::types::AuthoritySummary> {
        self.selected_authority.as_ref()
    }

    /// Returns rejected candidates and reasons.
    #[must_use]
    pub fn rejected_candidates(&self) -> &[super::types::RejectedCandidate] {
        self.rejected_candidates.as_slice()
    }

    /// Returns the freshness class.
    #[must_use]
    pub const fn freshness(&self) -> super::types::Freshness {
        self.freshness
    }

    /// Returns the fallback reason, if the selected state used fallback/unknown.
    #[must_use]
    pub const fn fallback_reason(&self) -> Option<super::types::FallbackReason> {
        self.fallback_reason
    }

    /// Returns safe evidence hashes associated with the selected authority.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[String] {
        self.evidence_hashes.as_slice()
    }

    /// Returns bounded redacted evidence summaries.
    #[must_use]
    pub fn evidence_summaries(&self) -> &[super::safe::EvidenceSummary] {
        self.evidence_summaries.as_slice()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DecisionExplanationParts<'a> {
    pub(super) target_id: &'a str,
    pub(super) selected_authority: Option<super::types::AuthoritySummary>,
    pub(super) rejected_candidates: Vec<super::types::RejectedCandidate>,
    pub(super) freshness: super::types::Freshness,
    pub(super) fallback_reason: Option<super::types::FallbackReason>,
    pub(super) evidence_hashes: Vec<String>,
    pub(super) evidence_summary: super::safe::EvidenceSummary,
}
