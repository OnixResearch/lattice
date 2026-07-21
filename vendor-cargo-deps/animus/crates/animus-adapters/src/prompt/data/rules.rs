use super::forms::ContextReferenceKind;
use crate::prompt::base::{ALL_CONTEXT_KINDS, ALL_LOCAL_KINDS, ZERO_ALLOWED_KINDS, ZERO_COUNT};

/// Policy decision reached before resolver dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextReferencePolicyDecision {
    /// The active policy allowed resolver dispatch.
    Allowed,
    /// The active policy denied resolver dispatch.
    Denied,
    /// The reference kind is unsupported by policy.
    Unsupported,
}

impl ContextReferencePolicyDecision {
    /// Returns a stable policy decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Host resolver class used for context expansion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextResolverKind {
    /// Filesystem file resolver.
    File,
    /// Filesystem directory resolver.
    Directory,
    /// VCS diff resolver.
    Diff,
    /// Image resolver.
    Image,
    /// URL resolver.
    Url,
}

impl ContextResolverKind {
    /// Returns a stable resolver label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file_resolver",
            Self::Directory => "directory_resolver",
            Self::Diff => "diff_resolver",
            Self::Image => "image_resolver",
            Self::Url => "url_resolver",
        }
    }
}

/// Expansion status recorded for context-reference outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextReferenceStatus {
    /// The fragment was included without truncation.
    Included,
    /// The fragment was included after deterministic truncation.
    Truncated,
    /// The fragment was rejected by deterministic bounds.
    Rejected,
    /// The fragment was denied by policy.
    Denied,
    /// No required resolver was supplied.
    MissingResolver,
    /// The resolver reported an unavailable backend.
    Unavailable,
}

impl ContextReferenceStatus {
    /// Returns a stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::Truncated => "truncated",
            Self::Rejected => "rejected",
            Self::Denied => "denied",
            Self::MissingResolver => "missing_resolver",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Truncation state recorded without raw content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextTruncationState {
    /// No truncation was applied.
    None,
    /// Content was deterministically truncated before model-visible use.
    Truncated,
    /// Content was rejected instead of made model-visible.
    Rejected,
}

impl ContextTruncationState {
    /// Returns a stable truncation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Truncated => "truncated",
            Self::Rejected => "rejected",
        }
    }
}

/// Oversized-reference behavior selected by policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextOversizedPolicy {
    /// Truncate oversized content to the configured byte and line budgets.
    Truncate,
    /// Reject oversized content before it becomes model-visible.
    Reject,
}

/// Deterministic visible-context budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextReferenceBudget {
    pub(in crate::prompt) max_visible_bytes: u32,
    pub(in crate::prompt) max_visible_lines: u32,
    pub(in crate::prompt) oversized_policy: ContextOversizedPolicy,
}

impl ContextReferenceBudget {
    /// Builds a deterministic context-reference budget.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::ContextReferenceError`] when either visible budget is zero.
    pub const fn new(
        max_visible_bytes: u32,
        max_visible_lines: u32,
        oversized_policy: ContextOversizedPolicy,
    ) -> Result<Self, super::failure::ContextReferenceError> {
        if max_visible_bytes == ZERO_COUNT || max_visible_lines == ZERO_COUNT {
            return Err(super::failure::ContextReferenceError::InvalidBudget);
        }
        Ok(Self {
            max_visible_bytes,
            max_visible_lines,
            oversized_policy,
        })
    }

    /// Returns the maximum model-visible bytes.
    #[must_use]
    pub const fn max_visible_bytes(&self) -> u32 {
        self.max_visible_bytes
    }

    /// Returns the maximum model-visible lines.
    #[must_use]
    pub const fn max_visible_lines(&self) -> u32 {
        self.max_visible_lines
    }

    /// Returns the oversized-reference policy.
    #[must_use]
    pub const fn oversized_policy(&self) -> ContextOversizedPolicy {
        self.oversized_policy
    }
}

/// Context-reference admission policy before resolver dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextReferencePolicy {
    allowed: u8,
}

impl ContextReferencePolicy {
    /// Builds a policy that denies every context-reference kind.
    #[must_use]
    pub const fn deny_all() -> Self {
        Self {
            allowed: ZERO_ALLOWED_KINDS,
        }
    }

    /// Builds a policy that allows file, directory, diff, and image resolvers.
    #[must_use]
    pub const fn allow_local() -> Self {
        Self {
            allowed: ALL_LOCAL_KINDS,
        }
    }

    /// Builds a policy that allows every currently known resolver kind.
    #[must_use]
    pub const fn allow_all() -> Self {
        Self {
            allowed: ALL_CONTEXT_KINDS,
        }
    }

    /// Returns a copy with the supplied kind allowed.
    #[must_use]
    pub const fn with_kind(mut self, kind: ContextReferenceKind) -> Self {
        self.allowed |= kind.policy_bit();
        self
    }

    /// Returns true when the supplied kind is allowed by policy.
    #[must_use]
    pub const fn allows(self, kind: ContextReferenceKind) -> bool {
        self.allowed & kind.policy_bit() != ZERO_ALLOWED_KINDS
    }
}
