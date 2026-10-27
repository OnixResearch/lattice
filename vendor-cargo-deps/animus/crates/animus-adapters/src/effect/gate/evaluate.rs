const DECISION_DOMAIN: &[u8] = b"animus/host-authority-decision/v1";
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_BYTES: usize = 64;

/// Stable provider and composition issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAuthorityIssue {
    /// No issue was observed.
    None,
    /// A named bound was zero, inconsistent, or too large.
    InvalidBound,
    /// A BLAKE3 identity was malformed.
    InvalidIdentity,
    /// The UCAN DID principal was malformed.
    InvalidPrincipal,
    /// Display metadata was unsafe or authority-shaped.
    UnsafeDisplayLabel,
    /// A binding was stale or its freshness was unavailable.
    StaleBinding,
    /// Two admitted bindings alias the same immutable authority.
    DuplicateBinding,
    /// Current request facts do not match the admitted agent binding.
    BindingMismatch,
    /// The action, contract, or ability was malformed or inconsistent.
    InvalidActionBinding,
    /// The resource was malformed, wildcarded, or path-like.
    InvalidResource,
    /// The selected provider does not support this exact action mapping.
    UnsupportedAction,
    /// The required provider or one required service was unavailable.
    ProviderUnavailable,
    /// The provider policy denied the exact request.
    PolicyDenied,
    /// Raw UCAN verification denied the exact request.
    VerificationDenied,
    /// The provider reported stale verification inputs.
    StaleVerification,
    /// Replay admission denied or was uncertain.
    ReplayDenied,
    /// The provider returned a result for a different request.
    DecisionMismatch,
    /// Another required Animus gate denied the action.
    LocalGateDenied,
    /// Durable safe-decision commit failed.
    CommitFailed,
    /// Receipt material was unsafe.
    UnsafeReceipt,
}

impl HostAuthorityIssue {
    /// Return the stable safe issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::InvalidBound => "invalid_bound",
            Self::InvalidIdentity => "invalid_identity",
            Self::InvalidPrincipal => "invalid_principal",
            Self::UnsafeDisplayLabel => "unsafe_display_label",
            Self::StaleBinding => "stale_binding",
            Self::DuplicateBinding => "duplicate_binding",
            Self::BindingMismatch => "binding_mismatch",
            Self::InvalidActionBinding => "invalid_action_binding",
            Self::InvalidResource => "invalid_resource",
            Self::UnsupportedAction => "unsupported_action",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::PolicyDenied => "policy_denied",
            Self::VerificationDenied => "verification_denied",
            Self::StaleVerification => "stale_verification",
            Self::ReplayDenied => "replay_denied",
            Self::DecisionMismatch => "decision_mismatch",
            Self::LocalGateDenied => "local_gate_denied",
            Self::CommitFailed => "commit_failed",
            Self::UnsafeReceipt => "unsafe_receipt",
        }
    }
}

/// Generic provider decision status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAuthorityStatus {
    /// The provider gate admitted the exact request.
    Allowed,
    /// The provider gate denied the request.
    Denied,
}

/// Safe class for provider freshness evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAuthorityFreshnessClass {
    /// Current under the supplied observation.
    Current,
    /// Stale under the supplied observation.
    Stale,
    /// The provider could not establish freshness.
    Unavailable,
}

impl HostAuthorityFreshnessClass {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Safe class for replay admission evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAuthorityReplayClass {
    /// The provider does not use replay for this request.
    NotApplicable,
    /// The final replay gate admitted one use.
    Admitted,
    /// The replay gate denied this use.
    Denied,
    /// Replay state was unavailable or uncertain.
    Unavailable,
}

impl HostAuthorityReplayClass {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Admitted => "admitted",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Provider-supplied safe decision facts before exact request binding.
#[derive(Clone, Copy, Debug)]
pub struct HostAuthorityDecisionDraft<'a> {
    /// Exact request being decided.
    pub request: &'a super::input::HostAuthorityRequest,
    /// Stable provider identity.
    pub provider_identity: &'a str,
    /// Provider policy identity.
    pub policy_identity: &'a str,
    /// Decision status.
    pub status: HostAuthorityStatus,
    /// Stable issue class.
    pub issue: HostAuthorityIssue,
    /// BLAKE3 hash of the verified principal metadata.
    pub principal_hash: &'a str,
    /// BLAKE3 hash of the exact logical resource metadata.
    pub resource_hash: &'a str,
    /// Number of provider facts checked under the named request bound.
    pub checked_fact_count: u32,
    /// Freshness class.
    pub freshness: HostAuthorityFreshnessClass,
    /// Replay class.
    pub replay: HostAuthorityReplayClass,
    /// Provider-specific redacted receipt identity.
    pub provider_receipt_identity: &'a str,
}

/// Validated metadata-only provider decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAuthorityDecision {
    request_identity: alloc::string::String,
    binding_identity: alloc::string::String,
    provider_identity: alloc::string::String,
    policy_identity: alloc::string::String,
    status: HostAuthorityStatus,
    issue: HostAuthorityIssue,
    action: super::super::GuardedAction,
    ability: alloc::string::String,
    principal_hash: alloc::string::String,
    resource_hash: alloc::string::String,
    checked_fact_count: u32,
    freshness: HostAuthorityFreshnessClass,
    replay: HostAuthorityReplayClass,
    provider_receipt_identity: alloc::string::String,
    decision_identity: alloc::string::String,
}

impl HostAuthorityDecision {
    /// Return the exact request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }
    /// Return the exact agent binding identity.
    #[must_use]
    pub fn binding_identity(&self) -> &str {
        self.binding_identity.as_str()
    }
    /// Return the provider identity.
    #[must_use]
    pub fn provider_identity(&self) -> &str {
        self.provider_identity.as_str()
    }
    /// Return the provider policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }
    /// Return the decision status.
    #[must_use]
    pub const fn status(&self) -> HostAuthorityStatus {
        self.status
    }
    /// Return the safe issue class.
    #[must_use]
    pub const fn issue(&self) -> HostAuthorityIssue {
        self.issue
    }
    /// Return the guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }
    /// Return the safe ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }
    /// Return the redacted principal hash.
    #[must_use]
    pub fn principal_hash(&self) -> &str {
        self.principal_hash.as_str()
    }
    /// Return the redacted resource hash.
    #[must_use]
    pub fn resource_hash(&self) -> &str {
        self.resource_hash.as_str()
    }
    /// Return the checked provider fact count.
    #[must_use]
    pub const fn checked_fact_count(&self) -> u32 {
        self.checked_fact_count
    }
    /// Return the freshness class.
    #[must_use]
    pub const fn freshness(&self) -> HostAuthorityFreshnessClass {
        self.freshness
    }
    /// Return the replay class.
    #[must_use]
    pub const fn replay(&self) -> HostAuthorityReplayClass {
        self.replay
    }
    /// Return the redacted provider receipt identity.
    #[must_use]
    pub fn provider_receipt_identity(&self) -> &str {
        self.provider_receipt_identity.as_str()
    }
    /// Return the complete generic decision identity.
    #[must_use]
    pub fn decision_identity(&self) -> &str {
        self.decision_identity.as_str()
    }
    /// Return true only for an exact allowed provider decision.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.status, HostAuthorityStatus::Allowed)
            && matches!(self.issue, HostAuthorityIssue::None)
    }
}

/// Build one exact metadata-only provider decision.
///
/// # Errors
///
/// Returns a stable issue when status, issue, identity, freshness, replay, or count facts conflict.
pub fn build_host_authority_decision(
    draft: HostAuthorityDecisionDraft<'_>,
) -> Result<HostAuthorityDecision, HostAuthorityIssue> {
    let is_allowed = matches!(draft.status, HostAuthorityStatus::Allowed);
    if is_allowed != matches!(draft.issue, HostAuthorityIssue::None) {
        return Err(HostAuthorityIssue::DecisionMismatch);
    }
    if is_allowed && draft.freshness != HostAuthorityFreshnessClass::Current {
        return Err(HostAuthorityIssue::DecisionMismatch);
    }
    if is_allowed && draft.replay != HostAuthorityReplayClass::Admitted {
        return Err(HostAuthorityIssue::DecisionMismatch);
    }
    if draft.policy_identity != draft.request.policy_identity()
        || draft.checked_fact_count > draft.request.bounds().checked_fact_allowance
    {
        return Err(HostAuthorityIssue::DecisionMismatch);
    }
    for identity in [
        draft.provider_identity,
        draft.principal_hash,
        draft.resource_hash,
        draft.provider_receipt_identity,
    ] {
        if !valid_identity(identity, draft.request.bounds().identity_capacity_bytes) {
            return Err(HostAuthorityIssue::InvalidIdentity);
        }
    }
    debug_assert_eq!(draft.policy_identity, draft.request.policy_identity());
    debug_assert!(draft.checked_fact_count <= draft.request.bounds().checked_fact_allowance);
    let decision_identity = hash_decision(&draft)?;
    Ok(HostAuthorityDecision {
        request_identity: alloc::string::String::from(draft.request.request_identity()),
        binding_identity: alloc::string::String::from(draft.request.binding_identity()),
        provider_identity: alloc::string::String::from(draft.provider_identity),
        policy_identity: alloc::string::String::from(draft.policy_identity),
        status: draft.status,
        issue: draft.issue,
        action: draft.request.action(),
        ability: alloc::string::String::from(draft.request.ability()),
        principal_hash: alloc::string::String::from(draft.principal_hash),
        resource_hash: alloc::string::String::from(draft.resource_hash),
        checked_fact_count: draft.checked_fact_count,
        freshness: draft.freshness,
        replay: draft.replay,
        provider_receipt_identity: alloc::string::String::from(draft.provider_receipt_identity),
        decision_identity,
    })
}

/// Result from one independent Animus host gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// The gate admits the exact request.
    Allowed,
    /// The gate denies the exact request.
    Denied,
}

/// Every independent Animus gate that remains required after provider admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequiredHostGates {
    /// Agent-definition admission gate.
    pub agent_definition: Status,
    /// Semantic role and delegate gate.
    pub semantic_role: Status,
    /// Parent authority gate.
    pub parent: Status,
    /// Task admission gate.
    pub task: Status,
    /// Model selection gate.
    pub model: Status,
    /// Tool capability gate.
    pub tool: Status,
    /// Workspace admission gate.
    pub workspace: Status,
    /// Approval or confirmation gate.
    pub approval: Status,
    /// Sandbox policy gate.
    pub sandbox: Status,
    /// Cancellation-state gate.
    pub cancellation: Status,
    /// Timeout gate.
    pub timeout: Status,
    /// Budget gate.
    pub budget: Status,
    /// Runtime authority-seal gate.
    pub runtime_authority_seal: Status,
}

impl RequiredHostGates {
    fn all_allow(self) -> bool {
        [
            self.agent_definition,
            self.semantic_role,
            self.parent,
            self.task,
            self.model,
            self.tool,
            self.workspace,
            self.approval,
            self.sandbox,
            self.cancellation,
            self.timeout,
            self.budget,
            self.runtime_authority_seal,
        ]
        .iter()
        .all(|status| *status == Status::Allowed)
    }
}

/// Effective additive authority result after every required gate intersects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectiveAuthorityDecision {
    /// Effective status.
    pub status: HostAuthorityStatus,
    /// Provider issue or local-gate denial.
    pub issue: HostAuthorityIssue,
}

/// Intersect one provider decision with every independent Animus gate.
#[must_use]
pub fn intersect_host_authority(
    provider: &HostAuthorityDecision,
    gates: RequiredHostGates,
) -> EffectiveAuthorityDecision {
    if !provider.is_allowed() {
        return EffectiveAuthorityDecision {
            status: HostAuthorityStatus::Denied,
            issue: provider.issue(),
        };
    }
    if !gates.all_allow() {
        return EffectiveAuthorityDecision {
            status: HostAuthorityStatus::Denied,
            issue: HostAuthorityIssue::LocalGateDenied,
        };
    }
    EffectiveAuthorityDecision {
        status: HostAuthorityStatus::Allowed,
        issue: HostAuthorityIssue::None,
    }
}

fn valid_identity(value: &str, max_bytes: usize) -> bool {
    let hex = value
        .strip_prefix(BLAKE3_PREFIX)
        .or_else(|| value.strip_prefix("blake3:"));
    value.len() <= max_bytes
        && hex.is_some_and(|hex| {
            hex.len() == BLAKE3_HEX_BYTES
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
}

fn hash_decision(
    draft: &HostAuthorityDecisionDraft<'_>,
) -> Result<alloc::string::String, HostAuthorityIssue> {
    debug_assert!(!draft.provider_identity.is_empty());
    debug_assert!(!draft.policy_identity.is_empty());
    let checked = draft.checked_fact_count.to_be_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(DECISION_DOMAIN);
    for field in [
        draft.request.request_identity().as_bytes(),
        draft.request.binding_identity().as_bytes(),
        draft.provider_identity.as_bytes(),
        draft.policy_identity.as_bytes(),
        draft.request.action().as_str().as_bytes(),
        draft.request.ability().as_bytes(),
        draft.principal_hash.as_bytes(),
        draft.resource_hash.as_bytes(),
        checked.as_slice(),
        draft.freshness.as_str().as_bytes(),
        draft.replay.as_str().as_bytes(),
        draft.provider_receipt_identity.as_bytes(),
        draft.issue.as_str().as_bytes(),
    ] {
        let field_bytes =
            u64::try_from(field.len()).map_err(|_| HostAuthorityIssue::InvalidBound)?;
        hasher.update(&field_bytes.to_be_bytes());
        hasher.update(field);
    }
    Ok(alloc::format!(
        "{BLAKE3_PREFIX}{}",
        hasher.finalize().to_hex()
    ))
}
