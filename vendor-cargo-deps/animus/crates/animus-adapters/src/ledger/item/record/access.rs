impl super::Entry {
    /// Returns the entry identifier.
    #[must_use]
    pub const fn id(&self) -> &super::super::EntryId {
        match self {
            Self::UserMessage { id, .. }
            | Self::AssistantMessage { id, .. }
            | Self::ToolResult { id, .. }
            | Self::Summary { id, .. }
            | Self::UsageObservation { id, .. }
            | Self::SafeReceipt { id, .. }
            | Self::Unsupported { id, .. }
            | Self::AdapterOwned { id, .. } => id,
        }
    }

    /// Returns the entry kind.
    #[must_use]
    pub const fn kind(&self) -> super::super::EntryKind {
        match self {
            Self::UserMessage { .. } => super::super::EntryKind::UserMessage,
            Self::AssistantMessage { .. } => super::super::EntryKind::AssistantMessage,
            Self::ToolResult { .. } => super::super::EntryKind::ToolResult,
            Self::Summary { .. } => super::super::EntryKind::Summary,
            Self::UsageObservation { .. } => super::super::EntryKind::UsageObservation,
            Self::SafeReceipt { .. } => super::super::EntryKind::SafeReceipt,
            Self::Unsupported { .. } => super::super::EntryKind::Unsupported,
            Self::AdapterOwned { .. } => super::super::EntryKind::AdapterOwned,
        }
    }

    /// Returns summary semantics when this entry is a summary.
    #[must_use]
    pub const fn summary_kind(&self) -> Option<super::super::SummaryKind> {
        match self {
            Self::Summary { summary_kind, .. } => Some(*summary_kind),
            Self::UserMessage { .. }
            | Self::AssistantMessage { .. }
            | Self::ToolResult { .. }
            | Self::UsageObservation { .. }
            | Self::SafeReceipt { .. }
            | Self::Unsupported { .. }
            | Self::AdapterOwned { .. } => None,
        }
    }
}
