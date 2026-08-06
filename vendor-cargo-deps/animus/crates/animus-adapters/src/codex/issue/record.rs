/// Review finding severity labels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewFindingSeverity {
    /// Informational finding.
    Info,
    /// Low-severity finding.
    Low,
    /// Medium-severity finding.
    Medium,
    /// High-severity finding.
    High,
    /// Critical finding.
    Critical,
}

/// Review finding categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewFindingCategory {
    /// Correctness issue.
    Correctness,
    /// Safety or security issue.
    Safety,
    /// Test coverage issue.
    Testing,
    /// Documentation issue.
    Documentation,
    /// Maintainability issue.
    Maintainability,
}

/// Borrowed review finding before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewFindingDraft<'a> {
    /// Stable finding identity.
    pub finding_id: &'a str,
    /// Severity label.
    pub severity: DiffReviewFindingSeverity,
    /// Finding category.
    pub category: DiffReviewFindingCategory,
    /// Affected source span.
    pub span: super::DiffReviewSpanDraft<'a>,
    /// Bounded evidence.
    pub evidence: super::DiffReviewEvidenceDraft<'a>,
    /// Safe rationale summary.
    pub rationale: &'a str,
    /// Normalized confidence score in the inclusive zero-to-one-hundred range.
    pub confidence: u8,
    /// Safe remediation guidance.
    pub remediation: &'a str,
}

/// Validated structured review finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewFinding {
    finding_id: super::super::Text,
    severity: DiffReviewFindingSeverity,
    category: DiffReviewFindingCategory,
    span: super::DiffReviewSpan,
    evidence: super::DiffReviewEvidence,
    rationale: super::super::Text,
    confidence: u8,
    remediation: super::super::Text,
}

impl DiffReviewFinding {
    /// Validates a structured review finding.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::DiffReviewError`] when identity, span, evidence,
    /// rationale, remediation, or confidence fields are invalid.
    pub fn validate(draft: &DiffReviewFindingDraft<'_>) -> Result<Self, super::super::DiffReviewError> {
        let finding_id = super::super::check::required_safe_text(super::super::FINDING_ID_FIELD, draft.finding_id)?;
        let span = super::DiffReviewSpan::validate(&draft.span)?;
        let evidence = super::DiffReviewEvidence::validate(&draft.evidence)?;
        let rationale =
            super::super::check::required_safe_text(super::super::FINDING_RATIONALE_FIELD, draft.rationale)?;
        let remediation =
            super::super::check::required_safe_text(super::super::FINDING_REMEDIATION_FIELD, draft.remediation)?;
        super::super::check::confidence(super::super::check::ScoreBounds {
            score: draft.confidence,
            max_score: super::super::MAX_CONFIDENCE_SCORE,
        })?;
        Ok(Self {
            finding_id,
            severity: draft.severity,
            category: draft.category,
            span,
            evidence,
            rationale,
            confidence: draft.confidence,
            remediation,
        })
    }

    /// Returns the stable finding identity.
    #[must_use]
    pub fn finding_id(&self) -> &str {
        self.finding_id.as_str()
    }

    /// Returns the finding severity.
    #[must_use]
    pub const fn severity(&self) -> DiffReviewFindingSeverity {
        self.severity
    }

    /// Returns the finding category.
    #[must_use]
    pub const fn category(&self) -> DiffReviewFindingCategory {
        self.category
    }

    /// Returns the affected source span.
    #[must_use]
    pub const fn span(&self) -> &super::DiffReviewSpan {
        &self.span
    }

    /// Returns the bounded evidence.
    #[must_use]
    pub const fn evidence(&self) -> &super::DiffReviewEvidence {
        &self.evidence
    }

    /// Returns the safe rationale summary.
    #[must_use]
    pub fn rationale(&self) -> &str {
        self.rationale.as_str()
    }

    /// Returns the normalized confidence score.
    #[must_use]
    pub const fn confidence(&self) -> u8 {
        self.confidence
    }

    /// Returns the safe remediation guidance.
    #[must_use]
    pub fn remediation(&self) -> &str {
        self.remediation.as_str()
    }
}
