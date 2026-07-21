/// Review-mode action classes used by the read-only policy core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewActionKind {
    /// Inspect a diff, patch, or hunk without mutation.
    InspectDiff,
    /// Read a source file through an admitted read-only adapter.
    ReadFile,
    /// Ask a model to produce review findings.
    ModelReview,
    /// Edit or write a file.
    EditFile,
    /// Invoke a write-capable tool.
    WriteCapableTool,
    /// Request approval escalation inside the review run.
    ApprovalEscalation,
    /// Create a commit or equivalent version-control mutation.
    CommitCreation,
    /// Open, update, merge, or otherwise manage a pull request.
    PullRequestManagement,
}

/// Reasons the default review policy denies an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewDenialReason {
    /// The action would mutate files.
    FileMutation,
    /// The action would dispatch a write-capable tool.
    WriteCapableTool,
    /// The action would change approval authority.
    ApprovalEscalation,
    /// The action would mutate version-control state.
    VersionControlMutation,
    /// The action would manage pull requests.
    PullRequestManagement,
}

/// Pure policy profile for dedicated diff-review mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewPolicyProfile {
    read_only: bool,
}

impl DiffReviewPolicyProfile {
    /// Creates the default read-only review policy profile.
    #[must_use]
    pub const fn read_only() -> Self {
        Self { read_only: true }
    }

    /// Evaluates one planned action without performing it.
    #[must_use]
    pub const fn evaluate_action(self, action: DiffReviewActionKind) -> DiffReviewPolicyDecision {
        if !self.read_only {
            return DiffReviewPolicyDecision::denied(
                action,
                DiffReviewDenialReason::ApprovalEscalation,
            );
        }
        match action {
            DiffReviewActionKind::InspectDiff
            | DiffReviewActionKind::ReadFile
            | DiffReviewActionKind::ModelReview => DiffReviewPolicyDecision::allowed(action),
            DiffReviewActionKind::EditFile => {
                DiffReviewPolicyDecision::denied(action, DiffReviewDenialReason::FileMutation)
            }
            DiffReviewActionKind::WriteCapableTool => {
                DiffReviewPolicyDecision::denied(action, DiffReviewDenialReason::WriteCapableTool)
            }
            DiffReviewActionKind::ApprovalEscalation => {
                DiffReviewPolicyDecision::denied(action, DiffReviewDenialReason::ApprovalEscalation)
            }
            DiffReviewActionKind::CommitCreation => DiffReviewPolicyDecision::denied(
                action,
                DiffReviewDenialReason::VersionControlMutation,
            ),
            DiffReviewActionKind::PullRequestManagement => DiffReviewPolicyDecision::denied(
                action,
                DiffReviewDenialReason::PullRequestManagement,
            ),
        }
    }
}

/// Policy decision for one review-mode action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewPolicyDecision {
    action: DiffReviewActionKind,
    denial_reason: Option<DiffReviewDenialReason>,
}

impl DiffReviewPolicyDecision {
    const fn allowed(action: DiffReviewActionKind) -> Self {
        Self {
            action,
            denial_reason: None,
        }
    }

    const fn denied(action: DiffReviewActionKind, reason: DiffReviewDenialReason) -> Self {
        Self {
            action,
            denial_reason: Some(reason),
        }
    }

    /// Returns the evaluated action.
    #[must_use]
    pub const fn action(self) -> DiffReviewActionKind {
        self.action
    }

    /// Returns whether the policy admits the action.
    #[must_use]
    pub const fn is_allowed(self) -> bool {
        self.denial_reason.is_none()
    }

    /// Returns the denial reason when the action was rejected.
    #[must_use]
    pub const fn denial_reason(self) -> Option<DiffReviewDenialReason> {
        self.denial_reason
    }
}
