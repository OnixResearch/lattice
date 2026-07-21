use super::forms::ContextReference;
use super::rules::{
    ContextReferencePolicyDecision, ContextReferenceStatus, ContextResolverKind,
    ContextTruncationState,
};

/// Candidate fragment returned by an injected host resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedContextCandidate {
    pub(in crate::prompt) reference: ContextReference,
    pub(in crate::prompt) resolver_kind: ContextResolverKind,
    pub(in crate::prompt) content: alloc::string::String,
}

impl ResolvedContextCandidate {
    /// Builds a resolver candidate without inspecting the host-owned content.
    #[must_use]
    pub fn new(
        reference: ContextReference,
        resolver_kind: ContextResolverKind,
        content: impl Into<alloc::string::String>,
    ) -> Self {
        Self {
            reference,
            resolver_kind,
            content: content.into(),
        }
    }

    /// Returns the reference that produced this candidate.
    #[must_use]
    pub const fn reference(&self) -> &ContextReference {
        &self.reference
    }

    /// Returns the resolver class that produced this candidate.
    #[must_use]
    pub const fn resolver_kind(&self) -> ContextResolverKind {
        self.resolver_kind
    }

    /// Returns the host-supplied candidate content before budgeting.
    #[must_use]
    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

/// Budgeted model-visible context fragment plus safe metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextReferenceFragment {
    pub(in crate::prompt) reference: ContextReference,
    pub(in crate::prompt) resolver_kind: ContextResolverKind,
    pub(in crate::prompt) status: ContextReferenceStatus,
    pub(in crate::prompt) truncation: ContextTruncationState,
    pub(in crate::prompt) visible_text: Option<alloc::string::String>,
    pub(in crate::prompt) byte_count: u32,
    pub(in crate::prompt) line_count: u32,
    pub(in crate::prompt) visible_byte_count: u32,
    pub(in crate::prompt) visible_line_count: u32,
    pub(in crate::prompt) content_hash: crate::ArtifactHash,
}

impl ContextReferenceFragment {
    /// Returns the source reference.
    #[must_use]
    pub const fn reference(&self) -> &ContextReference {
        &self.reference
    }

    /// Returns the resolver class.
    #[must_use]
    pub const fn resolver_kind(&self) -> ContextResolverKind {
        self.resolver_kind
    }

    /// Returns the budget outcome status.
    #[must_use]
    pub const fn status(&self) -> ContextReferenceStatus {
        self.status
    }

    /// Returns the truncation state.
    #[must_use]
    pub const fn truncation(&self) -> ContextTruncationState {
        self.truncation
    }

    /// Returns model-visible text when included.
    #[must_use]
    pub fn visible_text(&self) -> Option<&str> {
        self.visible_text.as_deref()
    }

    /// Returns original content byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u32 {
        self.byte_count
    }

    /// Returns original content line count.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.line_count
    }

    /// Returns model-visible byte count.
    #[must_use]
    pub const fn visible_byte_count(&self) -> u32 {
        self.visible_byte_count
    }

    /// Returns model-visible line count.
    #[must_use]
    pub const fn visible_line_count(&self) -> u32 {
        self.visible_line_count
    }

    /// Returns the BLAKE3 content identity for the original candidate.
    #[must_use]
    pub const fn content_hash(&self) -> &crate::ArtifactHash {
        &self.content_hash
    }
}

/// Resolver plan produced before host dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextResolutionPlan {
    pub(in crate::prompt) resolver_kind: ContextResolverKind,
    pub(in crate::prompt) decision: ContextReferencePolicyDecision,
}

impl ContextResolutionPlan {
    /// Returns the resolver class selected by policy.
    #[must_use]
    pub const fn resolver_kind(self) -> ContextResolverKind {
        self.resolver_kind
    }

    /// Returns the policy decision.
    #[must_use]
    pub const fn decision(self) -> ContextReferencePolicyDecision {
        self.decision
    }
}
