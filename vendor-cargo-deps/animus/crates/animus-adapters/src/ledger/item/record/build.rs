impl super::Entry {
    /// Builds a user message entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn user(id: &str, content: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::UserMessage {
            id: super::super::EntryId::new(id)?,
            content: super::super::normalized_text(content)?,
        })
    }

    /// Builds an assistant message entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn assistant(id: &str, content: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::AssistantMessage {
            id: super::super::EntryId::new(id)?,
            content: super::super::normalized_text(content)?,
        })
    }

    /// Builds a tool result entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn tool(
        id: &str,
        tool_label: &str,
        content: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::ToolResult {
            id: super::super::EntryId::new(id)?,
            tool_label: super::super::normalized_label(tool_label)?,
            content: super::super::normalized_text(content)?,
        })
    }

    /// Builds a summary entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn summary(id: &str, content: &str) -> Result<Self, crate::AdapterContractError> {
        Self::summary_with_kind(id, super::super::SummaryKind::Compaction, content)
    }

    /// Builds a branch handoff summary entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn branch_summary(id: &str, content: &str) -> Result<Self, crate::AdapterContractError> {
        Self::summary_with_kind(id, super::super::SummaryKind::BranchHandoff, content)
    }

    /// Builds a summary entry with explicit summary semantics.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn summary_with_kind(
        id: &str,
        summary_kind: super::super::SummaryKind,
        content: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::Summary {
            id: super::super::EntryId::new(id)?,
            summary_kind,
            content: super::super::normalized_text(content)?,
        })
    }

    /// Builds a usage observation entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the identifier is invalid.
    pub fn usage(
        id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::UsageObservation {
            id: super::super::EntryId::new(id)?,
            input_tokens,
            output_tokens,
        })
    }

    /// Builds a safe receipt entry.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the identifier is invalid.
    pub fn safe_receipt(
        id: &str,
        receipt: crate::SafeReceipt,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::SafeReceipt {
            id: super::super::EntryId::new(id)?,
            receipt,
        })
    }

    /// Builds an unsupported entry marker without retaining raw payload.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn unsupported(id: &str, kind_label: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::Unsupported {
            id: super::super::EntryId::new(id)?,
            kind_label: super::super::normalized_label(kind_label)?,
        })
    }

    /// Builds an adapter-owned marker without retaining raw payload.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when an entry field is invalid.
    pub fn adapter_owned(id: &str, kind_label: &str) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::AdapterOwned {
            id: super::super::EntryId::new(id)?,
            kind_label: super::super::normalized_label(kind_label)?,
        })
    }
}
