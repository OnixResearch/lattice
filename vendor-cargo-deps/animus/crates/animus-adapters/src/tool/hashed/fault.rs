/// Hashed edit validation and receipt-construction errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A common adapter field check failed.
    Adapter(crate::AdapterContractError),
    /// A label parsed successfully but is unsupported by this contract.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
    /// A hash field failed lowercase BLAKE3 validation.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// A bounded collection exceeded the hashed edit limit.
    TooManyItems {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A line number fell outside the snapshot line count.
    LineNumberOutOfRange {
        /// Invalid one-based line number.
        line_number: u32,
        /// Snapshot line count.
        line_count: u32,
    },
    /// A snapshot repeated one tag for multiple lines.
    DuplicateTag {
        /// Repeated tag.
        tag: super::Text,
    },
    /// A snapshot repeated one line number for multiple tag descriptors.
    AmbiguousLine {
        /// Repeated one-based line number.
        line_number: u32,
    },
    /// A span selector referenced a tag not present in the snapshot.
    UnknownTag {
        /// Missing tag.
        tag: super::Text,
    },
    /// A tag lookup found more than one matching descriptor.
    AmbiguousTag {
        /// Ambiguous tag.
        tag: super::Text,
    },
    /// A request hash did not identify the supplied snapshot.
    SnapshotMismatch {
        /// Hash supplied by the request.
        request_hash: super::Text,
        /// Hash carried by the supplied snapshot.
        snapshot_hash: super::Text,
    },
    /// The host-reported current snapshot differs from the requested CAS token.
    StaleSnapshot {
        /// Expected snapshot hash supplied by the request.
        expected_hash: super::Text,
        /// Current snapshot hash reported by the host.
        current_hash: super::Text,
    },
    /// A span start line came after its end line.
    InvalidSpan {
        /// Start line selected by the span.
        start_line: u32,
        /// End line selected by the span.
        end_line: u32,
    },
    /// The same span selector was supplied more than once.
    DuplicateSpan {
        /// Repeated start tag.
        start_tag: super::Text,
        /// Repeated end tag.
        end_tag: super::Text,
    },
    /// Requested spans were not sorted in source order.
    OutOfOrderSpan {
        /// Previous start line.
        previous_start_line: u32,
        /// Current start line.
        current_start_line: u32,
    },
    /// Requested spans overlap.
    OverlappingSpan {
        /// Previous end line.
        previous_end_line: u32,
        /// Current start line.
        current_start_line: u32,
    },
    /// Replacement text exceeded the hashed edit bound.
    ReplacementTooLong {
        /// Maximum accepted replacement characters.
        max_chars: u64,
        /// Actual replacement characters.
        actual_chars: u64,
    },
    /// A receipt redaction mode required a resource identity.
    MissingResourceIdentity {
        /// Redaction mode that requires identity text.
        redaction: super::ResourceRedaction,
    },
    /// Omitted resource redaction was paired with identity text.
    UnexpectedResourceIdentity {
        /// Redaction mode that rejects identity text.
        redaction: super::ResourceRedaction,
    },
    /// A receipt resource identity looked like a private host locator.
    UnsafeResourceIdentity {
        /// Rejected identity text.
        resource_identity: super::Text,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Adapter(error)
    }
}
