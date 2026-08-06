/// Stable renderer-policy diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RendererPolicyIssueCode {
    /// Identity or required label is empty.
    EmptyIdentity,
    /// A family identity is duplicated.
    DuplicateFamily,
    /// A target identity is duplicated.
    DuplicateTarget,
    /// An evidence adapter identity is duplicated.
    DuplicateEvidenceAdapter,
    /// A target references an unknown family.
    UnknownFamily,
    /// An adapter references an unknown target.
    UnknownTarget,
    /// A replacement identity is unknown.
    UnknownReplacement,
    /// A maintained child is attached to a non-maintained parent.
    LifecycleMismatch,
    /// A retired entry lacks a reason.
    RetiredWithoutReason,
    /// More than one adapter owns a target/evidence-role mapping.
    DuplicateEvidenceMapping,
    /// Scope identity or member list is empty.
    EmptyScope,
    /// A scope member is duplicated.
    DuplicateScopeMember,
    /// A scope member has the wrong nominal identity kind.
    WrongScopeMemberKind,
    /// A scope member is not registered.
    UnknownScopeMember,
    /// A retired renderer is required by a scope.
    RetiredScopeMember,
    /// An experimental renderer is included in maintained completion.
    ExperimentalInMaintainedScope,
    /// A renderer cell is duplicated.
    DuplicateCell,
    /// A required renderer cell is missing.
    MissingRequiredCell,
    /// A renderer cell is not in the named scope.
    OutOfScopeCell,
    /// Renderer cells do not follow scope order.
    CellOrderMismatch,
    /// Unsupported support lacks a reason.
    MissingUnsupportedReason,
    /// Complete support lacks evidence.
    MissingEvidence,
    /// Partial or missing support lacks a next action.
    MissingNextAction,
}

/// Deterministic renderer-policy diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RendererPolicyIssue {
    /// Stable diagnostic code.
    pub code: RendererPolicyIssueCode,
    /// Identity domain such as family, target, adapter, scope, or cell.
    pub domain: String,
    /// Stable affected identity.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl RendererPolicyIssue {
    fn new(
        code: RendererPolicyIssueCode,
        domain: &'static str,
        subject: impl Into<String>,
        message: &'static str,
    ) -> Self {
        Self {
            code,
            domain: domain.to_string(),
            subject: subject.into(),
            message: message.to_string(),
        }
    }
}

/// Renderer policy validation result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RendererPolicyReport {
    /// Stable ordered diagnostics.
    pub issues: Vec<RendererPolicyIssue>,
}

impl RendererPolicyReport {
    /// Returns true when the policy is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

#[path = "checks/registry.rs"]
mod registry;
#[path = "checks/requirements.rs"]
mod requirements;

pub use registry::validate_renderer_registry;
pub use requirements::validate_renderer_scope;
pub use requirements::validate_scoped_renderer_cells;

pub(super) fn finish(mut issues: Vec<RendererPolicyIssue>) -> RendererPolicyReport {
    issues.sort();
    issues.dedup();
    RendererPolicyReport { issues }
}
