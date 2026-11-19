//! Authoritative pre-dispatch gate over serving-owner admission and a current recheck.
//!
//! A snapshot's capacity is never a reservation. Only a request-scoped grant
//! from the serving owner, checked against a current observation taken
//! immediately before provider contact, permits dispatch.

/// The request the serving owner is asked to admit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionQuery {
    pub(super) request_id: super::Token,
    pub(super) caller: super::CallerBinding,
    pub(super) route: super::RouteBinding,
    pub(super) deployment: super::DeploymentBinding,
    pub(super) model: super::ModelBinding,
    pub(super) tokens: (u32, u32),
    pub(super) policy_generation: Option<u64>,
    pub(super) selected_replica: super::ReplicaFact,
}

impl AdmissionQuery {
    /// Request identity.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }

    /// Caller identities.
    #[must_use]
    pub const fn caller(&self) -> &super::CallerBinding {
        &self.caller
    }

    /// Named route.
    #[must_use]
    pub const fn route(&self) -> &super::RouteBinding {
        &self.route
    }

    /// Deployment selected at screening time.
    #[must_use]
    pub const fn deployment(&self) -> &super::DeploymentBinding {
        &self.deployment
    }

    /// Exact model binding.
    #[must_use]
    pub const fn model(&self) -> &super::ModelBinding {
        &self.model
    }

    /// Requested `(input, output)` tokens.
    #[must_use]
    pub const fn tokens(&self) -> (u32, u32) {
        self.tokens
    }

    /// Required rollout policy generation, if any.
    #[must_use]
    pub const fn policy_generation(&self) -> Option<u64> {
        self.policy_generation
    }

    /// Replica observed at screening time.
    #[must_use]
    pub const fn selected_replica(&self) -> &super::ReplicaFact {
        &self.selected_replica
    }
}

/// Builds the admission query for one eligible route.
///
/// # Errors
///
/// Returns [`super::ContractIssue::UnsafeToken`] for an unsafe request identity.
pub fn admission_query(
    route: &super::EligibleRoute,
    caller: &super::CallerBinding,
    request_id: &str,
) -> Result<AdmissionQuery, super::ContractIssue> {
    Ok(AdmissionQuery {
        request_id: super::Token::new("request_id", request_id)?,
        caller: caller.clone(),
        route: route.route.clone(),
        deployment: route.deployment.clone(),
        model: route.model.clone(),
        tokens: (
            route.requirements.input_tokens,
            route.requirements.output_tokens,
        ),
        policy_generation: route
            .rollout
            .as_ref()
            .map(super::RolloutRequirement::policy_generation),
        selected_replica: route.replica.clone(),
    })
}

/// Borrowed request-scoped grant from the serving owner.
#[derive(Clone, Copy, Debug)]
pub struct GrantDraft<'a> {
    /// Opaque grant reference.
    pub grant_ref: &'a str,
    /// Request the grant admits.
    pub request_id: &'a str,
    /// Service the grant admits.
    pub service_id: &'a str,
    /// Deployment the grant admits.
    pub deployment_id: &'a str,
    /// Deployment generation the grant admits.
    pub deployment_generation: u64,
    /// Model the grant admits.
    pub model_id: &'a str,
    /// Configuration the grant admits.
    pub configuration_ref: &'a str,
    /// Owner admission epoch; the owner revokes outstanding grants by advancing it.
    pub admission_epoch: u64,
    /// Owner logical tick at which the grant was issued.
    pub issued_tick: u64,
    /// Exclusive owner logical tick at which the grant expires.
    pub valid_until_tick: u64,
    /// Replica the owner placed the request on, if exposed.
    pub replica: Option<&'a str>,
    /// Loaded rollout policy generation the owner admits, if any.
    pub policy_generation: Option<u64>,
}

/// Validated request-scoped grant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant {
    reference: super::Token,
    request_id: super::Token,
    service_id: super::Token,
    deployment_id: super::Token,
    deployment_generation: u64,
    model: super::ModelBinding,
    admission_epoch: u64,
    valid_until_tick: u64,
    replica: super::ReplicaFact,
    policy_generation: Option<u64>,
}

impl Grant {
    /// Validates a grant.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue`] for unsafe identities, zero generations
    /// or epochs, or an inverted validity window.
    pub fn new(draft: &GrantDraft<'_>) -> Result<Self, super::ContractIssue> {
        super::basis::window(
            "grant.valid_until_tick",
            (draft.issued_tick, draft.valid_until_tick),
        )?;
        Ok(Self {
            reference: super::Token::new("grant_ref", draft.grant_ref)?,
            request_id: super::Token::new("grant.request_id", draft.request_id)?,
            service_id: super::Token::new("grant.service_id", draft.service_id)?,
            deployment_id: super::Token::new("grant.deployment_id", draft.deployment_id)?,
            deployment_generation: super::basis::positive(
                "grant.deployment_generation",
                draft.deployment_generation,
            )?,
            model: super::ModelBinding::new(draft.model_id, draft.configuration_ref)?,
            admission_epoch: super::basis::positive(
                "grant.admission_epoch",
                draft.admission_epoch,
            )?,
            valid_until_tick: draft.valid_until_tick,
            replica: super::ReplicaFact::new(draft.replica)?,
            policy_generation: draft.policy_generation,
        })
    }

    /// Opaque grant reference.
    #[must_use]
    pub fn grant_ref(&self) -> &str {
        self.reference.as_str()
    }

    pub(super) fn reference_token(&self) -> super::Token {
        self.reference.clone()
    }

    /// Owner admission epoch.
    #[must_use]
    pub const fn admission_epoch(&self) -> u64 {
        self.admission_epoch
    }
}

/// Typed serving-owner rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerRejection {
    /// No request slot is available.
    Saturated,
    /// A hosted quota is exhausted.
    QuotaExhausted,
    /// The deployment is not ready.
    NotReady,
    /// The deployment is draining.
    Draining,
    /// The model is not loaded.
    ModelNotLoaded,
    /// The owner's policy denied the request.
    PolicyDenied,
    /// The owner gave no reason.
    Unspecified,
}

impl OwnerRejection {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Saturated => "saturated",
            Self::QuotaExhausted => "quota-exhausted",
            Self::NotReady => "not-ready",
            Self::Draining => "draining",
            Self::ModelNotLoaded => "model-not-loaded",
            Self::PolicyDenied => "policy-denied",
            Self::Unspecified => "unspecified",
        }
    }
}

/// The serving owner's reply to one admission query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnerReply {
    /// The owner granted this request.
    Granted(alloc::boxed::Box<Grant>),
    /// The owner rejected this request.
    Rejected(OwnerRejection),
    /// No admission service was reachable.
    Unavailable,
    /// The admission port failed (transport, timeout, malformed reply).
    PortIssue,
}

impl OwnerReply {
    /// Wraps a validated grant.
    #[must_use]
    pub fn granted(grant: Grant) -> Self {
        Self::Granted(alloc::boxed::Box::new(grant))
    }
}

/// Borrowed current serving observation taken immediately before provider contact.
#[derive(Clone, Copy, Debug)]
pub struct CurrentState<'a> {
    /// Current service identifier.
    pub service_id: &'a str,
    /// Current endpoint identity.
    pub endpoint_identity: &'a str,
    /// Current deployment identifier.
    pub deployment_id: &'a str,
    /// Current deployment generation.
    pub deployment_generation: u64,
    /// Currently served model.
    pub model_id: &'a str,
    /// Currently served configuration.
    pub configuration_ref: &'a str,
    /// Current readiness.
    pub readiness: super::Readiness,
    /// Current owner admission epoch.
    pub admission_epoch: u64,
    /// Current owner logical tick; `None` is unknown.
    pub now_tick: Option<u64>,
    /// Currently loaded rollout policy generation, if established.
    pub policy_generation: Option<u64>,
}

/// The host's current-state reading.
#[derive(Clone, Copy, Debug)]
pub enum CurrentReading<'a> {
    /// The host observed the current state.
    Observed(CurrentState<'a>),
    /// No recheck service was reachable.
    Unavailable,
    /// The recheck port failed.
    PortIssue,
}

/// Inputs to the pure dispatch gate.
#[derive(Clone, Copy, Debug)]
pub struct GateInput<'a> {
    /// Capacity-aware profile.
    pub profile: &'a super::Profile,
    /// Admission query.
    pub query: &'a AdmissionQuery,
    /// Serving-owner reply.
    pub owner: &'a OwnerReply,
    /// Current-state recheck.
    pub current: CurrentReading<'a>,
    /// Existing exact-model, profile, privacy, budget, authority, remote,
    /// deadline, cancellation, and transport decisions.
    pub existing: crate::mesh::DispatchAdmission,
}

/// Definite pre-dispatch rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateRejection {
    /// An existing policy, authority, or control decision denied dispatch.
    ExistingPolicy(crate::mesh::DispatchDecision),
    /// The serving owner rejected the request.
    OwnerRejected(OwnerRejection),
    /// The grant names another request.
    GrantForOtherRequest,
    /// The grant names another service or deployment.
    GrantTargetMismatch,
    /// The deployment was replaced between selection and dispatch.
    GenerationReplaced,
    /// The endpoint changed between selection and dispatch.
    EndpointReplaced,
    /// Another semantic model would serve the request.
    ModelSubstituted,
    /// The model's configuration changed.
    ConfigurationChanged,
    /// The deployment is no longer ready.
    NotReadyAtDispatch(super::Readiness),
    /// The grant expired.
    GrantExpired,
    /// The owner revoked the grant by advancing its admission epoch.
    GrantRevoked,
    /// The replica changed under a pinned profile.
    ReplicaChangeDenied,
    /// The admitted or loaded policy generation differs from the requirement.
    PolicyGenerationMismatch,
    /// The owner could not establish the required policy generation.
    PolicyGenerationUnestablished,
}

impl GateRejection {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExistingPolicy(decision) => existing_label(decision),
            Self::OwnerRejected(reason) => reason.as_str(),
            Self::GrantForOtherRequest => "grant-for-other-request",
            Self::GrantTargetMismatch => "grant-target-mismatch",
            Self::GenerationReplaced => "generation-replaced",
            Self::EndpointReplaced => "endpoint-replaced",
            Self::ModelSubstituted => "model-substituted",
            Self::ConfigurationChanged => "configuration-changed",
            Self::NotReadyAtDispatch(_) => "not-ready-at-dispatch",
            Self::GrantExpired => "grant-expired",
            Self::GrantRevoked => "grant-revoked",
            Self::ReplicaChangeDenied => "replica-change-denied",
            Self::PolicyGenerationMismatch => "policy-generation-mismatch",
            Self::PolicyGenerationUnestablished => "policy-generation-unestablished",
        }
    }
}

const fn existing_label(decision: crate::mesh::DispatchDecision) -> &'static str {
    match decision {
        crate::mesh::DispatchDecision::Admitted => "existing-admitted",
        crate::mesh::DispatchDecision::ExactModelDenied => "existing-exact-model-denied",
        crate::mesh::DispatchDecision::PolicyDenied => "existing-policy-denied",
        crate::mesh::DispatchDecision::AuthorityDenied => "existing-authority-denied",
        crate::mesh::DispatchDecision::RemoteDenied => "existing-remote-denied",
        crate::mesh::DispatchDecision::ControlUnavailable => "existing-control-unavailable",
        crate::mesh::DispatchDecision::MissingTransport => "existing-missing-transport",
    }
}

/// Pre-dispatch block on unknown or unavailable facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateBlock {
    /// The admission service was unavailable.
    AdmissionUnavailable,
    /// The admission port failed.
    AdmissionPortIssue,
    /// The recheck service was unavailable.
    RecheckUnavailable,
    /// The recheck port failed.
    RecheckPortIssue,
    /// The owner's current tick was unknown, so grant expiry is unknown.
    ClockUnknown,
}

impl GateBlock {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdmissionUnavailable => "admission-unavailable",
            Self::AdmissionPortIssue => "admission-port-issue",
            Self::RecheckUnavailable => "recheck-unavailable",
            Self::RecheckPortIssue => "recheck-port-issue",
            Self::ClockUnknown => "clock-unknown",
        }
    }
}

/// Facts that permitted one dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchPermit {
    pub(super) grant_ref: super::Token,
    pub(super) admission_epoch: u64,
    pub(super) replica: super::ReplicaFact,
    pub(super) is_replica_changed: bool,
}

impl DispatchPermit {
    /// Opaque grant reference.
    #[must_use]
    pub fn grant_ref(&self) -> &str {
        self.grant_ref.as_str()
    }

    /// Owner admission epoch of the grant.
    #[must_use]
    pub const fn admission_epoch(&self) -> u64 {
        self.admission_epoch
    }

    /// Replica the owner placed the request on.
    #[must_use]
    pub const fn replica(&self) -> &super::ReplicaFact {
        &self.replica
    }

    /// Whether the replica differs from the one observed at selection.
    #[must_use]
    pub const fn is_replica_changed(&self) -> bool {
        self.is_replica_changed
    }
}

/// Gate verdict. Only `Admitted` permits provider contact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GateVerdict {
    /// Dispatch may proceed.
    Admitted(DispatchPermit),
    /// Dispatch is definitely refused.
    Rejected(GateRejection),
    /// Dispatch is blocked on unknown or unavailable facts.
    Blocked(GateBlock),
}

/// Decides whether one admitted request may reach the provider.
///
/// Order: existing decisions, owner reply, grant binding, current recheck,
/// grant currentness, policy generation, then the replica rule.
#[must_use]
pub fn gate_dispatch(input: &GateInput<'_>) -> GateVerdict {
    match decide(input) {
        Ok(permit) => GateVerdict::Admitted(permit),
        Err(Refusal::Rejected(rejection)) => GateVerdict::Rejected(rejection),
        Err(Refusal::Blocked(block)) => GateVerdict::Blocked(block),
    }
}

enum Refusal {
    Rejected(GateRejection),
    Blocked(GateBlock),
}

fn decide(input: &GateInput<'_>) -> Result<DispatchPermit, Refusal> {
    let existing = crate::mesh::admit_dispatch(&input.existing);
    if existing != crate::mesh::DispatchDecision::Admitted {
        return Err(Refusal::Rejected(GateRejection::ExistingPolicy(existing)));
    }
    let grant = granted(input.owner)?;
    check_grant(input.query, grant).map_err(Refusal::Rejected)?;
    let current = observed(input.current)?;
    check_current(input.query, &current).map_err(Refusal::Rejected)?;
    check_currentness(grant, &current)?;
    let required = input.query.policy_generation;
    check_policy_generation(required, grant.policy_generation).map_err(Refusal::Rejected)?;
    check_policy_generation(required, current.policy_generation).map_err(Refusal::Rejected)?;
    debug_assert_eq!(grant.request_id, input.query.request_id);
    debug_assert_eq!(grant.admission_epoch, current.admission_epoch);
    permit(input.profile, input.query, grant).map_err(Refusal::Rejected)
}

fn granted(owner: &OwnerReply) -> Result<&Grant, Refusal> {
    match owner {
        OwnerReply::Granted(grant) => Ok(grant),
        OwnerReply::Rejected(reason) => {
            Err(Refusal::Rejected(GateRejection::OwnerRejected(*reason)))
        }
        OwnerReply::Unavailable => Err(Refusal::Blocked(GateBlock::AdmissionUnavailable)),
        OwnerReply::PortIssue => Err(Refusal::Blocked(GateBlock::AdmissionPortIssue)),
    }
}

fn observed(current: CurrentReading<'_>) -> Result<CurrentState<'_>, Refusal> {
    match current {
        CurrentReading::Observed(state) => Ok(state),
        CurrentReading::Unavailable => Err(Refusal::Blocked(GateBlock::RecheckUnavailable)),
        CurrentReading::PortIssue => Err(Refusal::Blocked(GateBlock::RecheckPortIssue)),
    }
}

fn check_currentness(grant: &Grant, current: &CurrentState<'_>) -> Result<(), Refusal> {
    let now_tick = current
        .now_tick
        .ok_or(Refusal::Blocked(GateBlock::ClockUnknown))?;
    if now_tick >= grant.valid_until_tick {
        return Err(Refusal::Rejected(GateRejection::GrantExpired));
    }
    if grant.admission_epoch != current.admission_epoch {
        return Err(Refusal::Rejected(GateRejection::GrantRevoked));
    }
    Ok(())
}

fn check_grant(query: &AdmissionQuery, grant: &Grant) -> Result<(), GateRejection> {
    if grant.request_id != query.request_id {
        return Err(GateRejection::GrantForOtherRequest);
    }
    let is_same_target = grant.service_id == query.deployment.service_id
        && grant.deployment_id == query.deployment.deployment_id;
    if !is_same_target {
        return Err(GateRejection::GrantTargetMismatch);
    }
    if grant.deployment_generation != query.deployment.deployment_generation {
        return Err(GateRejection::GenerationReplaced);
    }
    check_model(
        &query.model,
        (grant.model.model_id(), grant.model.configuration_ref()),
    )
}

fn check_current(query: &AdmissionQuery, current: &CurrentState<'_>) -> Result<(), GateRejection> {
    let deployment = &query.deployment;
    if current.service_id != deployment.service_id() {
        return Err(GateRejection::GrantTargetMismatch);
    }
    if current.endpoint_identity != deployment.endpoint_identity() {
        return Err(GateRejection::EndpointReplaced);
    }
    let is_same_generation = current.deployment_id == deployment.deployment_id()
        && current.deployment_generation == deployment.deployment_generation;
    if !is_same_generation {
        return Err(GateRejection::GenerationReplaced);
    }
    check_model(&query.model, (current.model_id, current.configuration_ref))?;
    if current.readiness != super::Readiness::Ready {
        return Err(GateRejection::NotReadyAtDispatch(current.readiness));
    }
    Ok(())
}

/// Compares an `(model_id, configuration_ref)` pair with the admitted binding.
fn check_model(expected: &super::ModelBinding, model: (&str, &str)) -> Result<(), GateRejection> {
    if expected.matches(model) {
        return Ok(());
    }
    if model.0 != expected.model_id() {
        return Err(GateRejection::ModelSubstituted);
    }
    Err(GateRejection::ConfigurationChanged)
}

fn check_policy_generation(
    required: Option<u64>,
    established: Option<u64>,
) -> Result<(), GateRejection> {
    match (required, established) {
        (None, _) => Ok(()),
        (Some(_), None) => Err(GateRejection::PolicyGenerationUnestablished),
        (Some(required), Some(established)) if required == established => Ok(()),
        (Some(_), Some(_)) => Err(GateRejection::PolicyGenerationMismatch),
    }
}

fn permit(
    profile: &super::Profile,
    query: &AdmissionQuery,
    grant: &Grant,
) -> Result<DispatchPermit, GateRejection> {
    let is_replica_changed = match (&query.selected_replica, &grant.replica) {
        (super::ReplicaFact::Observed(selected), super::ReplicaFact::Observed(placed)) => {
            selected != placed
        }
        (
            super::ReplicaFact::Unknown | super::ReplicaFact::Observed(_),
            super::ReplicaFact::Unknown,
        )
        | (super::ReplicaFact::Unknown, super::ReplicaFact::Observed(_)) => false,
    };
    let is_pinned = profile.replica_rule() == super::ReplicaRule::Pinned;
    if is_replica_changed && is_pinned {
        return Err(GateRejection::ReplicaChangeDenied);
    }
    debug_assert!(grant.admission_epoch > 0);
    debug_assert!(!is_replica_changed || matches!(grant.replica, super::ReplicaFact::Observed(_)));
    Ok(DispatchPermit {
        grant_ref: grant.reference.clone(),
        admission_epoch: grant.admission_epoch,
        replica: grant.replica.clone(),
        is_replica_changed,
    })
}
