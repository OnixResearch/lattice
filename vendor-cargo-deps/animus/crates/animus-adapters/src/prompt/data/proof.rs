use super::carrier::ContextReferenceFragment;
use super::failure::ContextReferenceError;
use super::forms::ContextReferenceKind;
use super::rules::ContextReferencePolicyDecision;
use super::rules::ContextReferenceStatus;
use super::rules::ContextResolverKind;
use super::rules::ContextTruncationState;
use crate::prompt::base::BYTE_COUNT_LABEL;
use crate::prompt::base::CONTEXT_BOUNDS_FIELD;
use crate::prompt::base::LINE_COUNT_LABEL;
use crate::prompt::base::VISIBLE_BYTE_COUNT_LABEL;
use crate::prompt::base::VISIBLE_LINE_COUNT_LABEL;
use crate::prompt::base::ZERO_COUNT;

/// Original and model-visible context-reference count metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextReferenceCounts {
    total_bytes: u32,
    total_lines: u32,
    visible_bytes: u32,
    visible_lines: u32,
}

impl ContextReferenceCounts {
    /// Builds count metadata for provenance validation.
    #[must_use]
    pub const fn new(byte_count: u32, line_count: u32, visible_byte_count: u32, visible_line_count: u32) -> Self {
        Self {
            total_bytes: byte_count,
            total_lines: line_count,
            visible_bytes: visible_byte_count,
            visible_lines: visible_line_count,
        }
    }
}

/// Borrowed provenance metadata before safe receipt validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextReferenceProvenanceDraft<'a> {
    kind: ContextReferenceKind,
    status: ContextReferenceStatus,
    safe_target_label: &'a str,
    resolver_kind: ContextResolverKind,
    policy_decision: ContextReferencePolicyDecision,
    truncation: ContextTruncationState,
    byte_count: u32,
    line_count: u32,
    visible_byte_count: u32,
    visible_line_count: u32,
    content_hash: Option<&'a crate::ArtifactHash>,
}

impl<'a> ContextReferenceProvenanceDraft<'a> {
    /// Builds a draft with required safe classification fields.
    #[must_use]
    pub const fn new(
        kind: ContextReferenceKind,
        status: ContextReferenceStatus,
        safe_target_label: &'a str,
        resolver_kind: ContextResolverKind,
    ) -> Self {
        Self {
            kind,
            status,
            safe_target_label,
            resolver_kind,
            policy_decision: ContextReferencePolicyDecision::Allowed,
            truncation: ContextTruncationState::None,
            byte_count: ZERO_COUNT,
            line_count: ZERO_COUNT,
            visible_byte_count: ZERO_COUNT,
            visible_line_count: ZERO_COUNT,
            content_hash: None,
        }
    }

    /// Builds a draft from a budgeted fragment.
    #[must_use]
    pub fn from_fragment(
        fragment: &'a ContextReferenceFragment,
        policy_decision: ContextReferencePolicyDecision,
    ) -> Self {
        Self {
            kind: fragment.reference.kind,
            status: fragment.status,
            safe_target_label: fragment.reference.safe_target_label.as_str(),
            resolver_kind: fragment.resolver_kind,
            policy_decision,
            truncation: fragment.truncation,
            byte_count: fragment.byte_count,
            line_count: fragment.line_count,
            visible_byte_count: fragment.visible_byte_count,
            visible_line_count: fragment.visible_line_count,
            content_hash: Some(&fragment.content_hash),
        }
    }

    /// Returns a copy with a policy decision.
    #[must_use]
    pub const fn with_policy_decision(mut self, decision: ContextReferencePolicyDecision) -> Self {
        self.policy_decision = decision;
        self
    }

    /// Returns a copy with truncation state.
    #[must_use]
    pub const fn with_truncation(mut self, truncation: ContextTruncationState) -> Self {
        self.truncation = truncation;
        self
    }

    /// Returns a copy with original and visible counts.
    #[must_use]
    pub const fn with_counts(mut self, counts: ContextReferenceCounts) -> Self {
        self.byte_count = counts.total_bytes;
        self.line_count = counts.total_lines;
        self.visible_byte_count = counts.visible_bytes;
        self.visible_line_count = counts.visible_lines;
        self
    }

    /// Returns a copy with a BLAKE3 content hash.
    #[must_use]
    pub const fn with_content_hash(mut self, hash: &'a crate::ArtifactHash) -> Self {
        self.content_hash = Some(hash);
        self
    }
}

/// Validated context-reference provenance record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextReferenceProvenance {
    receipt: crate::safe::receipt::Record,
}

impl ContextReferenceProvenance {
    /// Builds provenance after validating every metadata field for leaks.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when labels, counts, hashes, or
    /// truncation metadata are unsafe or inconsistent.
    pub fn new(draft: ContextReferenceProvenanceDraft<'_>) -> Result<Self, ContextReferenceError> {
        reject_unbounded_output(&draft)?;
        let labels = [
            draft.kind.as_str(),
            draft.status.as_str(),
            draft.safe_target_label,
            draft.resolver_kind.as_str(),
            draft.policy_decision.as_str(),
            draft.truncation.as_str(),
        ];
        let counts = [
            crate::safe::receipt::BoundedCountDraft::new(BYTE_COUNT_LABEL, draft.byte_count),
            crate::safe::receipt::BoundedCountDraft::new(LINE_COUNT_LABEL, draft.line_count),
            crate::safe::receipt::BoundedCountDraft::new(VISIBLE_BYTE_COUNT_LABEL, draft.visible_byte_count),
            crate::safe::receipt::BoundedCountDraft::new(VISIBLE_LINE_COUNT_LABEL, draft.visible_line_count),
        ];
        let redactions = [
            crate::safe::receipt::Redaction::OutputBounded,
            crate::safe::receipt::Redaction::PayloadHashOnly,
        ];
        let hash_storage = draft.content_hash.cloned().map(|hash| [hash]);
        let empty_hashes: &[crate::ArtifactHash] = &[];
        let hashes = match &hash_storage {
            Some(values) => values.as_slice(),
            None => empty_hashes,
        };
        let receipt = crate::safe::receipt::Record::new(
            crate::safe::receipt::Draft::new(
                receipt_status(draft.status),
                crate::safe::receipt::IssueCode::ProvenanceObservation,
                crate::safe::receipt::Operation::ProvenanceObservation,
            )
            .with_labels(&labels)
            .with_counts(&counts)
            .with_artifact_hashes(hashes)
            .with_redactions(&redactions),
        )
        .map_err(ContextReferenceError::SafeMetadata)?;
        Ok(Self { receipt })
    }

    /// Returns the reusable safe receipt record.
    #[must_use]
    pub const fn receipt(&self) -> &crate::safe::receipt::Record {
        &self.receipt
    }

    /// Renders bounded review text without raw expanded content.
    #[must_use]
    pub fn review_summary(&self) -> alloc::string::String {
        self.receipt.review_summary()
    }
}

fn reject_unbounded_output(draft: &ContextReferenceProvenanceDraft<'_>) -> Result<(), ContextReferenceError> {
    let is_visible_smaller = draft.visible_byte_count < draft.byte_count || draft.visible_line_count < draft.line_count;
    if draft.status == ContextReferenceStatus::Included
        && draft.truncation == ContextTruncationState::None
        && is_visible_smaller
    {
        return Err(ContextReferenceError::SafeMetadata(crate::safe::receipt::Error::UnsafeMetadata {
            field: CONTEXT_BOUNDS_FIELD,
            class: crate::safe::receipt::LeakClass::UnboundedOutput,
        }));
    }
    Ok(())
}

fn receipt_status(status: ContextReferenceStatus) -> crate::safe::receipt::Status {
    match status {
        ContextReferenceStatus::Included => crate::safe::receipt::Status::Completed,
        ContextReferenceStatus::Truncated => crate::safe::receipt::Status::Redacted,
        ContextReferenceStatus::Rejected | ContextReferenceStatus::Denied => crate::safe::receipt::Status::Denied,
        ContextReferenceStatus::MissingResolver => crate::safe::receipt::Status::Blocked,
        ContextReferenceStatus::Unavailable => crate::safe::receipt::Status::Failed,
    }
}
