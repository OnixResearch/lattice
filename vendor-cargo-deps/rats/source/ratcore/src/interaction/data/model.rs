//! Prompt session values and immutable supplied snapshots.

const MAX_ID_BYTES: usize = 128;
const MAX_TEXT_BYTES: usize = 4096;

macro_rules! prompt_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);
        impl $name {
            /// Creates an identity for later bounded validation.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
            /// Returns the stable textual identity.
            #[must_use]
            pub fn as_str(&self) -> &str { self.0.as_str() }
            pub(crate) fn is_valid(&self) -> bool {
                !self.0.trim().is_empty() && self.0.len() <= MAX_ID_BYTES
            }
        }
    };
}

prompt_id! {
    /// Stable identity of one prompt instance.
    PromptSessionId
}
prompt_id! {
    /// Stable identity of one completion candidate.
    PromptCandidateId
}
prompt_id! {
    /// Stable namespace of caller-owned prompt history.
    PromptHistoryNamespace
}

/// Monotonic revision of committed input text.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PromptInputRevision(pub u64);

/// Monotonic generation of a supplied snapshot.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PromptGeneration(pub u64);

/// Candidate filtering and ordering policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCompletionPolicy {
    /// Do not expose completion candidates.
    None,
    /// Preserve caller order without query filtering.
    CallerOrder,
    /// Match the complete candidate text case-insensitively.
    Exact,
    /// Match a case-insensitive candidate prefix.
    Prefix,
    /// Use the existing deterministic Rats fuzzy scorer.
    Fuzzy,
}

/// Behavior at the first and last eligible candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMovementPolicy {
    /// Continue from the opposite boundary.
    Wrap,
    /// Stay at the boundary.
    Clamp,
    /// Preserve state and report the boundary.
    Reject,
}

/// Caller-selected history and privacy policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptHistoryPolicy {
    /// History is unavailable.
    Disabled,
    /// Supplied, non-sensitive history may be navigated.
    Enabled,
    /// Values are sensitive and must not be retained or projected as evidence.
    Sensitive,
}

/// Value accepted by submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptAcceptancePolicy {
    /// Submit the active supplied candidate.
    Candidate,
    /// Submit committed free text.
    FreeText,
}

/// Bounded prompt behavior selected by a domain adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptConfiguration {
    /// Candidate matching policy.
    pub completion: PromptCompletionPolicy,
    /// Candidate boundary behavior.
    pub movement: PromptMovementPolicy,
    /// Supplied-history policy.
    pub history: PromptHistoryPolicy,
    /// Submission policy.
    pub acceptance: PromptAcceptancePolicy,
    /// Whether empty free text can be accepted.
    pub allow_empty: bool,
    /// Whether free text is trimmed before acceptance.
    pub trim_on_submit: bool,
    /// Maximum accepted candidates, bounded by the implementation limit.
    pub max_candidates: u16,
}

/// Availability of a supplied completion catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCompletionStatus {
    /// The caller has supplied its complete current catalog.
    Complete,
    /// The caller reports that external completion work remains pending.
    Pending,
}

/// One payload-free completion candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptCandidate {
    /// Stable candidate identity.
    pub id: PromptCandidateId,
    /// Visible candidate label.
    pub label: String,
    /// Optional visible detail.
    pub detail: Option<String>,
    /// Optional visible group.
    pub group: Option<String>,
    /// Optional explicit matching text.
    pub match_text: Option<String>,
    /// Whether activation is currently eligible.
    pub enabled: bool,
    /// Required reason when disabled and forbidden when enabled.
    pub disabled_reason: Option<String>,
    /// Deterministic caller order.
    pub source_order: u32,
}

impl PromptCandidate {
    /// Creates an enabled candidate.
    #[must_use]
    pub fn enabled(id: impl Into<String>, label: impl Into<String>, source_order: u32) -> Self {
        Self {
            id: PromptCandidateId::new(id),
            label: label.into(),
            detail: None,
            group: None,
            match_text: None,
            enabled: true,
            disabled_reason: None,
            source_order,
        }
    }
    /// Creates a disabled candidate with an explicit reason.
    #[must_use]
    pub fn disabled(
        id: impl Into<String>,
        label: impl Into<String>,
        reason: impl Into<String>,
        source_order: u32,
    ) -> Self {
        Self {
            id: PromptCandidateId::new(id),
            label: label.into(),
            detail: None,
            group: None,
            match_text: None,
            enabled: false,
            disabled_reason: Some(reason.into()),
            source_order,
        }
    }
    pub(crate) fn matching_text(&self) -> &str {
        self.match_text.as_deref().unwrap_or(self.label.as_str())
    }
    pub(crate) fn metadata_is_bounded(&self) -> bool {
        !self.label.trim().is_empty()
            && self.label.len() <= MAX_TEXT_BYTES
            && self.detail.as_ref().is_none_or(|value| value.len() <= MAX_TEXT_BYTES)
            && self.group.as_ref().is_none_or(|value| value.len() <= MAX_ID_BYTES)
            && self.match_text.as_ref().is_none_or(|value| value.len() <= MAX_TEXT_BYTES)
    }
}

/// Immutable caller-supplied completion catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptCompletionSnapshot {
    /// Monotonic catalog generation.
    pub generation: PromptGeneration,
    /// Whether external catalog construction is complete.
    pub status: PromptCompletionStatus,
    /// Bounded payload-free candidates.
    pub candidates: Vec<PromptCandidate>,
}

/// Immutable caller-supplied history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptHistorySnapshot {
    /// Stable privacy/persistence namespace.
    pub namespace: PromptHistoryNamespace,
    /// Monotonic history generation.
    pub generation: PromptGeneration,
    /// Oldest-to-newest caller-owned entries.
    pub entries: Vec<String>,
}

/// Caller-supplied validation state tied to committed input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptValidationState {
    /// Input is accepted by the caller's validator.
    Accepted,
    /// Validation is not complete.
    Pending,
    /// Input is rejected with a bounded safe reason.
    Rejected(String),
}

/// Immutable validation observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptValidationSnapshot {
    /// Input revision observed by the caller.
    pub input_revision: PromptInputRevision,
    /// Validation observation.
    pub state: PromptValidationState,
}

pub(crate) fn text_is_bounded(value: &str) -> bool {
    value.len() <= MAX_TEXT_BYTES
}
