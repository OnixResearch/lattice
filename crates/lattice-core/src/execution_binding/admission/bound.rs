//! The exact admitted binding persisted before dispatch.

pub const ADMITTED_SCHEMA: &str = "lattice.admitted-execution-binding.v1";

/// Profile revision, attempt, compute provenance and model routes of one
/// admitted attempt. It grants no tool, endpoint, credential or compute.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admitted {
    pub schema: String,
    pub binding_identity: String,
    pub profile_revision: String,
    pub attempt: AttemptIdentity,
    pub compute: AdmittedCompute,
    pub model_routes: super::super::request::RouteDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttemptIdentity {
    pub work_id: String,
    pub attempt_id: String,
    pub fence: u64,
    pub envelope_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "compute_kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AdmittedCompute {
    /// No allocation, claims or placement exist; nothing is fabricated.
    HostLocal {},
    ProviderAllocation(Box<AllocationProvenance>),
}

/// Provider allocation facts frozen at admission. Producer record references,
/// the grant reference and activation evidence are not retained here.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllocationProvenance {
    pub allocation_id: String,
    pub consumer_id: String,
    pub owner_generation: u64,
    pub placement: Placement,
    pub charge_cursor: u64,
    pub charge_at: u64,
    pub kind: super::super::model::AllocationKind,
    pub view_id: String,
    pub granularity: super::super::model::Granularity,
    /// This allocation's own claims only; serving-deployment claims never appear.
    pub claims: Vec<super::super::model::Claim>,
    /// Opaque reference; never read, reserved or charged by Lattice.
    pub shared_serving_ref: Option<String>,
    pub port_id: String,
    pub enforcement_at_bind: super::super::model::Enforcement,
    pub admission_sequence: u64,
}

/// Placement generation: any change requires a new attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    pub host_id: String,
    pub host_incarnation: String,
    pub device_incarnation: String,
    pub topology_generation: u64,
}

/// Recompute the `lattice-execution-binding-v1` identity of an admitted binding.
// r[impl lattice.execution_binding.identity]
pub fn identity(admitted: &Admitted) -> Result<String, content_identity_core::IdentityError> {
    let routes = match &admitted.model_routes {
        super::super::request::RouteDeclaration::NoModel {} => None,
        super::super::request::RouteDeclaration::Routes { routes } => Some(
            routes
                .iter()
                .map(|route| crate::content_identity::RouteInput {
                    name: &route.name,
                    route_identity: &route.route_identity,
                })
                .collect::<Vec<_>>(),
        ),
    };
    debug_assert_eq!(
        routes.is_some(),
        matches!(admitted.model_routes, super::super::request::RouteDeclaration::Routes { .. })
    );
    let input = |allocation| crate::content_identity::ExecutionBindingInput {
        profile_revision: &admitted.profile_revision,
        work_id: &admitted.attempt.work_id,
        attempt_id: &admitted.attempt.attempt_id,
        fence: admitted.attempt.fence,
        envelope_identity: &admitted.attempt.envelope_identity,
        allocation,
        routes: routes.as_deref(),
    };
    match &admitted.compute {
        AdmittedCompute::HostLocal {} => crate::content_identity::execution_binding(input(None)),
        AdmittedCompute::ProviderAllocation(provenance) => {
            let claims = provenance
                .claims
                .iter()
                .map(|claim| crate::content_identity::ClaimInput {
                    pool_id: &claim.pool_id,
                    quantity: claim.quantity,
                })
                .collect::<Vec<_>>();
            let allocation = allocation_input(provenance, &claims);
            debug_assert_eq!(claims.len(), provenance.claims.len());
            crate::content_identity::execution_binding(input(Some(&allocation)))
        }
    }
}

fn allocation_input<'a>(
    provenance: &'a AllocationProvenance,
    claims: &'a [crate::content_identity::ClaimInput<'a>],
) -> crate::content_identity::AllocationInput<'a> {
    crate::content_identity::AllocationInput {
        allocation_id: &provenance.allocation_id,
        consumer_id: &provenance.consumer_id,
        owner_generation: provenance.owner_generation,
        host_id: &provenance.placement.host_id,
        host_incarnation: &provenance.placement.host_incarnation,
        device_incarnation: &provenance.placement.device_incarnation,
        topology_generation: provenance.placement.topology_generation,
        charge_cursor: provenance.charge_cursor,
        charge_at: provenance.charge_at,
        kind: provenance.kind.label(),
        view_id: &provenance.view_id,
        granularity: provenance.granularity.label(),
        claims,
        shared_serving_ref: provenance.shared_serving_ref.as_deref(),
        port_id: &provenance.port_id,
    }
}

/// A persisted binding is intact when its schema is current and its recorded
/// identity equals the identity recomputed from its own fields.
#[must_use]
pub fn is_intact(admitted: &Admitted) -> bool {
    admitted.schema == ADMITTED_SCHEMA
        && identity(admitted).is_ok_and(|recomputed| recomputed == admitted.binding_identity)
}

/// Freeze the allocation facts of a validated observation at admission.
pub(crate) fn freeze(observation: &super::super::model::ComputeAllocationObservation) -> AllocationProvenance {
    let allocation = &observation.allocation;
    debug_assert!(!allocation.claims.is_empty(), "validated allocations claim at least one pool");
    debug_assert!(observation.port.observation_sequence > 0, "validated sequences are positive");
    AllocationProvenance {
        allocation_id: allocation.id.clone(),
        consumer_id: allocation.consumer_id.clone(),
        owner_generation: allocation.owner_generation,
        placement: Placement {
            host_id: allocation.host_id.clone(),
            host_incarnation: allocation.host_incarnation.clone(),
            device_incarnation: allocation.device_incarnation.clone(),
            topology_generation: allocation.topology_generation,
        },
        charge_cursor: allocation.charge_cursor,
        charge_at: allocation.charge_at,
        kind: allocation.kind,
        view_id: allocation.view_id.clone(),
        granularity: allocation.granularity,
        claims: allocation.claims.clone(),
        shared_serving_ref: allocation.serving_allocation_id.clone(),
        port_id: observation.port.port_id.clone(),
        enforcement_at_bind: allocation.enforcement,
        admission_sequence: observation.port.observation_sequence,
    }
}
