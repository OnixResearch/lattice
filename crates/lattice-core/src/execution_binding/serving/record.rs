//! Durable per-request evidence of one served route.
//!
//! A `lattice.model-request-dispatch.v1` intent is written before upstream
//! contact; a `lattice.model-request-observation.v1` record after the gate
//! refuses or the upstream call ends. Both bind the exact admitted binding,
//! worker incarnation, attempt, fence, route, service and request.

// r[impl lattice.serving_binding.request_observation]

pub const INTENT_SCHEMA: &str = "lattice.model-request-dispatch.v1";
pub const OBSERVATION_SCHEMA: &str = "lattice.model-request-observation.v1";

type Admitted = super::super::admission::bound::Admitted;

/// Written once, before the upstream endpoint is contacted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchIntent {
    pub schema: String,
    pub binding_identity: String,
    pub attempt_id: String,
    pub fence: u64,
    pub route_name: String,
    pub request_id: String,
    pub ordinal: u64,
    pub service_id: String,
    pub deployment_id: String,
    pub deployment_generation: u64,
    pub grant_ref: String,
    pub admission_epoch: u64,
}

/// Written once per request after the gate refused or the call ended.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema: String,
    pub binding_identity: String,
    pub profile_revision: String,
    pub worker_incarnation: String,
    pub attempt_id: String,
    pub fence: u64,
    pub route_name: String,
    pub route_identity: String,
    pub service_id: String,
    pub request_id: String,
    pub ordinal: u64,
    /// `None` when screening selected no route.
    pub served: Option<Served>,
    pub snapshot: Option<Snapshot>,
    pub admission: Admission,
    pub outcome: Outcome,
    pub usage: Usage,
    pub evidence: Evidence,
    /// Identity of the Animus `animus.serving-invocation-receipt.v1`; `None`
    /// only when no snapshot could be screened.
    pub receipt_identity: Option<String>,
    pub provider_receipt_identity: Option<String>,
}

/// The deployment, model and replica the gate selected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Served {
    pub endpoint_identity: String,
    pub deployment_id: String,
    pub deployment_generation: u64,
    pub model_id: String,
    pub configuration_ref: String,
    pub attestation: Attestation,
    pub capacity: Capacity,
    pub replica: Option<String>,
    pub is_replica_changed: bool,
    /// Serving owner's resource-consumer reference; never charged per worker.
    pub resource_consumer_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attestation {
    Claimed,
    Observed,
}

/// Capacity evidence at screening. It is never a reservation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "standing", rename_all = "snake_case", deny_unknown_fields)]
pub enum Capacity {
    Known { kind: String, value: u64 },
    OwnerDecides {},
}

/// Snapshot identity, position and status the screening used.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub identity: String,
    pub source_generation: u64,
    pub sequence: u64,
    pub status: String,
}

/// Serving-owner admission as recorded by the producer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub label: String,
    pub grant_ref: Option<String>,
    pub admission_epoch: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub class: OutcomeClass,
    /// Producer outcome label, for example `grant-revoked` or `uncertain-timed-out`.
    pub label: String,
    /// Screening exclusion, for example `not-ready:draining` or `out-of-order`.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeClass {
    NotDispatched,
    Completed,
    ResponseRejected,
    Uncertain,
}

impl OutcomeClass {
    #[must_use]
    pub const fn is_dispatched(self) -> bool {
        !matches!(self, Self::NotDispatched)
    }
}

/// Provider-reported counters or unknown; never inferred, never zero-filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub input_tokens: Counter,
    pub output_tokens: Counter,
    pub origin: UsageOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counter {
    Unknown,
    Known(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOrigin {
    ProviderReported,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Fixture,
    /// An admitted request through an embedder-supplied port reached upstream.
    LiveEndpoint,
    /// Not a fixture and no upstream contact; never live evidence.
    NoUpstreamContact,
}

impl Usage {
    pub const UNKNOWN: Self = Self {
        input_tokens: Counter::Unknown,
        output_tokens: Counter::Unknown,
        origin: UsageOrigin::Unknown,
    };

    /// Both counters are provider-reported and known.
    #[must_use]
    pub const fn is_known(self) -> bool {
        let is_reported = matches!(self.origin, UsageOrigin::ProviderReported);
        is_reported && matches!((self.input_tokens, self.output_tokens), (Counter::Known(_), Counter::Known(_)))
    }
}

/// Why a record or intent does not belong to the admitted binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Mismatch {
    Schema,
    Crossed { field: &'static str },
    UndeclaredRoute,
    Inconsistent,
}

/// The intent belongs to this binding and a declared served route.
pub fn check_intent(admitted: &Admitted, intent: &DispatchIntent) -> Result<(), Mismatch> {
    check(intent.schema == INTENT_SCHEMA, Mismatch::Schema)?;
    check_owner(admitted, (&intent.binding_identity, &intent.attempt_id, intent.fence))?;
    let route = served_route(admitted, &intent.route_name)?;
    check(route.service_id == intent.service_id, Mismatch::Crossed { field: "service_id" })?;
    debug_assert_eq!(intent.attempt_id, admitted.attempt.attempt_id);
    debug_assert_eq!(intent.fence, admitted.attempt.fence);
    Ok(())
}

/// The record belongs to this binding, route and service and is internally consistent.
pub fn check_observation(admitted: &Admitted, record: &Observation) -> Result<(), Mismatch> {
    check(record.schema == OBSERVATION_SCHEMA, Mismatch::Schema)?;
    check_owner(admitted, (&record.binding_identity, &record.attempt_id, record.fence))?;
    check(record.profile_revision == admitted.profile_revision, Mismatch::Crossed {
        field: "profile_revision",
    })?;
    let route = served_route(admitted, &record.route_name)?;
    check(route.service_id == record.service_id, Mismatch::Crossed { field: "service_id" })?;
    let is_dispatched = record.outcome.class.is_dispatched();
    let is_granted = record.admission.grant_ref.is_some() && record.admission.admission_epoch.is_some();
    let has_provider_usage = record.usage.origin == UsageOrigin::ProviderReported;
    let is_quiet = !has_provider_usage && record.provider_receipt_identity.is_none();
    let is_consistent = if is_dispatched {
        is_granted && record.served.is_some()
    } else {
        is_quiet
    };
    debug_assert_eq!(record.binding_identity, admitted.binding_identity);
    debug_assert!(super::is_producer_token(&route.service_id));
    check(is_consistent, Mismatch::Inconsistent)
}

pub(super) fn check_owner(admitted: &Admitted, owner: (&str, &str, u64)) -> Result<(), Mismatch> {
    check(owner.0 == admitted.binding_identity, Mismatch::Crossed {
        field: "binding_identity",
    })?;
    check(owner.1 == admitted.attempt.attempt_id, Mismatch::Crossed { field: "attempt_id" })?;
    check(owner.2 == admitted.attempt.fence, Mismatch::Crossed { field: "fence" })
}

fn served_route<'a>(admitted: &'a Admitted, name: &str) -> Result<&'a super::Route, Mismatch> {
    admitted
        .serving
        .as_ref()
        .and_then(|declaration| super::route(declaration, name))
        .ok_or(Mismatch::UndeclaredRoute)
}

fn check(condition: bool, mismatch: Mismatch) -> Result<(), Mismatch> {
    if condition { Ok(()) } else { Err(mismatch) }
}
