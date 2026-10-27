/// Evidence status after one retrieval attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceStatus {
    /// Current evidence is sufficient to answer.
    Sufficient,
    /// A required fact or artifact is missing.
    MissingRequired,
    /// Evidence contradicts itself and needs targeted clarification.
    Contradictory,
    /// Evidence exists but is insufficient for the requested output.
    Insufficient,
}

/// Borrowed evidence summary used for repeat/stop planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvidenceSummaryDraft<'a> {
    /// Evidence status.
    pub status: EvidenceStatus,
    /// Missing facts, artifacts, owners, dates, parameters, or sources.
    pub missing_evidence: &'a [&'a str],
    /// Retrieval attempts already performed for this budget.
    pub retrievals_performed: u32,
}

/// Validated evidence summary used for repeat/stop planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSummary {
    status: EvidenceStatus,
    missing_evidence: alloc::vec::Vec<alloc::string::String>,
    retrievals_performed: u32,
}

impl EvidenceSummary {
    /// Validates a retrieval evidence summary.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when missing-evidence names are absent
    /// for a status that requires them or any name is invalid.
    pub fn validate(draft: &EvidenceSummaryDraft<'_>) -> Result<Self, super::failure::Error> {
        let missing_evidence =
            super::check::missing_evidence(draft.status, draft.missing_evidence)?;
        Ok(Self {
            status: draft.status,
            missing_evidence,
            retrievals_performed: draft.retrievals_performed,
        })
    }

    /// Returns the evidence status.
    #[must_use]
    pub const fn status(&self) -> EvidenceStatus {
        self.status
    }

    /// Returns missing evidence labels.
    #[must_use]
    pub fn missing_evidence(&self) -> &[alloc::string::String] {
        self.missing_evidence.as_slice()
    }

    /// Returns retrieval count already performed.
    #[must_use]
    pub const fn retrievals_performed(&self) -> u32 {
        self.retrievals_performed
    }
}

/// Retrieval planning decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    /// Answer from current evidence and do not retrieve for polish.
    AnswerFromEvidence,
    /// Perform one targeted repeat retrieval for named missing evidence.
    TargetedRepeat {
        /// Missing evidence labels that justify the repeat.
        missing_evidence: alloc::vec::Vec<alloc::string::String>,
    },
    /// Ask for the smallest missing input because the budget cannot retrieve more.
    RequestSmallestMissingInput {
        /// Missing evidence labels that should be requested.
        missing_evidence: alloc::vec::Vec<alloc::string::String>,
    },
}

/// Safe retrieval receipt/report metadata draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataDraft<'a> {
    /// Safe source label.
    pub source_label: Option<&'a str>,
    /// Safe host reference or content handle.
    pub safe_reference: Option<&'a str>,
    /// Optional lowercase BLAKE3 content hash.
    pub content_hash: Option<&'a str>,
    /// Truncation status.
    pub truncation: Truncation,
    /// Raw fetched body; rejected by default receipt metadata.
    pub raw_body: Option<&'a str>,
}

/// Truncation status for retrieved content metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Truncation {
    /// Content was complete under host policy.
    Complete,
    /// Content was truncated before model or receipt visibility.
    Truncated,
}

/// Bounded retrieval evidence metadata safe for receipts and reports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    source_label: alloc::string::String,
    safe_reference: alloc::string::String,
    content_hash: Option<crate::artifact::Hash>,
    truncation: Truncation,
}

impl Metadata {
    /// Validates retrieval evidence metadata without raw fetched bodies.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when required safe metadata is missing,
    /// the reference is unsafe, the hash is invalid, or a raw body is present.
    pub fn validate(draft: &MetadataDraft<'_>) -> Result<Self, super::failure::Error> {
        super::check::raw_body(draft.raw_body)?;
        Ok(Self {
            source_label: super::check::required_text(super::SOURCE_FIELD, draft.source_label)?,
            safe_reference: super::check::safe_reference(
                super::REFERENCE_FIELD,
                draft.safe_reference,
            )?,
            content_hash: super::check::content_hash(draft.content_hash)?,
            truncation: draft.truncation,
        })
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns safe reference.
    #[must_use]
    pub fn safe_reference(&self) -> &str {
        self.safe_reference.as_str()
    }

    /// Returns optional BLAKE3 content hash.
    #[must_use]
    pub const fn content_hash(&self) -> Option<&crate::artifact::Hash> {
        self.content_hash.as_ref()
    }

    /// Returns truncation status.
    #[must_use]
    pub const fn truncation(&self) -> Truncation {
        self.truncation
    }
}

/// Plans whether retrieval should stop, repeat, or request missing input.
#[must_use]
pub fn plan(budget: &super::contract::Budget, evidence: &EvidenceSummary) -> Decision {
    match evidence.status() {
        EvidenceStatus::Sufficient => Decision::AnswerFromEvidence,
        EvidenceStatus::MissingRequired
        | EvidenceStatus::Contradictory
        | EvidenceStatus::Insufficient => plan_missing_evidence(budget, evidence),
    }
}

fn plan_missing_evidence(budget: &super::contract::Budget, evidence: &EvidenceSummary) -> Decision {
    if budget.repeat_allowed_after(evidence.retrievals_performed()) {
        return Decision::TargetedRepeat {
            missing_evidence: evidence.missing_evidence().to_vec(),
        };
    }
    Decision::RequestSmallestMissingInput {
        missing_evidence: evidence.missing_evidence().to_vec(),
    }
}
