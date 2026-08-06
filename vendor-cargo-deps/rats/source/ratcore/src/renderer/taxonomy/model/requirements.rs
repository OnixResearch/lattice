/// Completion semantics applied by a requirement scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererScopeCompletion {
    /// Scope contributes to maintained completion claims.
    Maintained,
    /// Scope is isolated experimental evidence.
    Experimental,
}

/// Ordered renderer requirement scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererRequirementScope {
    /// Stable scope identity.
    pub id: super::RendererScopeId,
    /// Nominal kind required by every member.
    pub subject_kind: super::RendererSubjectKind,
    /// Ordered required members.
    pub members: Vec<super::RendererSubject>,
    /// Whether completion is maintained or experimental.
    pub completion: RendererScopeCompletion,
}

impl RendererRequirementScope {
    /// Creates an empty named scope.
    #[must_use]
    pub fn new(
        id: impl Into<super::RendererScopeId>,
        subject_kind: super::RendererSubjectKind,
        completion: RendererScopeCompletion,
    ) -> Self {
        Self {
            id: id.into(),
            subject_kind,
            members: Vec::new(),
            completion,
        }
    }

    /// Adds an ordered scope member.
    #[must_use]
    pub fn with_member(mut self, member: super::RendererSubject) -> Self {
        self.members.push(member);
        self
    }
}

/// Generic support state used by policy-aware consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererCellStatus {
    /// Complete support with evidence.
    Complete,
    /// Partial support with evidence and a next action.
    Partial,
    /// Unsupported with an explicit reason.
    Unsupported,
    /// Explicitly missing with a next action.
    Missing,
}

/// One typed renderer cell validated against a requirement scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedRendererCell {
    /// Nominal renderer identity.
    pub subject: super::RendererSubject,
    /// Support state.
    pub status: RendererCellStatus,
    /// Evidence links or receipts.
    pub evidence: Vec<String>,
    /// Unsupported reason.
    pub reason: Option<String>,
    /// Follow-up action for partial or missing support.
    pub next_action: Option<String>,
}

impl ScopedRendererCell {
    /// Creates a typed renderer cell.
    #[must_use]
    pub fn new(subject: super::RendererSubject, status: RendererCellStatus) -> Self {
        Self {
            subject,
            status,
            evidence: Vec::new(),
            reason: None,
            next_action: None,
        }
    }

    /// Adds evidence.
    #[must_use]
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence.push(evidence.into());
        self
    }

    /// Adds an unsupported reason.
    #[must_use]
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Adds a follow-up action.
    #[must_use]
    pub fn with_next_action(mut self, next_action: impl Into<String>) -> Self {
        self.next_action = Some(next_action.into());
        self
    }
}
