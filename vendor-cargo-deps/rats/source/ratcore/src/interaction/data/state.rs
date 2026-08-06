//! Accepted prompt state, messages, outputs, and diagnostics.

/// Bounded lifecycle of one prompt instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptLifecycle {
    /// Prompt interaction is active.
    Open,
    /// A submit or cancel output awaits owner acknowledgement.
    ClosePending,
    /// Prompt interaction is inactive.
    Closed,
}

/// Candidate movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDirection {
    /// Move toward the preceding eligible item.
    Previous,
    /// Move toward the following eligible item.
    Next,
}

/// History movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptHistoryDirection {
    /// Move toward an older entry.
    Older,
    /// Move toward a newer entry or the preserved draft.
    Newer,
}

/// Stable prompt diagnostic vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDiagnosticCode {
    /// Session identity or prompt metadata is malformed.
    MalformedSession,
    /// Configuration combines incompatible policies or excessive limits.
    InvalidConfiguration,
    /// Input text or revision is malformed or stale.
    InvalidInput,
    /// Completion snapshot generation is stale.
    StaleCandidates,
    /// Completion snapshot content is malformed or ambiguous.
    InvalidCandidates,
    /// Requested candidate does not exist or is filtered.
    UnknownCandidate,
    /// Requested candidate is disabled.
    DisabledCandidate,
    /// No eligible candidate is available.
    NoEligibleCandidate,
    /// Movement reached a rejecting boundary.
    MovementBoundary,
    /// History is disabled, sensitive, stale, or malformed.
    HistoryUnavailable,
    /// Validation observation is stale or malformed.
    InvalidValidation,
    /// Validation remains pending.
    ValidationPending,
    /// Caller-supplied validation rejected input.
    ValidationRejected,
    /// Empty free text is forbidden.
    EmptyInput,
    /// Submission observed stale input or candidates.
    StaleSubmission,
    /// Message is impossible in the current lifecycle.
    InvalidLifecycle,
}

/// Content-bounded deterministic diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDiagnostic {
    /// Stable machine-readable code.
    pub code: PromptDiagnosticCode,
    /// Stable content-free explanation.
    pub message: &'static str,
}

impl PromptDiagnostic {
    pub(crate) const fn new(code: PromptDiagnosticCode, message: &'static str) -> Self {
        Self { code, message }
    }
}

/// Typed accepted prompt result. It never authorizes a domain effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOutput {
    /// Accepted payload-free candidate identity and visible value.
    Candidate {
        /// Stable candidate identity.
        id: super::model::PromptCandidateId,
        /// Visible candidate label.
        value: String,
        /// Catalog generation observed at acceptance.
        candidate_generation: super::model::PromptGeneration,
        /// Input revision observed at acceptance.
        input_revision: super::model::PromptInputRevision,
    },
    /// Accepted free text.
    FreeText {
        /// Accepted committed text after configured trimming.
        value: String,
        /// Input revision observed at acceptance.
        input_revision: super::model::PromptInputRevision,
    },
    /// Cancellation request for this session.
    Cancelled {
        /// Stable session identity.
        session_id: super::model::PromptSessionId,
    },
}

/// Renderer-neutral prompt session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSession {
    pub(crate) id: super::model::PromptSessionId,
    pub(crate) label: String,
    pub(crate) help: Option<String>,
    pub(crate) config: super::model::PromptConfiguration,
    pub(crate) lifecycle: PromptLifecycle,
    pub(crate) input: String,
    pub(crate) input_revision: super::model::PromptInputRevision,
    pub(crate) completion: Option<super::model::PromptCompletionSnapshot>,
    pub(crate) active: Option<super::model::PromptCandidateId>,
    pub(crate) history: Option<super::model::PromptHistorySnapshot>,
    pub(crate) history_position: Option<usize>,
    pub(crate) history_draft: Option<String>,
    pub(crate) validation: Option<super::model::PromptValidationSnapshot>,
}

impl PromptSession {
    /// Returns the stable session identity.
    #[must_use]
    pub fn id(&self) -> &super::model::PromptSessionId {
        &self.id
    }
    /// Returns visible prompt label metadata.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
    /// Returns optional visible help metadata.
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }
    /// Returns the accepted configuration.
    #[must_use]
    pub fn configuration(&self) -> &super::model::PromptConfiguration {
        &self.config
    }
    /// Returns current lifecycle.
    #[must_use]
    pub fn lifecycle(&self) -> PromptLifecycle {
        self.lifecycle
    }
    /// Returns committed renderer-supplied input.
    #[must_use]
    pub fn input(&self) -> &str {
        self.input.as_str()
    }
    /// Returns current input revision.
    #[must_use]
    pub fn input_revision(&self) -> super::model::PromptInputRevision {
        self.input_revision
    }
    /// Returns current completion snapshot.
    #[must_use]
    pub fn completion(&self) -> Option<&super::model::PromptCompletionSnapshot> {
        self.completion.as_ref()
    }
    /// Returns current active candidate identity.
    #[must_use]
    pub fn active_candidate(&self) -> Option<&super::model::PromptCandidateId> {
        self.active.as_ref()
    }
    /// Returns supplied history without exposing sensitive entries when policy forbids it.
    #[must_use]
    pub fn history(&self) -> Option<&super::model::PromptHistorySnapshot> {
        (self.config.history == super::model::PromptHistoryPolicy::Enabled)
            .then_some(self.history.as_ref())
            .flatten()
    }
    /// Returns current validation observation.
    #[must_use]
    pub fn validation(&self) -> Option<&super::model::PromptValidationSnapshot> {
        self.validation.as_ref()
    }
    /// Returns whether this session accepts candidates.
    #[must_use]
    pub fn accepts_candidates(&self) -> bool {
        self.config.acceptance == super::model::PromptAcceptancePolicy::Candidate
    }
}

/// Pure prompt input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMessage {
    /// Reset and open with explicit committed input and snapshots.
    Reset {
        /// Explicit committed input for the reopened session.
        input: String,
        /// Optional initial immutable catalog.
        completion: Option<super::model::PromptCompletionSnapshot>,
    },
    /// Replace committed text if the expected revision is current.
    ReplaceInput {
        /// Newly committed renderer-owned text.
        input: String,
        /// Revision the renderer observed before replacement.
        expected_revision: super::model::PromptInputRevision,
    },
    /// Replace the immutable completion catalog.
    ReplaceCandidates(super::model::PromptCompletionSnapshot),
    /// Move among eligible filtered candidates.
    Move(PromptDirection),
    /// Request one candidate by stable identity.
    SetActive(super::model::PromptCandidateId),
    /// Replace immutable history.
    ReplaceHistory(super::model::PromptHistorySnapshot),
    /// Navigate supplied history.
    MoveHistory(PromptHistoryDirection),
    /// Replace caller validation observation.
    ReplaceValidation(super::model::PromptValidationSnapshot),
    /// Submit with exact observed generations.
    Submit {
        /// Exact committed input revision observed by the caller.
        input_revision: super::model::PromptInputRevision,
        /// Exact catalog generation for candidate acceptance.
        candidate_generation: Option<super::model::PromptGeneration>,
    },
    /// Request cancellation.
    Cancel,
    /// Acknowledge an emitted close request.
    AcknowledgeClose,
}

/// Result of one pure transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptTransition {
    /// Next accepted state or the preserved previous state.
    pub session: PromptSession,
    /// Whether candidate state was accepted.
    pub accepted: bool,
    /// Optional typed non-authorizing output.
    pub output: Option<PromptOutput>,
    /// Deterministic diagnostics.
    pub diagnostics: Vec<PromptDiagnostic>,
    /// Whether renderer-visible projection changed.
    pub invalidated: bool,
}
