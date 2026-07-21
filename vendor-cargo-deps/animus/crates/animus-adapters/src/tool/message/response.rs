/// Response returned by a tool component.
#[allow(path_segment_repetition)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    result: crate::tool::BoundedOutput,
    redaction_class: super::OutputRedactionClass,
}

impl Response {
    /// Builds a runtime-visible tool response from an already bounded result.
    #[must_use]
    pub const fn from_bounded(
        result: crate::tool::BoundedOutput,
        redaction_class: super::OutputRedactionClass,
    ) -> Self {
        Self {
            result,
            redaction_class,
        }
    }

    /// Returns the bounded result metadata.
    #[must_use]
    pub const fn result(&self) -> &crate::tool::BoundedOutput {
        &self.result
    }

    /// Returns model-visible output text only.
    #[must_use]
    pub fn output(&self) -> &str {
        self.result.visible_text()
    }

    /// Returns model-visible output text only.
    #[must_use]
    pub fn model_visible_text(&self) -> &str {
        self.result.visible_text()
    }

    /// Returns the result completion or truncation status.
    #[must_use]
    pub const fn status(&self) -> crate::tool::OutputStatus {
        self.result.status()
    }

    /// Returns the original output byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.result.byte_count()
    }

    /// Returns the original output character count.
    #[must_use]
    pub const fn char_count(&self) -> u64 {
        self.result.char_count()
    }

    /// Returns the original output line count.
    #[must_use]
    pub const fn line_count(&self) -> u64 {
        self.result.line_count()
    }

    /// Returns the BLAKE3 content hash when policy requires it.
    #[must_use]
    pub fn content_hash(&self) -> Option<&str> {
        self.result.content_hash()
    }

    /// Returns the optional host-owned full-output handle.
    #[must_use]
    pub fn full_output_reference(&self) -> Option<&str> {
        self.result.full_output_reference()
    }

    /// Returns the output redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> super::OutputRedactionClass {
        self.redaction_class
    }

    /// Returns the stable tool response contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        crate::tool::TOOL_RESPONSE_CONTRACT_IDENTITY
    }
}
