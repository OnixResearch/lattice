/// Validation errors raised by dedicated diff-review contracts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiffReviewError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A collection exceeded its item ceiling.
    CollectionTooLarge {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A collection contained a duplicate value.
    DuplicateValue {
        /// Boundary collection field name.
        field: &'static str,
        /// Duplicated normalized value.
        value: super::Text,
    },
    /// The active adapter cannot supply the selected change set.
    UnsupportedSelector {
        /// Requested selector kind.
        kind: super::input::DiffReviewSelectorKind,
    },
    /// Selector references did not match the selector kind.
    InvalidSelectorShape {
        /// Requested selector kind.
        kind: super::input::DiffReviewSelectorKind,
    },
    /// A finding span was impossible or stale for the reviewed change set.
    InvalidSpan {
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// Evidence kind is not embeddable in a review report.
    UnsupportedEvidenceKind {
        /// Rejected evidence kind.
        kind: super::issue::DiffReviewEvidenceKind,
    },
    /// Evidence exceeded a bounded report limit.
    UnboundedEvidence {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted value.
        max_value: u64,
        /// Actual value.
        actual_value: u64,
    },
    /// Evidence or labels contained unsafe raw material.
    UnsafeEvidence {
        /// Boundary field name.
        field: &'static str,
        /// Marker that triggered rejection.
        marker: &'static str,
    },
    /// Finding confidence was outside the normalized score range.
    InvalidConfidence {
        /// Maximum accepted score.
        max_score: u8,
        /// Actual score.
        actual_score: u8,
    },
    /// A finding referenced a source outside the reviewed change set.
    FindingOutsideReviewScope {
        /// Finding identity.
        finding_id: super::Text,
        /// Referenced source label.
        source_label: super::Text,
    },
}

impl From<crate::AdapterContractError> for DiffReviewError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}
