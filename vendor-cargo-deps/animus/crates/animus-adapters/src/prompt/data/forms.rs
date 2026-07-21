use crate::prompt::base::{
    KIND_DIFF_BIT, KIND_DIRECTORY_BIT, KIND_FILE_BIT, KIND_IMAGE_BIT, KIND_URL_BIT,
    MALFORMED_RANGE_REASON, MIN_LINE_NUMBER,
};
use alloc::string::ToString;

/// Kind of prompt context reference requested by a turn.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextReferenceKind {
    /// Local file text supplied by a host resolver.
    File,
    /// Directory listing supplied by a host resolver.
    Directory,
    /// Diff text supplied by a VCS-aware host resolver.
    Diff,
    /// Image descriptor or bytes supplied by a host resolver.
    Image,
    /// URL content supplied by a policy-enabled network resolver.
    Url,
}

impl ContextReferenceKind {
    /// Returns a stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Diff => "diff",
            Self::Image => "image",
            Self::Url => "url",
        }
    }

    pub(in crate::prompt) const fn policy_bit(self) -> u8 {
        match self {
            Self::File => KIND_FILE_BIT,
            Self::Directory => KIND_DIRECTORY_BIT,
            Self::Diff => KIND_DIFF_BIT,
            Self::Image => KIND_IMAGE_BIT,
            Self::Url => KIND_URL_BIT,
        }
    }
}

/// Requested model-visible rendering mode for a context reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRenderMode {
    /// Render as bounded text.
    Text,
    /// Render as a bounded directory listing.
    DirectoryListing,
    /// Render as a bounded diff.
    Diff,
    /// Render as an image payload descriptor.
    Image,
}

impl ContextRenderMode {
    /// Returns a stable render-mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::DirectoryListing => "directory_listing",
            Self::Diff => "diff",
            Self::Image => "image",
        }
    }
}

/// Trust class attached to the source of a raw context-reference label.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextSourceTrust {
    /// The reference came directly from a user prompt or request.
    UserSupplied,
    /// The reference was proposed by an agent.
    AgentSupplied,
    /// The reference was supplied by a host integration.
    HostSupplied,
    /// The reference was generated from checked policy.
    PolicyGenerated,
}

impl ContextSourceTrust {
    /// Returns a stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserSupplied => "user_supplied",
            Self::AgentSupplied => "agent_supplied",
            Self::HostSupplied => "host_supplied",
            Self::PolicyGenerated => "policy_generated",
        }
    }
}

/// Optional inclusive one-based line range requested for a text reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextLineRange {
    start_line: u32,
    end_line: Option<u32>,
}

impl ContextLineRange {
    /// Builds an inclusive one-based range.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::ContextReferenceError`] when the start is zero or the end is
    /// lower than the start.
    pub const fn new(
        start_line: u32,
        end_line: Option<u32>,
    ) -> Result<Self, super::failure::ContextReferenceError> {
        if start_line < MIN_LINE_NUMBER {
            return Err(super::failure::ContextReferenceError::MalformedRange {
                reason: MALFORMED_RANGE_REASON,
            });
        }
        if let Some(end) = end_line {
            if end < start_line {
                return Err(super::failure::ContextReferenceError::MalformedRange {
                    reason: MALFORMED_RANGE_REASON,
                });
            }
        }
        Ok(Self {
            start_line,
            end_line,
        })
    }

    /// Returns the inclusive one-based start line.
    #[must_use]
    pub const fn start_line(&self) -> u32 {
        self.start_line
    }

    /// Returns the inclusive one-based end line when bounded.
    #[must_use]
    pub const fn end_line(&self) -> Option<u32> {
        self.end_line
    }
}

/// Parsed and normalized context reference before resolver dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextReference {
    pub(in crate::prompt) raw_label: alloc::string::String,
    pub(in crate::prompt) normalized_target: alloc::string::String,
    pub(in crate::prompt) safe_target_label: alloc::string::String,
    pub(in crate::prompt) kind: ContextReferenceKind,
    pub(in crate::prompt) range: Option<ContextLineRange>,
    pub(in crate::prompt) render_mode: ContextRenderMode,
    pub(in crate::prompt) source_trust: ContextSourceTrust,
}

impl ContextReference {
    pub(in crate::prompt) fn new(input: ContextReferenceInput<'_>) -> Self {
        Self {
            raw_label: input.raw_label.to_string(),
            normalized_target: input.normalized_target,
            safe_target_label: input.safe_target_label,
            kind: input.kind,
            range: input.range,
            render_mode: input.render_mode,
            source_trust: input.source_trust,
        }
    }

    /// Returns the raw label text without granting authority.
    #[must_use]
    pub fn raw_label(&self) -> &str {
        self.raw_label.as_str()
    }

    /// Returns the normalized target label consumed by policy and resolvers.
    #[must_use]
    pub fn normalized_target(&self) -> &str {
        self.normalized_target.as_str()
    }

    /// Returns a safe target label suitable for receipts and ledgers.
    #[must_use]
    pub fn safe_target_label(&self) -> &str {
        self.safe_target_label.as_str()
    }

    /// Returns the context reference kind.
    #[must_use]
    pub const fn kind(&self) -> ContextReferenceKind {
        self.kind
    }

    /// Returns the optional requested line range.
    #[must_use]
    pub const fn range(&self) -> Option<ContextLineRange> {
        self.range
    }

    /// Returns the requested render mode.
    #[must_use]
    pub const fn render_mode(&self) -> ContextRenderMode {
        self.render_mode
    }

    /// Returns source trust metadata for the raw label.
    #[must_use]
    pub const fn source_trust(&self) -> ContextSourceTrust {
        self.source_trust
    }
}

pub(in crate::prompt) struct ContextReferenceInput<'a> {
    pub(in crate::prompt) raw_label: &'a str,
    pub(in crate::prompt) normalized_target: alloc::string::String,
    pub(in crate::prompt) safe_target_label: alloc::string::String,
    pub(in crate::prompt) kind: ContextReferenceKind,
    pub(in crate::prompt) range: Option<ContextLineRange>,
    pub(in crate::prompt) render_mode: ContextRenderMode,
    pub(in crate::prompt) source_trust: ContextSourceTrust,
}
