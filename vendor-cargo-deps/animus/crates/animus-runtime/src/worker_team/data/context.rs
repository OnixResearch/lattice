// Public root reexports intentionally keep the `Worker*` prefix for embedders.
#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

/// Admission policy for unsafe context item kinds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContextAdmissionPolicy {
    /// Explicitly admitted unsafe context item kinds.
    pub admitted_unsafe_kinds: Vec<WorkerContextItemKind>,
}

impl ContextAdmissionPolicy {
    /// Returns true when the item kind is safe by default or explicitly admitted.
    #[must_use]
    pub fn admits(&self, kind: WorkerContextItemKind) -> bool {
        kind.is_default_safe() || self.admitted_unsafe_kinds.contains(&kind)
    }
}

/// Minimized packet of parent context selected for a worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerContextPacket {
    /// Stable safe packet identity.
    pub packet_id: String,
    /// Policy for explicitly admitted unsafe context classes.
    pub policy: ContextAdmissionPolicy,
    /// Selected context items.
    pub items: Vec<WorkerContextItem>,
}

/// One selected context item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerContextItem {
    /// Context item kind.
    pub kind: WorkerContextItemKind,
    /// Safe item label.
    pub label: String,
    /// Bounded text body, summary, or reference.
    pub body: String,
    /// Redaction posture for the body.
    pub redaction: WorkerContextRedaction,
}

/// Context item kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerContextItemKind {
    /// Selected parent message excerpt.
    ParentMessage,
    /// Parent-provided summary.
    Summary,
    /// Bounded file observation.
    FileObservation,
    /// Safe memory descriptor.
    MemoryDescriptor,
    /// Bounded tool result summary.
    ToolResult,
    /// Full parent transcript, unsafe by default.
    FullTranscript,
    /// Raw hidden memory, unsafe by default.
    RawHiddenMemory,
    /// Credential material, unsafe by default.
    Credential,
    /// Provider-native payload, unsafe by default.
    ProviderPayload,
    /// Unbounded tool output, unsafe by default.
    UnboundedToolOutput,
}

impl WorkerContextItemKind {
    /// Returns true when this item kind may appear without explicit admission.
    #[must_use]
    pub const fn is_default_safe(self) -> bool {
        matches!(
            self,
            Self::ParentMessage | Self::Summary | Self::FileObservation | Self::MemoryDescriptor | Self::ToolResult
        )
    }
}

/// Redaction posture for context item text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerContextRedaction {
    /// Body is a bounded safe summary.
    SafeSummary,
    /// Body is a redacted reference label.
    RedactedReference,
    /// Body is raw material and must pass leak validation.
    Raw,
}
