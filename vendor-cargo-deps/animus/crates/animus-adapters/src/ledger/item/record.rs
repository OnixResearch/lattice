mod access;
mod build;

/// Neutral ledger entry owned by Animus adapter boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Entry {
    /// User-authored model-visible message.
    UserMessage {
        /// Entry identifier.
        id: super::EntryId,
        /// Model-visible content.
        content: alloc::string::String,
    },
    /// Assistant-authored model-visible message.
    AssistantMessage {
        /// Entry identifier.
        id: super::EntryId,
        /// Model-visible content.
        content: alloc::string::String,
    },
    /// Tool result made visible to the model.
    ToolResult {
        /// Entry identifier.
        id: super::EntryId,
        /// Safe tool label.
        tool_label: alloc::string::String,
        /// Model-visible tool content.
        content: alloc::string::String,
    },
    /// Bounded summary made visible as context.
    Summary {
        /// Entry identifier.
        id: super::EntryId,
        /// Summary semantics.
        summary_kind: super::SummaryKind,
        /// Summary content.
        content: alloc::string::String,
    },
    /// Usage observation retained as metadata counts.
    UsageObservation {
        /// Entry identifier.
        id: super::EntryId,
        /// Observed input token count.
        input_tokens: u32,
        /// Observed output token count.
        output_tokens: u32,
    },
    /// Safe receipt retained as replay metadata.
    SafeReceipt {
        /// Entry identifier.
        id: super::EntryId,
        /// Safe receipt payload.
        receipt: crate::SafeReceipt,
    },
    /// Unsupported entry that must stop replay.
    Unsupported {
        /// Entry identifier.
        id: super::EntryId,
        /// Safe unsupported kind label.
        kind_label: alloc::string::String,
    },
    /// Adapter-owned payload marker that must stop neutral replay.
    AdapterOwned {
        /// Entry identifier.
        id: super::EntryId,
        /// Safe adapter-owned kind label.
        kind_label: alloc::string::String,
    },
}
