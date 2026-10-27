//! Typed component-scope state, command, observation, effect, and evidence data.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

/// Minimum valid scope generation.
pub(crate) const MINIMUM_SCOPE_GENERATION: u64 = 1;
/// Maximum number of registrations in one scope.
pub(crate) const MAX_SCOPE_REGISTRATIONS: usize = 512;
/// Maximum number of dependencies for one registration.
pub(crate) const MAX_REGISTRATION_DEPENDENCIES: usize = 128;
/// Maximum number of transition classes retained in one scope.
pub(crate) const MAX_SCOPE_TRANSITIONS: usize = 2_048;
/// Maximum safe label size.
pub(crate) const MAX_SCOPE_LABEL_BYTES: usize = 256;
/// Maximum required non-claim count.
pub(crate) const MAX_SCOPE_NON_CLAIMS: usize = 16;
/// Scope receipt schema.
pub(crate) const SCOPE_RECEIPT_SCHEMA: &str = "animus.component-scope-receipt.v1";
/// Scope identity domain.
pub(crate) const SCOPE_IDENTITY_DOMAIN: &str = "animus/component-scope/v1";
/// Registration identity domain.
pub(crate) const REGISTRATION_IDENTITY_DOMAIN: &str = "animus/component-scope-registration/v1";
/// Lease identity domain.
pub(crate) const LEASE_IDENTITY_DOMAIN: &str = "animus/component-scope-lease/v1";
/// Close-plan identity domain.
pub(crate) const CLOSE_PLAN_IDENTITY_DOMAIN: &str = "animus/component-scope-close-plan/v1";
/// Receipt identity domain.
pub(crate) const SCOPE_RECEIPT_IDENTITY_DOMAIN: &str = "animus/component-scope-receipt/v1";
/// Hash frame separator.
pub(crate) const HASH_SEPARATOR: u8 = 0;
/// Mandatory claim boundaries.
pub(crate) const REQUIRED_SCOPE_NON_CLAIMS: &[&str] = &[
    "not-effect-authority",
    "not-task-cancellation",
    "not-process-death",
    "not-resource-reclamation",
    "not-sandboxing",
    "not-production-readiness",
    "not-release-eligibility",
    "not-external-liveness",
];

/// Exact non-zero component-scope generation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopeGeneration(u64);

impl ScopeGeneration {
    /// Validates a non-zero generation.
    ///
    /// # Errors
    ///
    /// Returns [`ComponentScopeError::InvalidGeneration`] for zero.
    pub const fn new(value: u64) -> Result<Self, ComponentScopeError> {
        if value < MINIMUM_SCOPE_GENERATION {
            return Err(ComponentScopeError::InvalidGeneration);
        }
        Ok(Self(value))
    }

    /// Returns the numeric generation.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Explicit parent-scope binding for one child scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeParent {
    /// Parent scope identity.
    pub scope_id: String,
    /// Exact parent generation.
    pub generation: ScopeGeneration,
}

/// Kind of member owned by a component scope.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeMemberKind {
    /// One resolvable component registration.
    Component,
    /// One child scope owned by a component registration.
    ChildScope,
}

impl ScopeMemberKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::ChildScope => "child_scope",
        }
    }
}

/// Visibility of one component registration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeVisibility {
    /// Only the exact owning scope can resolve the component.
    LocalOnly,
    /// The exact owning scope and one explicitly supplied child can resolve it.
    Descendants,
}

impl ScopeVisibility {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::LocalOnly => "local_only",
            Self::Descendants => "descendants",
        }
    }
}

/// Unvalidated registration for one component or owned child scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentRegistrationDraft {
    /// Stable member identity. A component member must use its component identity.
    pub member_id: String,
    /// Catalog component that owns this member.
    pub component_id: String,
    /// Member kind.
    pub member_kind: ScopeMemberKind,
    /// Resolution visibility.
    pub visibility: ScopeVisibility,
    /// Explicit member identities that this member depends on.
    pub dependency_member_ids: Vec<String>,
}

/// Validated generation-bound registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentRegistration {
    pub(crate) member_id: String,
    pub(crate) component_id: String,
    pub(crate) member_kind: ScopeMemberKind,
    pub(crate) visibility: ScopeVisibility,
    pub(crate) dependency_member_ids: Vec<String>,
    pub(crate) identity: String,
}

impl ComponentRegistration {
    /// Returns the member identity.
    #[must_use]
    pub fn member_id(&self) -> &str {
        self.member_id.as_str()
    }

    /// Returns the catalog component identity.
    #[must_use]
    pub fn component_id(&self) -> &str {
        self.component_id.as_str()
    }

    /// Returns the member kind.
    #[must_use]
    pub const fn member_kind(&self) -> ScopeMemberKind {
        self.member_kind
    }

    /// Returns the visibility policy.
    #[must_use]
    pub const fn visibility(&self) -> ScopeVisibility {
        self.visibility
    }

    /// Returns canonical dependency member identities.
    #[must_use]
    pub fn dependency_member_ids(&self) -> &[String] {
        self.dependency_member_ids.as_slice()
    }

    /// Returns the BLAKE3 registration identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Unvalidated component-scope declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentScopeDraft {
    /// Stable scope identity.
    pub scope_id: String,
    /// Exact generation.
    pub generation: ScopeGeneration,
    /// Optional explicit parent binding.
    pub parent: Option<ScopeParent>,
    /// Owned component and child registrations.
    pub registrations: Vec<ComponentRegistrationDraft>,
    /// Mandatory evidence non-claims.
    pub non_claims: Vec<String>,
}

/// Public lifecycle class for one component scope.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComponentScopeState {
    /// Registration and exact resolution are open.
    Open,
    /// Resolution is revoked while delivery closes.
    Closing,
    /// Cancellation was requested and quiescence is pending.
    Quiescing,
    /// Members release in reverse dependency order.
    Releasing,
    /// All required release and receipt observations matched.
    Closed,
    /// One or more cleanup facts remain explicitly unresolved.
    FailedCleanup,
}

/// One deterministic close-plan identity and release order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeClosePlan {
    pub(crate) identity: String,
    pub(crate) generation: ScopeGeneration,
    pub(crate) release_order: Vec<String>,
}

impl ScopeClosePlan {
    /// Returns the BLAKE3 plan identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the exact close generation.
    #[must_use]
    pub const fn generation(&self) -> ScopeGeneration {
        self.generation
    }

    /// Returns reverse dependency release order.
    #[must_use]
    pub fn release_order(&self) -> &[String] {
        self.release_order.as_slice()
    }
}

/// Deterministic transition class retained in metadata-only evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeTransitionClass {
    /// Scope declaration was admitted.
    ScopeAdmitted,
    /// New resolution was revoked and delivery close was requested.
    ResolutionRevoked,
    /// Delivery close was observed and cancellation was requested.
    DeliveryClosed,
    /// One exact member reached quiescence.
    MemberQuiescent,
    /// All members reached quiescence and release started.
    ReleaseStarted,
    /// One exact member release was observed.
    MemberReleased,
    /// A terminal receipt commit was requested.
    TerminalCommitRequested,
    /// A terminal receipt commit was observed.
    TerminalCommitObserved,
    /// Cleanup remained explicitly unresolved.
    CleanupFailed,
    /// A repeated close request reused the current plan.
    CloseReused,
}

impl ScopeTransitionClass {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ScopeAdmitted => "scope_admitted",
            Self::ResolutionRevoked => "resolution_revoked",
            Self::DeliveryClosed => "delivery_closed",
            Self::MemberQuiescent => "member_quiescent",
            Self::ReleaseStarted => "release_started",
            Self::MemberReleased => "member_released",
            Self::TerminalCommitRequested => "terminal_commit_requested",
            Self::TerminalCommitObserved => "terminal_commit_observed",
            Self::CleanupFailed => "cleanup_failed",
            Self::CloseReused => "close_reused",
        }
    }
}

/// Terminal cleanup outcome.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeTerminalOutcome {
    /// All exact cleanup observations matched.
    Closed,
    /// At least one cleanup fact remained unresolved.
    FailedCleanup,
}

impl ScopeTerminalOutcome {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::FailedCleanup => "failed_cleanup",
        }
    }
}

/// Safe unresolved-cleanup class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScopeCleanupIssueKind {
    /// Delivery close was not observed.
    DeliveryClose,
    /// Cancellation request was not admitted for one member.
    CancellationRequest,
    /// One member did not reach quiescence before host policy expired.
    QuiescenceDeadline,
    /// One member did not return an exact release observation.
    ReleaseObservation,
}

impl ScopeCleanupIssueKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DeliveryClose => "delivery_close_unresolved",
            Self::CancellationRequest => "cancellation_unresolved",
            Self::QuiescenceDeadline => "quiescence_unresolved",
            Self::ReleaseObservation => "release_unresolved",
        }
    }
}

/// One bounded metadata-only cleanup issue.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ScopeCleanupIssue {
    /// Safe issue class.
    pub kind: ScopeCleanupIssueKind,
    /// Safe scope or member identity.
    pub subject: String,
}

/// Host effect request emitted by the pure reducer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeEffectRequest {
    /// Close new event or message delivery for the scope generation.
    CloseDelivery {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
    },
    /// Request cancellation for one owned member.
    RequestCancellation {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Owned member identity.
        member_id: String,
    },
    /// Observe generation-bound quiescence for one member.
    ObserveQuiescence {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Owned member identity.
        member_id: String,
    },
    /// Release one member resource in the admitted order.
    ReleaseMember {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Owned member identity.
        member_id: String,
    },
    /// Commit one metadata-only terminal receipt.
    CommitTerminalReceipt {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Exact receipt identity.
        receipt_identity: String,
        /// Terminal outcome.
        outcome: ScopeTerminalOutcome,
    },
}

/// Exact host observation consumed by the pure reducer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeObservation {
    /// New delivery is closed for the exact scope generation.
    DeliveryClosed {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
    },
    /// One owned member reached quiescence.
    MemberQuiescent {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Owned member identity.
        member_id: String,
    },
    /// One owned member was released.
    MemberReleased {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Owned member identity.
        member_id: String,
    },
    /// One exact terminal receipt was committed.
    TerminalReceiptCommitted {
        /// Scope identity.
        scope_id: String,
        /// Exact generation.
        generation: ScopeGeneration,
        /// Stable close-plan identity.
        plan_identity: String,
        /// Exact receipt identity.
        receipt_identity: String,
    },
}

/// Exact cleanup failure supplied by the host shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeCleanupFailure {
    /// Scope identity.
    pub scope_id: String,
    /// Exact generation.
    pub generation: ScopeGeneration,
    /// Stable close-plan identity.
    pub plan_identity: String,
    /// Safe unresolved-cleanup class.
    pub kind: ScopeCleanupIssueKind,
    /// Exact member identity, or the scope identity for delivery close.
    pub subject: String,
}

/// Exact close request. It carries no client identity or effect authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeCloseRequest {
    /// Scope identity.
    pub scope_id: String,
    /// Exact generation.
    pub generation: ScopeGeneration,
}

/// Explicit resolution target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeResolutionTarget {
    /// Resolve only against this exact scope generation.
    Exact {
        /// Target scope identity.
        scope_id: String,
        /// Target scope generation.
        generation: ScopeGeneration,
    },
    /// Denied ambient lookup request.
    Ambient,
}

/// Explicit component resolution request from one exact current scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeResolutionRequest {
    /// Requesting scope identity.
    pub requester_scope_id: String,
    /// Requesting scope generation.
    pub requester_generation: ScopeGeneration,
    /// Explicit target or denied ambient mode.
    pub target: ScopeResolutionTarget,
    /// Exact catalog component identity.
    pub component_id: String,
}

/// Generation-bound metadata lease returned by exact resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentLease {
    pub(crate) scope_id: String,
    pub(crate) generation: ScopeGeneration,
    pub(crate) component_id: String,
    pub(crate) registration_identity: String,
    pub(crate) identity: String,
    pub(crate) effect_authority_granted: bool,
}

impl ComponentLease {
    /// Returns the exact owning scope identity.
    #[must_use]
    pub fn scope_id(&self) -> &str {
        self.scope_id.as_str()
    }

    /// Returns the exact owning generation.
    #[must_use]
    pub const fn generation(&self) -> ScopeGeneration {
        self.generation
    }

    /// Returns the exact component identity.
    #[must_use]
    pub fn component_id(&self) -> &str {
        self.component_id.as_str()
    }

    /// Returns the bound registration identity.
    #[must_use]
    pub fn registration_identity(&self) -> &str {
        self.registration_identity.as_str()
    }

    /// Returns the BLAKE3 lease identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns false because a resolution lease grants no effect authority.
    #[must_use]
    pub const fn grants_effect_authority(&self) -> bool {
        self.effect_authority_granted
    }
}

/// Validated component scope and pure lifecycle progress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentScope {
    pub(crate) scope_id: String,
    pub(crate) generation: ScopeGeneration,
    pub(crate) graph_identity: String,
    pub(crate) parent: Option<ScopeParent>,
    pub(crate) registrations: Vec<ComponentRegistration>,
    pub(crate) release_order: Vec<String>,
    pub(crate) state: ComponentScopeState,
    pub(crate) close_plan: Option<ScopeClosePlan>,
    pub(crate) quiescent_member_ids: BTreeSet<String>,
    pub(crate) released_member_ids: Vec<String>,
    pub(crate) unresolved_cleanup: Vec<ScopeCleanupIssue>,
    pub(crate) transition_classes: Vec<ScopeTransitionClass>,
    pub(crate) requested_effect_count: usize,
    pub(crate) terminal_outcome: Option<ScopeTerminalOutcome>,
    pub(crate) terminal_receipt_identity: Option<String>,
    pub(crate) terminal_receipt_committed: bool,
    pub(crate) non_claims: Vec<String>,
    pub(crate) identity: String,
}

impl ComponentScope {
    /// Returns the scope identity.
    #[must_use]
    pub fn scope_id(&self) -> &str {
        self.scope_id.as_str()
    }

    /// Returns the exact scope generation.
    #[must_use]
    pub const fn generation(&self) -> ScopeGeneration {
        self.generation
    }

    /// Returns the admitted activation-graph identity.
    #[must_use]
    pub fn graph_identity(&self) -> &str {
        self.graph_identity.as_str()
    }

    /// Returns the explicit parent binding, if present.
    #[must_use]
    pub const fn parent(&self) -> Option<&ScopeParent> {
        self.parent.as_ref()
    }

    /// Returns canonical registrations.
    #[must_use]
    pub fn registrations(&self) -> &[ComponentRegistration] {
        self.registrations.as_slice()
    }

    /// Returns the current lifecycle class.
    #[must_use]
    pub const fn state(&self) -> ComponentScopeState {
        self.state
    }

    /// Returns the fixed close plan after close starts.
    #[must_use]
    pub const fn close_plan(&self) -> Option<&ScopeClosePlan> {
        self.close_plan.as_ref()
    }

    /// Returns the deterministic scope identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns true only while exact resolution remains open.
    #[must_use]
    pub const fn resolution_open(&self) -> bool {
        matches!(self.state, ComponentScopeState::Open)
    }

    /// Returns safe unresolved cleanup issues.
    #[must_use]
    pub fn unresolved_cleanup(&self) -> &[ScopeCleanupIssue] {
        self.unresolved_cleanup.as_slice()
    }

    /// Returns the projected terminal receipt identity, if requested.
    #[must_use]
    pub fn terminal_receipt_identity(&self) -> Option<&str> {
        self.terminal_receipt_identity.as_deref()
    }

    /// Returns true only after the exact terminal receipt commit observation.
    #[must_use]
    pub const fn terminal_receipt_committed(&self) -> bool {
        self.terminal_receipt_committed
    }
}

/// Pure reducer output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeTransition {
    scope: ComponentScope,
    class: ScopeTransitionClass,
    effects: Vec<ScopeEffectRequest>,
}

impl ScopeTransition {
    pub(crate) const fn new(
        scope: ComponentScope,
        class: ScopeTransitionClass,
        effects: Vec<ScopeEffectRequest>,
    ) -> Self {
        Self {
            scope,
            class,
            effects,
        }
    }

    /// Returns the next scope state.
    #[must_use]
    pub const fn scope(&self) -> &ComponentScope {
        &self.scope
    }

    /// Returns the transition class.
    #[must_use]
    pub const fn class(&self) -> ScopeTransitionClass {
        self.class
    }

    /// Returns reducer-requested effects.
    #[must_use]
    pub fn effects(&self) -> &[ScopeEffectRequest] {
        self.effects.as_slice()
    }

    /// Consumes the transition into the next scope and effects.
    #[must_use]
    pub fn into_parts(self) -> (ComponentScope, Vec<ScopeEffectRequest>) {
        (self.scope, self.effects)
    }
}

/// Metadata-only lifecycle receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentScopeReceipt {
    /// Receipt schema.
    pub schema: &'static str,
    /// Scope identity.
    pub scope_id: String,
    /// Scope generation.
    pub generation: ScopeGeneration,
    /// Admitted activation-graph identity.
    pub graph_identity: String,
    /// Stable close-plan identity.
    pub plan_identity: String,
    /// Terminal outcome.
    pub outcome: ScopeTerminalOutcome,
    /// Component member count.
    pub component_count: u16,
    /// Child-scope member count.
    pub child_scope_count: u16,
    /// Requested host-effect count.
    pub requested_effect_count: u32,
    /// Observed quiescent member count.
    pub observed_quiescent_count: u16,
    /// Observed released member count.
    pub observed_release_count: u16,
    /// Safe unresolved cleanup issues.
    pub unresolved_cleanup: Vec<ScopeCleanupIssue>,
    /// Bounded transition classes.
    pub transition_classes: Vec<ScopeTransitionClass>,
    /// Required claim exclusions.
    pub non_claims: Vec<String>,
    /// Canonical BLAKE3 receipt identity.
    pub identity: String,
}

/// Typed component-scope rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentScopeError {
    /// A scope generation was zero.
    InvalidGeneration,
    /// A label, hash, or evidence field was malformed.
    InvalidIdentity,
    /// A configured collection exceeded its bound.
    BoundExceeded,
    /// A registration or dependency identity was duplicated.
    DuplicateRegistration,
    /// A registration names no activation-graph component.
    UnknownComponent,
    /// A registration dependency names no scope member.
    UnknownDependency,
    /// The activation graph does not admit a declared dependency.
    UndeclaredDependency,
    /// The admitted ownership graph has a cycle.
    DependencyCycle,
    /// Scope identity differs from the current state.
    ScopeMismatch,
    /// Generation differs from the current scope or parent.
    GenerationMismatch,
    /// Current state does not admit the requested operation.
    InvalidState,
    /// Ambient lookup is not allowed.
    AmbientResolutionDenied,
    /// The explicit target is outside the current scope and parent.
    OutOfScope,
    /// The component is absent or not visible to the requester.
    ComponentNotVisible,
    /// An observation names an unknown member.
    UnknownMember,
    /// An observation repeats a completed fact.
    DuplicateObservation,
    /// An observation is out of sequence or does not match the plan.
    InvalidObservation,
    /// Required cleanup remains incomplete.
    CleanupIncomplete,
    /// A receipt is not available for the current state.
    ReceiptNotReady,
    /// Evidence text contains an unsafe field or missing non-claim.
    UnsafeEvidence,
}

impl core::fmt::Display for ComponentScopeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "component scope rejected: {self:?}")
    }
}
