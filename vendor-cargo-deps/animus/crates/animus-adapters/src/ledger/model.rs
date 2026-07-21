/// Model-visible replay role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContextRole {
    /// User message role.
    User,
    /// Assistant message role.
    Assistant,
    /// Tool result role.
    Tool,
    /// System summary role.
    System,
}

impl ContextRole {
    /// Returns a stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
            Self::System => "system",
        }
    }
}

/// Model-visible context item produced by replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextItem {
    role: ContextRole,
    content: alloc::string::String,
    source_id: super::EntryId,
}

impl ContextItem {
    pub(super) fn new(
        role: ContextRole,
        content: alloc::string::String,
        source_id: super::EntryId,
    ) -> Self {
        Self {
            role,
            content,
            source_id,
        }
    }

    /// Returns the context role.
    #[must_use]
    pub const fn role(&self) -> ContextRole {
        self.role
    }

    /// Returns the model-visible content.
    #[must_use]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    /// Returns the source ledger entry identifier.
    #[must_use]
    pub const fn source_id(&self) -> &super::EntryId {
        &self.source_id
    }
}
