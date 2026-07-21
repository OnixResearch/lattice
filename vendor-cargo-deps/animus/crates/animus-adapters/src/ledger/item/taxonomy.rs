/// First-class summary semantics for model-visible replay.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SummaryKind {
    /// Summary replaces earlier compacted context on the active branch.
    Compaction,
    /// Summary carries bounded handoff context from another branch.
    BranchHandoff,
}

impl SummaryKind {
    /// Returns a stable summary-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compaction => "compaction",
            Self::BranchHandoff => "branch_handoff",
        }
    }
}

/// Kind of neutral ledger entry.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryKind {
    /// User-authored model-visible message.
    UserMessage,
    /// Assistant-authored model-visible message.
    AssistantMessage,
    /// Tool result made visible to the model.
    ToolResult,
    /// Bounded summary made visible as context.
    Summary,
    /// Usage observation retained as safe metadata.
    UsageObservation,
    /// Safe receipt retained as metadata.
    SafeReceipt,
    /// Unsupported entry class that must fail closed.
    Unsupported,
    /// Adapter-owned payload marker that must not be replayed by the neutral core.
    AdapterOwned,
}

impl EntryKind {
    /// Returns a stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserMessage => "user_message",
            Self::AssistantMessage => "assistant_message",
            Self::ToolResult => "tool_result",
            Self::Summary => "summary",
            Self::UsageObservation => "usage_observation",
            Self::SafeReceipt => "safe_receipt",
            Self::Unsupported => "unsupported",
            Self::AdapterOwned => "adapter_owned",
        }
    }
}
