/// Final security review decision status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Dispatch may continue to the normal runtime authority boundary.
    Allowed,
    /// Dispatch must stop before provider, tool, or host effects.
    Denied,
    /// Dispatch requires host/user confirmation before continuing.
    RequiresConfirmation,
}

impl Status {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::RequiresConfirmation => "requires_confirmation",
        }
    }

    const fn dispatch_allowed(self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Security review decision issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// Review completed without additional blocking evidence.
    None,
    /// Hard authority was missing before advisory review could matter.
    MissingAuthority,
    /// Capability policy denied the action.
    CapabilityDenied,
    /// Required sandbox policy was not enforceable.
    SandboxDenied,
    /// Required confirmation has not been satisfied.
    ConfirmationRequired,
    /// Deterministic scanner produced a blocking finding.
    BlockingFinding,
    /// Advisory reviewer produced a blocking recommendation.
    AdvisoryBlocked,
    /// Scanner or advisory review was unavailable, timed out, or malformed.
    ReviewUnavailable,
}

impl Issue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::MissingAuthority => "missing_authority",
            Self::CapabilityDenied => "capability_denied",
            Self::SandboxDenied => "sandbox_denied",
            Self::ConfirmationRequired => "confirmation_required",
            Self::BlockingFinding => "blocking_finding",
            Self::AdvisoryBlocked => "advisory_blocked",
            Self::ReviewUnavailable => "review_unavailable",
        }
    }
}

/// State of one hard permission gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gate {
    /// The gate admitted the reviewed action.
    Granted,
    /// The gate denied or has not admitted the reviewed action.
    Denied,
}

impl Gate {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Host authority snapshot that remains above advisory review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Snapshot {
    /// Hard authority grant gate.
    pub authority: Gate,
    /// Capability policy gate.
    pub capability: Gate,
    /// Required sandbox policy gate.
    pub sandbox: Gate,
    /// Required confirmation gate.
    pub confirmation: Gate,
}

impl Snapshot {
    /// Returns a snapshot where all hard gates are satisfied.
    #[must_use]
    pub const fn all_granted() -> Self {
        Self {
            authority: Gate::Granted,
            capability: Gate::Granted,
            sandbox: Gate::Granted,
            confirmation: Gate::Granted,
        }
    }
}

/// Explicit fail behavior for unavailable review subsystems.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailBehavior {
    /// Deny dispatch when review is unavailable.
    Deny,
    /// Require confirmation when review is unavailable.
    RequireConfirmation,
    /// Continue using completed deterministic scanner evidence only.
    ScannerOnly,
}

/// Review outage policy keyed by action risk class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FailPolicy {
    /// Behavior for high-risk actions.
    pub high_risk: FailBehavior,
    /// Behavior for low-risk actions.
    pub low_risk: FailBehavior,
}

impl FailPolicy {
    /// Returns the default fail-closed high-risk policy.
    #[must_use]
    pub const fn fail_closed_high_risk() -> Self {
        Self {
            high_risk: FailBehavior::Deny,
            low_risk: FailBehavior::ScannerOnly,
        }
    }

    const fn behavior_for(self, risk_class: super::RiskClass) -> FailBehavior {
        match risk_class {
            super::RiskClass::Low => self.low_risk,
            super::RiskClass::High => self.high_risk,
        }
    }
}

impl Default for FailPolicy {
    fn default() -> Self {
        Self::fail_closed_high_risk()
    }
}

/// Borrowed facts for binding review into dispatch permission metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding<'a> {
    /// Validated review input.
    pub input: &'a super::Review,
    /// Deterministic scanner report.
    pub scan_report: &'a super::Report,
    /// Optional classifier or adversary-review outcomes.
    pub advisory_reviews: &'a [super::AdvisoryReview],
    /// Hard permission gate snapshot.
    pub permissions: Snapshot,
    /// Explicit review outage policy.
    pub fail_policy: FailPolicy,
}

/// Bound security decision with a safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    status: Status,
    issue: Issue,
    receipt: super::Receipt,
}

#[derive(Clone, Copy)]
struct Draft<'a> {
    binding: Binding<'a>,
    status: Status,
    issue: Issue,
}

impl Outcome {
    fn new(draft: Draft<'_>) -> Self {
        let receipt = super::Receipt::new(super::evidence::Draft {
            input: draft.binding.input,
            scan_report: draft.binding.scan_report,
            advisory_reviews: draft.binding.advisory_reviews,
            status: draft.status,
            issue: draft.issue,
        });
        Self {
            status: draft.status,
            issue: draft.issue,
            receipt,
        }
    }

    /// Returns the final status.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Returns the final issue.
    #[must_use]
    pub const fn issue(&self) -> Issue {
        self.issue
    }

    /// Returns true only when dispatch may continue.
    #[must_use]
    pub const fn dispatch_allowed(&self) -> bool {
        self.status.dispatch_allowed()
    }

    /// Returns the safe review receipt.
    #[must_use]
    pub const fn receipt(&self) -> &super::Receipt {
        &self.receipt
    }
}

/// Binds scanner findings and advisory reviews into permission decision metadata.
#[must_use]
pub fn bind(binding: &Binding<'_>) -> Outcome {
    debug_assert!(!binding.input.evidence_hashes().is_empty());
    debug_assert!(
        binding
            .scan_report
            .findings()
            .iter()
            .all(|finding| !finding.evidence_hash().is_empty())
    );
    if !binding.permissions.authority.is_granted() {
        return make_outcome(*binding, Status::Denied, Issue::MissingAuthority);
    }
    if !binding.permissions.capability.is_granted() {
        return make_outcome(*binding, Status::Denied, Issue::CapabilityDenied);
    }
    if !binding.permissions.sandbox.is_granted() {
        return make_outcome(*binding, Status::Denied, Issue::SandboxDenied);
    }
    dispatchable_review(binding)
}

fn dispatchable_review(binding: &Binding<'_>) -> Outcome {
    debug_assert!(binding.permissions.authority.is_granted());
    debug_assert!(binding.permissions.capability.is_granted());
    if let Some(status) = outage_status(binding) {
        return make_outcome(*binding, status, Issue::ReviewUnavailable);
    }
    if binding.scan_report.has_blocking_finding() {
        return make_outcome(*binding, Status::Denied, Issue::BlockingFinding);
    }
    if binding
        .advisory_reviews
        .iter()
        .any(super::AdvisoryReview::blocks_dispatch)
    {
        return make_outcome(*binding, Status::Denied, Issue::AdvisoryBlocked);
    }
    if !binding.permissions.confirmation.is_granted() {
        return make_outcome(
            *binding,
            Status::RequiresConfirmation,
            Issue::ConfirmationRequired,
        );
    }
    make_outcome(*binding, Status::Allowed, Issue::None)
}

fn make_outcome(binding: Binding<'_>, status: Status, issue: Issue) -> Outcome {
    Outcome::new(Draft {
        binding,
        status,
        issue,
    })
}

fn outage_status(binding: &Binding<'_>) -> Option<Status> {
    let is_review_unavailable = !binding.scan_report.status().is_complete()
        || binding
            .advisory_reviews
            .iter()
            .any(super::AdvisoryReview::is_outage);
    if !is_review_unavailable {
        return None;
    }
    match binding.fail_policy.behavior_for(binding.input.risk_class()) {
        FailBehavior::Deny => Some(Status::Denied),
        FailBehavior::RequireConfirmation => Some(Status::RequiresConfirmation),
        FailBehavior::ScannerOnly => None,
    }
}
