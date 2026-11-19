//! Metadata-only per-invocation receipts with usage provenance.
//!
//! Every receipt field is an enum, a count, a logical tick, a validated opaque
//! token, or a canonical reference. No prompt, response, credential, header,
//! URL, path, or raw provider payload can be represented.

/// Whether the receipt came from deterministic fixtures or a live endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceLabel {
    /// Deterministic injected services; establishes no live behavior.
    Fixture,
    /// A selected live endpoint.
    LiveEndpoint,
}

impl EvidenceLabel {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::LiveEndpoint => "live-endpoint",
        }
    }
}

/// Who reported model usage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageOrigin {
    /// The provider reported the counters in its response.
    ProviderReported,
    /// No usage was reported.
    Unknown,
}

impl UsageOrigin {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderReported => "provider-reported",
            Self::Unknown => "unknown",
        }
    }
}

/// Per-request model usage. Missing counters stay unknown, never zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    /// Input tokens.
    pub input_tokens: super::Counter,
    /// Output tokens.
    pub output_tokens: super::Counter,
    /// Who reported the counters.
    pub origin: UsageOrigin,
}

impl Usage {
    /// Usage that was not reported.
    pub const UNKNOWN: Self = Self {
        input_tokens: super::Counter::Unknown,
        output_tokens: super::Counter::Unknown,
        origin: UsageOrigin::Unknown,
    };
}

/// Bounded response facts the injected transport extracted for the core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseFacts {
    /// Reported usage.
    pub usage: Usage,
    /// Rollout policy generation the response reports, if any.
    pub policy_generation: Option<u64>,
}

impl ResponseFacts {
    /// Facts from a response that reported nothing.
    pub const UNREPORTED: Self = Self {
        usage: Usage::UNKNOWN,
        policy_generation: None,
    };
}

/// Response policy-generation provenance, propagated without rewriting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyGenerationProvenance {
    /// The request required no policy generation.
    NotRequired,
    /// The response reported the required generation.
    Matched(u64),
    /// The response reported no generation.
    Missing,
    /// The response reported another generation.
    Crossed {
        /// Required generation.
        required: u64,
        /// Reported generation.
        observed: u64,
    },
}

/// Response identity provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponseProvenance {
    /// Exact response-model fidelity.
    pub model: crate::mesh::ResponseModelDecision,
    /// Policy-generation provenance.
    pub policy_generation: PolicyGenerationProvenance,
}

/// Classifies the response's reported identities against the query.
#[must_use]
pub fn response_provenance(
    query: &super::AdmissionQuery,
    response_model: Option<&str>,
    facts: &ResponseFacts,
) -> ResponseProvenance {
    let model = crate::mesh::validate_response_model(query.model().model_id(), response_model);
    let policy_generation = match (query.policy_generation(), facts.policy_generation) {
        (None, _) => PolicyGenerationProvenance::NotRequired,
        (Some(_), None) => PolicyGenerationProvenance::Missing,
        (Some(required), Some(observed)) if required == observed => {
            PolicyGenerationProvenance::Matched(observed)
        }
        (Some(required), Some(observed)) => {
            PolicyGenerationProvenance::Crossed { required, observed }
        }
    };
    ResponseProvenance {
        model,
        policy_generation,
    }
}

impl ResponseProvenance {
    /// The rejection this provenance requires, if any.
    #[must_use]
    pub const fn rejection(self) -> Option<ResponseRejection> {
        match (self.model, self.policy_generation) {
            (crate::mesh::ResponseModelDecision::Substituted, _) => {
                Some(ResponseRejection::ModelSubstituted)
            }
            (crate::mesh::ResponseModelDecision::Missing, _) => {
                Some(ResponseRejection::ModelMissing)
            }
            (_, PolicyGenerationProvenance::Missing) => {
                Some(ResponseRejection::PolicyGenerationMissing)
            }
            (_, PolicyGenerationProvenance::Crossed { .. }) => {
                Some(ResponseRejection::PolicyGenerationCrossed)
            }
            (
                crate::mesh::ResponseModelDecision::Exact,
                PolicyGenerationProvenance::NotRequired | PolicyGenerationProvenance::Matched(_),
            ) => None,
        }
    }
}

/// Why a dispatched response was not accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseRejection {
    /// The response reported another model.
    ModelSubstituted,
    /// The response reported no model.
    ModelMissing,
    /// The response reported no policy generation where one was required.
    PolicyGenerationMissing,
    /// The response reported another policy generation.
    PolicyGenerationCrossed,
    /// The response exceeded the bounded size.
    Oversized,
}

impl ResponseRejection {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelSubstituted => "response-model-substituted",
            Self::ModelMissing => "response-model-missing",
            Self::PolicyGenerationMissing => "response-generation-missing",
            Self::PolicyGenerationCrossed => "response-generation-crossed",
            Self::Oversized => "response-oversized",
        }
    }
}

/// Effects of an uncertain dispatch may have happened; never replay it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UncertainCause {
    /// The transport or provider failed after contact.
    TransportFailed,
    /// The caller deadline expired after contact.
    TimedOut,
    /// The caller cancelled after contact.
    Cancelled,
}

impl UncertainCause {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TransportFailed => "uncertain-transport-failed",
            Self::TimedOut => "uncertain-timed-out",
            Self::Cancelled => "uncertain-cancelled",
        }
    }
}

/// Why no provider contact happened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotDispatched {
    /// Screening left no eligible route.
    NoEligibleRoute,
    /// Existing recipe selection denied every eligible route.
    SelectionDenied,
    /// The gate definitely refused dispatch.
    Rejected(super::GateRejection),
    /// The gate blocked on unknown or unavailable facts.
    Blocked(super::GateBlock),
}

/// Invocation outcome, separating pre-dispatch refusal from post-contact uncertainty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    /// No provider contact happened.
    NotDispatched(NotDispatched),
    /// The provider answered with exact identities.
    Completed,
    /// The provider answered, but the response was not accepted.
    ResponseRejected(ResponseRejection),
    /// Provider effects may have happened; the result is unknown.
    Uncertain(UncertainCause),
}

impl DispatchOutcome {
    /// Whether the provider may have been contacted.
    #[must_use]
    pub const fn is_dispatched(self) -> bool {
        match self {
            Self::NotDispatched(_) => false,
            Self::Completed | Self::ResponseRejected(_) | Self::Uncertain(_) => true,
        }
    }

    /// Outcome class: `not-dispatched`, `completed`, `response-rejected`, or `uncertain`.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::NotDispatched(_) => "not-dispatched",
            Self::Completed => "completed",
            Self::ResponseRejected(_) => "response-rejected",
            Self::Uncertain(_) => "uncertain",
        }
    }

    /// Stable reason label within the class.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotDispatched(NotDispatched::NoEligibleRoute) => "no-eligible-route",
            Self::NotDispatched(NotDispatched::SelectionDenied) => "selection-denied",
            Self::NotDispatched(NotDispatched::Rejected(rejection)) => rejection.as_str(),
            Self::NotDispatched(NotDispatched::Blocked(block)) => block.as_str(),
            Self::Completed => "completed",
            Self::ResponseRejected(rejection) => rejection.as_str(),
            Self::Uncertain(cause) => cause.as_str(),
        }
    }
}

/// Serving-owner admission as recorded on the receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionRecord {
    /// No admission was requested because no route was selected.
    NotRequested,
    /// The owner granted the request.
    Granted {
        /// Opaque grant reference.
        grant_ref: super::Token,
        /// Owner admission epoch.
        admission_epoch: u64,
    },
    /// The owner rejected the request.
    Rejected(super::OwnerRejection),
    /// The admission service was unavailable.
    Unavailable,
    /// The admission port failed.
    PortIssue,
}

impl AdmissionRecord {
    /// Records an owner reply.
    #[must_use]
    pub fn from_reply(reply: &super::OwnerReply) -> Self {
        match reply {
            super::OwnerReply::Granted(grant) => Self::Granted {
                grant_ref: grant.reference_token(),
                admission_epoch: grant.admission_epoch(),
            },
            super::OwnerReply::Rejected(reason) => Self::Rejected(*reason),
            super::OwnerReply::Unavailable => Self::Unavailable,
            super::OwnerReply::PortIssue => Self::PortIssue,
        }
    }

    /// Stable label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NotRequested => "not-requested",
            Self::Granted { .. } => "granted",
            Self::Rejected(reason) => reason.as_str(),
            Self::Unavailable => "unavailable",
            Self::PortIssue => "port-issue",
        }
    }
}

/// Borrowed receipt inputs.
#[derive(Clone, Copy, Debug)]
pub struct ReceiptInput<'a> {
    /// Fixture or live-endpoint label.
    pub evidence: EvidenceLabel,
    /// Request identity.
    pub request_id: &'a str,
    /// Caller identities.
    pub caller: &'a super::CallerBinding,
    /// Screening behind the selection.
    pub screening: &'a super::Screening,
    /// Selected route, if any.
    pub selected: Option<&'a super::EligibleRoute>,
    /// Serving-owner admission.
    pub admission: &'a AdmissionRecord,
    /// Gate permit when dispatch was admitted.
    pub permit: Option<&'a super::DispatchPermit>,
    /// Outcome.
    pub outcome: DispatchOutcome,
    /// Response provenance when the provider answered.
    pub response: Option<ResponseProvenance>,
    /// Model usage.
    pub usage: Usage,
    /// Existing safe provider receipt identity, for example a Mesh receipt.
    pub provider_receipt_identity: Option<&'a str>,
}

/// Metadata-only invocation receipt keyed to the caller's named route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationReceipt {
    identity: super::ContentRef,
    evidence: EvidenceLabel,
    request_id: super::Token,
    caller: super::CallerBinding,
    profile_identity: super::ContentRef,
    snapshot_identity: super::ContentRef,
    snapshot_position: (u64, u64),
    snapshot_status: super::SnapshotStatus,
    screened_counts: (u32, u32),
    selected: Option<super::EligibleRoute>,
    admission: AdmissionRecord,
    replica: super::ReplicaFact,
    is_replica_changed: bool,
    outcome: DispatchOutcome,
    response: Option<ResponseProvenance>,
    usage: Usage,
    provider_receipt_identity: Option<super::Token>,
}

/// Projects a consistent metadata-only receipt.
///
/// # Errors
///
/// Returns [`super::ContractIssue`] for an unsafe request or provider-receipt
/// identity, or [`super::ContractIssue::InconsistentReceipt`] when facts
/// contradict each other: dispatch without a grant, permit, or selected route,
/// a permit, usage, response, or provider receipt without dispatch, or
/// completion without exact response identities.
pub fn project_receipt(
    input: &ReceiptInput<'_>,
) -> Result<InvocationReceipt, super::ContractIssue> {
    check_consistency(input)?;
    let provider_receipt_identity = match input.provider_receipt_identity {
        None => None,
        Some(value) => Some(super::Token::new("provider_receipt_identity", value)?),
    };
    let screened_counts = screened_counts(input.screening)?;
    let mut receipt = InvocationReceipt {
        identity: super::basis::Framer::new(super::RECEIPT_DOMAIN).finish(),
        evidence: input.evidence,
        request_id: super::Token::new("request_id", input.request_id)?,
        caller: input.caller.clone(),
        profile_identity: input.screening.profile_identity.clone(),
        snapshot_identity: input.screening.snapshot_identity.clone(),
        snapshot_position: input.screening.snapshot_position,
        snapshot_status: input.screening.status,
        screened_counts,
        selected: input.selected.cloned(),
        admission: input.admission.clone(),
        replica: input
            .permit
            .map_or(super::ReplicaFact::Unknown, |permit| permit.replica.clone()),
        is_replica_changed: input
            .permit
            .is_some_and(super::DispatchPermit::is_replica_changed),
        outcome: input.outcome,
        response: input.response,
        usage: input.usage,
        provider_receipt_identity,
    };
    receipt.identity = identity_of(&receipt);
    debug_assert!(receipt.identity.as_str().starts_with("blake3:"));
    debug_assert_eq!(receipt.outcome, input.outcome);
    Ok(receipt)
}

fn screened_counts(screening: &super::Screening) -> Result<(u32, u32), super::ContractIssue> {
    let eligible = u32::try_from(screening.eligible.len())
        .map_err(|_| super::ContractIssue::TooMany { field: "eligible" })?;
    let excluded =
        u32::try_from(screening.exclusions.len()).map_err(|_| super::ContractIssue::TooMany {
            field: "exclusions",
        })?;
    Ok((eligible, excluded))
}

fn check_consistency(input: &ReceiptInput<'_>) -> Result<(), super::ContractIssue> {
    let is_consistent = match input.outcome {
        DispatchOutcome::NotDispatched(_) => is_quiet(input),
        DispatchOutcome::Completed => {
            is_permitted(input)
                && input
                    .response
                    .is_some_and(|response| response.rejection().is_none())
        }
        DispatchOutcome::ResponseRejected(_) | DispatchOutcome::Uncertain(_) => is_permitted(input),
    };
    if is_consistent {
        return Ok(());
    }
    Err(super::ContractIssue::InconsistentReceipt)
}

/// No permit, response, reported usage, or provider receipt without dispatch.
fn is_quiet(input: &ReceiptInput<'_>) -> bool {
    let is_from_provider = input.usage.origin == UsageOrigin::ProviderReported;
    input.permit.is_none()
        && input.response.is_none()
        && !is_from_provider
        && input.provider_receipt_identity.is_none()
}

/// Dispatch requires a selected route, a grant, and a gate permit.
fn is_permitted(input: &ReceiptInput<'_>) -> bool {
    let is_granted = matches!(input.admission, AdmissionRecord::Granted { .. });
    input.permit.is_some() && input.selected.is_some() && is_granted
}

impl InvocationReceipt {
    /// BLAKE3 receipt identity (`animus-serving-receipt-v1`).
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Fixture or live-endpoint label.
    #[must_use]
    pub const fn evidence(&self) -> EvidenceLabel {
        self.evidence
    }

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

    /// Profile identity.
    #[must_use]
    pub fn profile_identity(&self) -> &str {
        self.profile_identity.as_str()
    }

    /// Snapshot identity.
    #[must_use]
    pub fn snapshot_identity(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    /// Snapshot `(source_generation, sequence)`.
    #[must_use]
    pub const fn snapshot_position(&self) -> (u64, u64) {
        self.snapshot_position
    }

    /// Snapshot freshness at screening.
    #[must_use]
    pub const fn snapshot_status(&self) -> super::SnapshotStatus {
        self.snapshot_status
    }

    /// Eligible and excluded route counts.
    #[must_use]
    pub const fn screened_counts(&self) -> (u32, u32) {
        self.screened_counts
    }

    /// Selected route, deployment, model, attestation, and capacity evidence.
    #[must_use]
    pub const fn selected(&self) -> Option<&super::EligibleRoute> {
        self.selected.as_ref()
    }

    /// Serving-owner admission.
    #[must_use]
    pub const fn admission(&self) -> &AdmissionRecord {
        &self.admission
    }

    /// Replica the owner placed the request on.
    #[must_use]
    pub const fn replica(&self) -> &super::ReplicaFact {
        &self.replica
    }

    /// Whether the replica changed since selection.
    #[must_use]
    pub const fn is_replica_changed(&self) -> bool {
        self.is_replica_changed
    }

    /// Outcome.
    #[must_use]
    pub const fn outcome(&self) -> DispatchOutcome {
        self.outcome
    }

    /// Response provenance.
    #[must_use]
    pub const fn response(&self) -> Option<ResponseProvenance> {
        self.response
    }

    /// Model usage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }

    /// Existing safe provider receipt identity.
    #[must_use]
    pub fn provider_receipt_identity(&self) -> Option<&str> {
        self.provider_receipt_identity
            .as_ref()
            .map(super::Token::as_str)
    }
}

fn identity_of(receipt: &InvocationReceipt) -> super::ContentRef {
    debug_assert!(!receipt.request_id.as_str().is_empty());
    debug_assert!(receipt.selected.is_some() || !receipt.outcome.is_dispatched());
    let mut framer = super::basis::Framer::new(super::RECEIPT_DOMAIN);
    frame_header(&mut framer, receipt);
    frame_selected(&mut framer, receipt.selected.as_ref());
    framer.text(receipt.admission.as_str());
    if let AdmissionRecord::Granted {
        grant_ref,
        admission_epoch,
    } = &receipt.admission
    {
        framer.text(grant_ref.as_str());
        framer.number(*admission_epoch);
    }
    framer.text(receipt.replica.label());
    framer.text(if receipt.is_replica_changed {
        "changed"
    } else {
        "same"
    });
    framer.text(receipt.outcome.class());
    framer.text(receipt.outcome.as_str());
    frame_response(&mut framer, receipt.response);
    frame_usage(&mut framer, receipt.usage);
    framer.optional(
        receipt
            .provider_receipt_identity
            .as_ref()
            .map(super::Token::as_str),
    );
    for non_claim in super::RECEIPT_NON_CLAIMS {
        framer.text(non_claim);
    }
    framer.finish()
}

fn frame_header(framer: &mut super::basis::Framer, receipt: &InvocationReceipt) {
    framer.text(super::RECEIPT_SCHEMA);
    framer.text(receipt.evidence.as_str());
    framer.text(receipt.request_id.as_str());
    framer.text(receipt.caller.agent_id());
    framer.text(receipt.caller.session_id());
    framer.text(receipt.caller.attempt_id());
    framer.text(receipt.profile_identity.as_str());
    framer.text(receipt.snapshot_identity.as_str());
    framer.number(receipt.snapshot_position.0);
    framer.number(receipt.snapshot_position.1);
    framer.text(receipt.snapshot_status.as_str());
    framer.number(u64::from(receipt.screened_counts.0));
    framer.number(u64::from(receipt.screened_counts.1));
}

fn frame_selected(framer: &mut super::basis::Framer, selected: Option<&super::EligibleRoute>) {
    let Some(route) = selected else {
        framer.text("no-route");
        return;
    };
    framer.text(route.route.route_name());
    framer.optional(route.route.route_ref());
    framer.text(route.deployment.service_id());
    framer.text(route.deployment.endpoint_identity());
    framer.text(route.deployment.deployment_id());
    framer.number(route.deployment.deployment_generation());
    framer.text(route.model.model_id());
    framer.text(route.model.configuration_ref());
    framer.text(route.attestation.as_str());
    frame_standing(framer, route.standing);
    framer.text(route.replica.label());
    framer.optional(route.resource_consumer_ref());
}

fn frame_standing(framer: &mut super::basis::Framer, standing: super::CapacityStanding) {
    match standing {
        super::CapacityStanding::Known { kind, value } => {
            framer.text(kind.as_str());
            framer.number(value);
        }
        super::CapacityStanding::UnknownOwnerDecides => framer.text("owner-decides"),
    }
}

/// Stable response-model label.
#[must_use]
pub const fn response_model_label(model: crate::mesh::ResponseModelDecision) -> &'static str {
    match model {
        crate::mesh::ResponseModelDecision::Exact => "exact",
        crate::mesh::ResponseModelDecision::Substituted => "substituted",
        crate::mesh::ResponseModelDecision::Missing => "missing",
    }
}

fn frame_response(framer: &mut super::basis::Framer, response: Option<ResponseProvenance>) {
    let Some(response) = response else {
        framer.text("no-response");
        return;
    };
    debug_assert!(
        response.rejection().is_some()
            || response.model == crate::mesh::ResponseModelDecision::Exact
    );
    debug_assert!(!response_model_label(response.model).is_empty());
    framer.text(response_model_label(response.model));
    match response.policy_generation {
        PolicyGenerationProvenance::NotRequired => framer.text("generation-not-required"),
        PolicyGenerationProvenance::Matched(generation) => {
            framer.text("generation-matched");
            framer.number(generation);
        }
        PolicyGenerationProvenance::Missing => framer.text("generation-missing"),
        PolicyGenerationProvenance::Crossed { required, observed } => {
            framer.text("generation-crossed");
            framer.number(required);
            framer.number(observed);
        }
    }
}

fn frame_usage(framer: &mut super::basis::Framer, usage: Usage) {
    framer.text(usage.origin.as_str());
    for counter in [usage.input_tokens, usage.output_tokens] {
        match counter {
            super::Counter::Unknown => framer.text("unknown"),
            super::Counter::Known(value) => framer.number(value),
        }
    }
}
