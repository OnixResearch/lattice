//! Pure exact-binding and authority models for Choregraph product effects.

mod evaluation;
mod receipt;

pub use evaluation::evaluate;
pub use evaluation::evaluate_for;
pub use receipt::basalt_diagnostic;
pub use receipt::blake3_reference;
pub use receipt::observed_outcome_event;
pub use receipt::outcome_event;
pub use receipt::replay_observation;
pub use receipt::reserve_attempt;

// r[impl lattice.choregraph_effect_authority.policy]
pub const CHOREGRAPH_EFFECT_AUTHORITY_POLICY_JSON: &str =
    include_str!("../../../policy/generated/choregraph-effect-authority.json");
pub const CHOREGRAPH_EFFECT_AUTHORITY_POLICY_BLAKE3: &str =
    "56800b3cb2f0819e31cbd32fc3ee41e1a74e7dbf6495540ad8d50db5341f0a79";
pub const CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF: &str =
    "blake3:56800b3cb2f0819e31cbd32fc3ee41e1a74e7dbf6495540ad8d50db5341f0a79";
pub const CHOREGRAPH_EFFECT_AUTHORITY_POLICY_SCHEMA: &str = basalt::SUPPORTED_SCHEMA_VERSION;
pub const CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA: &str = "lattice.choregraph-effect-binding.v1";
pub const CHOREGRAPH_EFFECT_SCOPE_SCHEMA: &str = "molten.collaboration-scope.snapshot.v1";
pub const UNSCOPED_SCOPE_IDENTITY: &str = "unscoped";
pub const REQUEST_REFERENCE_PROFILE: &str = choregraph_core::CANONICAL_REFERENCE_PROFILE_V1;

macro_rules! identity_type {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, BindingError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(BindingError::EmptyIdentity);
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

identity_type!(MachineIdentity);
identity_type!(WorkflowRevisionIdentity);
identity_type!(RunIdentity);
identity_type!(NodeIdentity);
identity_type!(TransitionIdentity);
identity_type!(OperationIdentity);
identity_type!(ResourceIdentity);
identity_type!(AbilityIdentity);
identity_type!(RequestIdentity);
identity_type!(PayloadIdentity);
identity_type!(PolicyIdentity);
identity_type!(AuthorityIdentity);
identity_type!(IntentIdentity);
identity_type!(AttemptIdentity);
identity_type!(ScopeIdentity);
identity_type!(ScopeRevisionIdentity);
identity_type!(AudienceIdentity);
identity_type!(ConsentDecisionIdentity);
identity_type!(ScopeSnapshotIdentity);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IntentBinding {
    pub schema: String,
    pub machine: MachineIdentity,
    pub workflow_revision: WorkflowRevisionIdentity,
    pub run: RunIdentity,
    pub node: NodeIdentity,
    pub transition: TransitionIdentity,
    pub operation: OperationIdentity,
    pub resource: ResourceIdentity,
    pub ability: AbilityIdentity,
    pub request: RequestIdentity,
    pub payload: PayloadIdentity,
    pub policy: PolicyIdentity,
    pub authority: AuthorityIdentity,
    pub intent: IntentIdentity,
    pub scope: CollaborationScopeBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollaborationScopeBinding {
    Unscoped,
    Protected {
        scope: ScopeIdentity,
        revision: ScopeRevisionIdentity,
        membership_epoch: u64,
        audience: AudienceIdentity,
        required_consent: ConsentDecisionIdentity,
        snapshot: ScopeSnapshotIdentity,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScopeSnapshotFacts {
    pub schema: String,
    pub scope: ScopeIdentity,
    pub revision: ScopeRevisionIdentity,
    pub membership_epoch: u64,
    pub audience: AudienceIdentity,
    pub consent: ConsentDecisionIdentity,
    pub resource: ResourceIdentity,
    pub ability: AbilityIdentity,
    pub request: RequestIdentity,
    pub snapshot: ScopeSnapshotIdentity,
    pub membership_current: bool,
    pub audience_current: bool,
    pub consent_current: bool,
    pub revocation_current: bool,
}

#[derive(Debug, Clone)]
pub struct AuthorizationFacts {
    pub binding: IntentBinding,
    pub evidence: Evidence,
    pub trusted_policy_ref: basalt::authority_core::BasaltPolicyRef,
    pub scope_requirement: ScopeRequirement,
    pub scope_snapshot: Option<ScopeSnapshotFacts>,
    pub revocation: RevocationDisposition,
}

/// Distinct trust boundaries: verified external UCAN facts or first-party local admission.
#[derive(Debug, Clone)]
pub enum Evidence {
    VerifiedUcan(basalt::authority_core::AuthorityInput),
    LocalOperator(LocalOperatorGrant),
}

/// A host-admitted local capability, never a token-verification receipt.
#[derive(Debug, Clone)]
pub struct LocalOperatorGrant {
    pub(crate) authority_ref: String,
    pub(crate) operation: String,
    pub(crate) request: basalt::authority_core::AuthorityRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeRequirement {
    UnscopedPermitted,
    ProtectedRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationDisposition {
    Current,
    Revoked,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorizationDenial {
    MalformedBinding,
    BindingMismatch,
    PolicyMismatch,
    AuthorityUnavailable,
    BasaltDenied,
    ScopeRequired,
    ScopeMismatch,
    StaleScope,
    ReplayDenied,
    RevocationDenied,
    AdapterRejected,
}

impl AuthorizationDenial {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MalformedBinding => "malformed-binding",
            Self::BindingMismatch => "binding-mismatch",
            Self::PolicyMismatch => "policy-mismatch",
            Self::AuthorityUnavailable => "authority-unavailable",
            Self::BasaltDenied => "basalt-denied",
            Self::ScopeRequired => "scope-required",
            Self::ScopeMismatch => "scope-mismatch",
            Self::StaleScope => "stale-scope",
            Self::ReplayDenied => "replay-denied",
            Self::RevocationDenied => "revocation-denied",
            Self::AdapterRejected => "adapter-rejected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub binding: Option<IntentBinding>,
    pub denial: Option<AuthorizationDenial>,
    pub basalt_diagnostic: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingError {
    EmptyIdentity,
    MalformedReference,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Attempt {
    pub identity: AttemptIdentity,
    pub binding: IntentBinding,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuthorizationOutcomeEvent {
    pub identity: String,
    pub attempt: AttemptIdentity,
    pub state: OutcomeState,
    pub denial: Option<AuthorizationDenial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutcomeState {
    Authorized,
    Denied,
    ObservedSuccess,
    ObservedFailure,
    UnknownAfterDispatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptError {
    MalformedBinding,
    Duplicate,
}
