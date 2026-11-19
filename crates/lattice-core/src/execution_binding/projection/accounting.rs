//! `lattice.execution-binding-provenance.v1`: the binding a usage observation
//! belongs to. Claims are reserved quantities, never measured device use, and
//! provenance never changes reserved or charged budget.

// r[impl lattice.execution_binding.usage_provenance]
// r[impl lattice.execution_binding.usage_provenance.scenario.bound]
// r[impl lattice.execution_binding.usage_provenance.scenario.shared_allocation]

pub const PROVENANCE_SCHEMA: &str = "lattice.execution-binding-provenance.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptProvenance {
    pub schema: String,
    pub binding_identity: String,
    pub profile_revision: String,
    pub worker_incarnation: String,
    pub attempt_id: String,
    pub fence: u64,
    pub compute: ProvenanceCompute,
    pub model_routes: super::super::request::RouteDeclaration,
    pub coverage: Coverage,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "compute_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProvenanceCompute {
    /// No lease, claim or placement is fabricated.
    HostLocal {},
    ProviderAllocation(Box<ReservedAllocation>),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReservedAllocation {
    pub allocation_id: String,
    pub consumer_id: String,
    pub owner_generation: u64,
    pub placement: super::super::admission::bound::Placement,
    pub charge_cursor: u64,
    pub kind: super::super::model::AllocationKind,
    /// Opaque; the shared deployment is never charged per worker.
    pub shared_serving_ref: Option<String>,
    pub reserved_claims: Vec<ReservedClaim>,
    pub enforcement_at_bind: super::super::model::Enforcement,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReservedClaim {
    pub pool_id: String,
    pub quantity: u64,
    pub label: ClaimLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimLabel {
    Reserved,
}

/// Per-dimension coverage. Device usage stays unknown until its owner supplies
/// it; model-request usage is per-service only for served routes. Neither is
/// ever zero-filled.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub worker_process: WorkerCoverage,
    // r[impl lattice.serving_binding.usage]
    pub model_requests: super::super::serving::usage::ModelRequestCoverage,
    pub device_utilization: UnknownCoverage,
    pub unknown_owner: UnknownOwners,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerCoverage {
    HostMeasured,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCoverage {
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownOwners {
    pub model_requests: String,
    pub device_utilization: String,
}

/// Freeze provenance for one admitted binding and worker incarnation.
#[must_use]
pub fn provenance(
    admitted: &super::super::admission::bound::Admitted,
    worker_incarnation: &str,
    worker_process: WorkerCoverage,
    model_requests: super::super::serving::usage::ModelRequestCoverage,
) -> AttemptProvenance {
    AttemptProvenance {
        schema: PROVENANCE_SCHEMA.to_owned(),
        binding_identity: admitted.binding_identity.clone(),
        profile_revision: admitted.profile_revision.clone(),
        worker_incarnation: worker_incarnation.to_owned(),
        attempt_id: admitted.attempt.attempt_id.clone(),
        fence: admitted.attempt.fence,
        compute: match &admitted.compute {
            super::super::admission::bound::AdmittedCompute::HostLocal {} => ProvenanceCompute::HostLocal {},
            super::super::admission::bound::AdmittedCompute::ProviderAllocation(frozen) => {
                ProvenanceCompute::ProviderAllocation(Box::new(reserved(frozen)))
            }
        },
        model_routes: admitted.model_routes.clone(),
        coverage: Coverage {
            worker_process,
            model_requests,
            device_utilization: UnknownCoverage::Unknown,
            unknown_owner: UnknownOwners {
                model_requests: super::SERVING_OWNER.to_owned(),
                device_utilization: super::DEVICE_UTILIZATION_OWNER.to_owned(),
            },
        },
    }
}

fn reserved(frozen: &super::super::admission::bound::AllocationProvenance) -> ReservedAllocation {
    ReservedAllocation {
        allocation_id: frozen.allocation_id.clone(),
        consumer_id: frozen.consumer_id.clone(),
        owner_generation: frozen.owner_generation,
        placement: frozen.placement.clone(),
        charge_cursor: frozen.charge_cursor,
        kind: frozen.kind,
        shared_serving_ref: frozen.shared_serving_ref.clone(),
        reserved_claims: frozen
            .claims
            .iter()
            .map(|claim| ReservedClaim {
                pool_id: claim.pool_id.clone(),
                quantity: claim.quantity,
                label: ClaimLabel::Reserved,
            })
            .collect(),
        enforcement_at_bind: frozen.enforcement_at_bind,
    }
}

/// Structural validity of a stored provenance record.
#[must_use]
pub fn is_valid_provenance(record: &AttemptProvenance) -> bool {
    let has_tokens = [&record.profile_revision, &record.worker_incarnation, &record.attempt_id]
        .into_iter()
        .all(|value| super::super::is_token(value));
    let owners = &record.coverage.unknown_owner;
    let has_owners =
        owners.model_requests == super::SERVING_OWNER && owners.device_utilization == super::DEVICE_UTILIZATION_OWNER;
    record.schema == PROVENANCE_SCHEMA
        && super::super::is_reference(&record.binding_identity)
        && has_tokens
        && record.fence > 0
        && has_owners
        && super::super::serving::is_valid_coverage(&record.coverage.model_requests, &record.model_routes)
        && is_valid_compute(&record.compute)
}

fn is_valid_compute(compute: &ProvenanceCompute) -> bool {
    let ProvenanceCompute::ProviderAllocation(allocation) = compute else {
        return true;
    };
    let claims = &allocation.reserved_claims;
    let is_bounded = (1..=super::super::MAX_ITEMS).contains(&claims.len());
    let has_positive = claims.iter().all(|claim| claim.quantity > 0 && super::super::is_token(&claim.pool_id));
    is_bounded
        && has_positive
        && super::super::is_distinct(claims.iter().map(|claim| claim.pool_id.as_str()))
        && allocation.owner_generation > 0
        && allocation.charge_cursor > 0
        && allocation.kind == super::super::model::AllocationKind::Execution
}
