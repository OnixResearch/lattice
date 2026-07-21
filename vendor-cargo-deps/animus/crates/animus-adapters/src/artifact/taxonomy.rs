/// Artifact payload class used in canonical envelopes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// Runtime receipt artifact.
    RuntimeReceipt,
    /// Provenance observation artifact.
    ProvenanceObservation,
    /// Model boundary metadata artifact.
    ModelBoundary,
    /// Tool boundary metadata artifact.
    ToolBoundary,
    /// Review or release-evidence summary artifact.
    ReviewSummary,
}

impl Kind {
    /// Returns a stable kind label for canonical hashing and display.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeReceipt => "runtime_receipt",
            Self::ProvenanceObservation => "provenance_observation",
            Self::ModelBoundary => "model_boundary",
            Self::ToolBoundary => "tool_boundary",
            Self::ReviewSummary => "review_summary",
        }
    }
}

/// Redaction class controlling payload display.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RedactionClass {
    /// Payload is intentionally public and may be displayed by default.
    Public,
    /// Payload contains metadata only and body display is omitted.
    MetadataOnly,
    /// Payload exists but must be replaced by an explicit redaction marker.
    RedactedPayload,
    /// Payload is secret and must never be displayed by default.
    Secret,
}

impl RedactionClass {
    /// Returns a stable redaction-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::MetadataOnly => "metadata_only",
            Self::RedactedPayload => "redacted_payload",
            Self::Secret => "secret",
        }
    }

    /// Reports whether the payload body may be displayed by default.
    #[must_use]
    pub const fn allows_payload_display(self) -> bool {
        matches!(self, Self::Public)
    }
}
