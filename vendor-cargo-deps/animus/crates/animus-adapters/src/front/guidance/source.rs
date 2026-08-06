use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use super::GUIDANCE_NAME_FIELD;
use super::GUIDANCE_ORIGIN_FIELD;
use super::GuidanceError;
use super::PromptResourceScope;
use super::normalize;

/// Ordering rank for layered guidance sources.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GuidancePrecedence(u16);

impl GuidancePrecedence {
    /// Builds a precedence rank where lower values are assembled earlier.
    #[must_use]
    pub const fn new(rank: u16) -> Self {
        Self(rank)
    }

    /// Returns the stable numeric rank.
    #[must_use]
    pub const fn rank(self) -> u16 {
        self.0
    }
}

/// Host-declared override behavior for a guidance source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidanceOverride {
    /// Source extends broader guidance without deleting prior entries.
    Extend,
    /// Source replaces matching broader guidance by host policy.
    Replace,
}

/// Borrowed layered guidance source fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuidanceSourceDraft<'a> {
    /// Stable model-visible source name.
    pub name: &'a str,
    /// Scope where this source applies.
    pub scope: PromptResourceScope,
    /// Precedence rank; lower ranks are assembled first.
    pub precedence: GuidancePrecedence,
    /// Override semantics declared by the host.
    pub override_semantics: GuidanceOverride,
    /// Safe source origin label.
    pub origin: &'a str,
    /// Lowercase BLAKE3 content hash for the source body.
    pub content_hash: &'a str,
    /// Prompt-budget units consumed by the source if fully included.
    pub budget_units: u32,
    /// Whether host policy redacted this source before model context assembly.
    pub redacted: bool,
}

/// Validated layered guidance source with attribution and budget metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuidanceSource {
    pub(super) name: String,
    pub(super) scope: PromptResourceScope,
    pub(super) precedence: GuidancePrecedence,
    pub(super) override_semantics: GuidanceOverride,
    pub(super) origin: String,
    pub(super) content_hash: crate::artifact::Hash,
    pub(super) budget_units: NonZeroU32,
    pub(super) redacted: bool,
}

impl GuidanceSource {
    /// Validates host-supplied guidance source metadata.
    ///
    /// # Errors
    ///
    /// Returns [`GuidanceError`] when source text is empty or unsafe, the hash is
    /// not lowercase BLAKE3 hex, or the budget cost is zero.
    pub fn validate(draft: &GuidanceSourceDraft<'_>) -> Result<Self, GuidanceError> {
        let name = normalize(GUIDANCE_NAME_FIELD, draft.name)?;
        let origin = normalize(GUIDANCE_ORIGIN_FIELD, draft.origin)?;
        let content_hash = crate::artifact::Hash::new(draft.content_hash).map_err(GuidanceError::InvalidHash)?;
        let budget_units = NonZeroU32::new(draft.budget_units)
            .ok_or_else(|| GuidanceError::ZeroBudgetCost { source: name.clone() })?;
        Ok(Self {
            name,
            scope: draft.scope,
            precedence: draft.precedence,
            override_semantics: draft.override_semantics,
            origin,
            content_hash,
            budget_units,
            redacted: draft.redacted,
        })
    }

    /// Returns the stable source name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn scope(&self) -> PromptResourceScope {
        self.scope
    }

    /// Returns the precedence rank.
    #[must_use]
    pub const fn precedence(&self) -> GuidancePrecedence {
        self.precedence
    }

    /// Returns the declared override semantics.
    #[must_use]
    pub const fn override_semantics(&self) -> GuidanceOverride {
        self.override_semantics
    }

    /// Returns the safe source origin label.
    #[must_use]
    pub fn origin(&self) -> &str {
        self.origin.as_str()
    }

    /// Returns the lowercase BLAKE3 content hash.
    #[must_use]
    pub const fn content_hash(&self) -> &crate::artifact::Hash {
        &self.content_hash
    }

    /// Returns the full-inclusion prompt budget cost.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }

    /// Returns true when host policy redacted this source.
    #[must_use]
    pub const fn is_redacted(&self) -> bool {
        self.redacted
    }
}

/// Prompt budget available for layered guidance assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuidanceBudget {
    max_units: NonZeroU32,
}

impl GuidanceBudget {
    /// Builds a guidance prompt budget.
    #[must_use]
    pub const fn new(max_units: NonZeroU32) -> Self {
        Self { max_units }
    }

    /// Returns the maximum prompt budget units.
    #[must_use]
    pub const fn max_units(self) -> NonZeroU32 {
        self.max_units
    }
}

/// Ledger disposition for one guidance source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidanceDisposition {
    /// Source was fully included in prompt context.
    Included,
    /// Source was skipped before consuming prompt budget.
    Skipped,
    /// Source was partially included and truncation was recorded.
    Truncated,
    /// Source was redacted before prompt context assembly.
    Redacted,
}

/// Reason attached to a non-fully-included guidance ledger entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidanceLedgerReason {
    /// Source exceeded the remaining prompt budget.
    OverBudget,
    /// Host policy redacted the source.
    RedactedByPolicy,
}

/// Prompt assembly ledger entry for one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuidanceLedgerEntry {
    pub(super) source_name: String,
    pub(super) source_origin: String,
    pub(super) content_hash: crate::artifact::Hash,
    pub(super) scope: PromptResourceScope,
    pub(super) precedence: GuidancePrecedence,
    pub(super) disposition: GuidanceDisposition,
    pub(super) requested_units: NonZeroU32,
    pub(super) consumed_units: u32,
    pub(super) reason: Option<GuidanceLedgerReason>,
}

impl GuidanceLedgerEntry {
    /// Returns the source name.
    #[must_use]
    pub fn source_name(&self) -> &str {
        self.source_name.as_str()
    }

    /// Returns the safe origin label.
    #[must_use]
    pub fn source_origin(&self) -> &str {
        self.source_origin.as_str()
    }

    /// Returns the source content hash.
    #[must_use]
    pub const fn content_hash(&self) -> &crate::artifact::Hash {
        &self.content_hash
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn scope(&self) -> PromptResourceScope {
        self.scope
    }

    /// Returns the source precedence rank.
    #[must_use]
    pub const fn precedence(&self) -> GuidancePrecedence {
        self.precedence
    }

    /// Returns the ledger disposition.
    #[must_use]
    pub const fn disposition(&self) -> GuidanceDisposition {
        self.disposition
    }

    /// Returns budget units requested by the source.
    #[must_use]
    pub const fn requested_units(&self) -> NonZeroU32 {
        self.requested_units
    }

    /// Returns budget units consumed by the source.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns the non-inclusion reason, when present.
    #[must_use]
    pub const fn reason(&self) -> Option<GuidanceLedgerReason> {
        self.reason
    }
}

/// Source-attributed prompt guidance assembly result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuidanceAssembly {
    pub(super) ledger: Vec<GuidanceLedgerEntry>,
}

impl GuidanceAssembly {
    /// Returns ordered source attribution and budget decisions.
    #[must_use]
    pub fn ledger(&self) -> &[GuidanceLedgerEntry] {
        self.ledger.as_slice()
    }
}
