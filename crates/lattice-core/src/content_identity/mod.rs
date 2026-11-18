//! Lattice-owned content identity roles over the reviewed shared framing core.
//!
//! Lattice owns every domain, canonical input, output spelling, persistence rule,
//! and claim. The shared package supplies only admitted unkeyed framing mechanics.

pub const SCHEDULE_OCCURRENCE_DOMAIN: &str = "lattice-schedule-occurrence-v1";
const SCHEDULE_ID_TAG: &[u8] = b"schedule-id";
const DUE_EPOCH_MS_TAG: &[u8] = b"due-epoch-ms";
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";

/// Canonical Lattice inputs for one durable schedule-occurrence identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleOccurrenceInput<'a> {
    pub schedule_id: &'a str,
    pub due_epoch_ms: i64,
}

/// Build the durable schedule-occurrence identity with unchanged Lattice bytes.
///
/// The role wrapper fixes the product domain, tags, integer encoding, and output
/// spelling. The shared core only performs the existing hashed-part framing.
// r[impl lattice.content_identity_adoption.boundary]
// r[impl lattice.content_identity_adoption.cutover]
pub fn schedule_occurrence(input: ScheduleOccurrenceInput<'_>) -> Result<String, content_identity_core::IdentityError> {
    let domain = content_identity_core::Domain::new(SCHEDULE_OCCURRENCE_DOMAIN)?;
    let mut hasher = content_identity_core::IdentityHasher::new(domain);
    hasher
        .update_hashed_part(SCHEDULE_ID_TAG, input.schedule_id.as_bytes())
        .update_hashed_part(DUE_EPOCH_MS_TAG, &input.due_epoch_ms.to_be_bytes());
    Ok(format!("{BLAKE3_IDENTITY_PREFIX}{}", hasher.finish().to_hex()))
}

pub const EXECUTION_BINDING_DOMAIN: &str = "lattice-execution-binding-v1";
pub const MODEL_ROUTE_DOMAIN: &str = "lattice-model-route-v1";
pub const WORKER_INSTANCE_STATE_DOMAIN: &str = "lattice-worker-instance-state-v1";
/// Spelling shared with `ConfiguredBinding` and producer references.
const BLAKE3_REFERENCE_PREFIX: &str = "blake3:";

/// Canonical inputs of one admitted execution binding.
///
/// Volatile observation facts (sequence, provider logical time, readback
/// revision, freshness, status, enforcement and producer record references)
/// are deliberately absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionBindingInput<'a> {
    pub profile_revision: &'a str,
    pub work_id: &'a str,
    pub attempt_id: &'a str,
    pub fence: u64,
    pub envelope_identity: &'a str,
    /// `None` is host-local compute.
    pub allocation: Option<&'a AllocationInput<'a>>,
    /// `None` is a no-model worker.
    pub routes: Option<&'a [RouteInput<'a>]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocationInput<'a> {
    pub allocation_id: &'a str,
    pub consumer_id: &'a str,
    pub owner_generation: u64,
    pub host_id: &'a str,
    pub host_incarnation: &'a str,
    pub device_incarnation: &'a str,
    pub topology_generation: u64,
    pub charge_cursor: u64,
    pub charge_at: u64,
    pub kind: &'a str,
    pub view_id: &'a str,
    pub granularity: &'a str,
    pub claims: &'a [ClaimInput<'a>],
    pub shared_serving_ref: Option<&'a str>,
    pub port_id: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimInput<'a> {
    pub pool_id: &'a str,
    pub quantity: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteInput<'a> {
    pub name: &'a str,
    pub route_identity: &'a str,
}

/// Build the `lattice-execution-binding-v1` identity, spelled `blake3:<hex>`.
///
/// Claims are framed as `pool_id=quantity` sorted by pool id and routes as
/// `name=route_identity` sorted by name; splitting at the last `=` stays
/// unambiguous because quantities are decimal and identities are hexadecimal.
// r[impl lattice.execution_binding.identity]
// r[impl lattice.execution_binding.identity.scenario.stable]
// r[impl lattice.execution_binding.identity.scenario.drift]
pub fn execution_binding(input: ExecutionBindingInput<'_>) -> Result<String, content_identity_core::IdentityError> {
    let mut hasher =
        content_identity_core::IdentityHasher::new(content_identity_core::Domain::new(EXECUTION_BINDING_DOMAIN)?);
    hasher
        .update_tagged_str(b"profile-revision", input.profile_revision)?
        .update_tagged_str(b"work-id", input.work_id)?
        .update_tagged_str(b"attempt-id", input.attempt_id)?
        .update_tagged_u64_le(b"fence", input.fence)
        .update_tagged_str(b"envelope-identity", input.envelope_identity)?;
    match input.allocation {
        None => {
            hasher.update_tagged_str(b"compute-kind", "host-local")?;
        }
        Some(allocation) => {
            hasher.update_tagged_str(b"compute-kind", "provider-allocation")?;
            update_allocation(&mut hasher, allocation)?;
        }
    }
    match input.routes {
        None => {
            hasher.update_tagged_str(b"model-kind", "no-model")?;
        }
        Some(routes) => {
            let mut sorted = routes.iter().collect::<Vec<_>>();
            sorted.sort_unstable_by_key(|route| route.name);
            let framed =
                sorted.iter().map(|route| format!("{}={}", route.name, route.route_identity)).collect::<Vec<_>>();
            let items = framed.iter().map(String::as_str).collect::<Vec<_>>();
            debug_assert_eq!(framed.len(), routes.len());
            debug_assert_eq!(items.len(), framed.len());
            hasher.update_tagged_str(b"model-kind", "routes")?.update_ordered_strs(b"model-routes", &items)?;
        }
    }
    Ok(format!("{BLAKE3_REFERENCE_PREFIX}{}", hasher.finish().to_hex()))
}

fn update_allocation(
    hasher: &mut content_identity_core::IdentityHasher,
    allocation: &AllocationInput<'_>,
) -> Result<(), content_identity_core::IdentityError> {
    let mut sorted = allocation.claims.iter().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|claim| claim.pool_id);
    let framed = sorted.iter().map(|claim| format!("{}={}", claim.pool_id, claim.quantity)).collect::<Vec<_>>();
    let claims = framed.iter().map(String::as_str).collect::<Vec<_>>();
    debug_assert_eq!(framed.len(), allocation.claims.len());
    debug_assert_eq!(claims.len(), framed.len());
    hasher
        .update_tagged_str(b"allocation-id", allocation.allocation_id)?
        .update_tagged_str(b"consumer-id", allocation.consumer_id)?
        .update_tagged_u64_le(b"owner-generation", allocation.owner_generation)
        .update_tagged_str(b"host-id", allocation.host_id)?
        .update_tagged_str(b"host-incarnation", allocation.host_incarnation)?
        .update_tagged_str(b"device-incarnation", allocation.device_incarnation)?
        .update_tagged_u64_le(b"topology-generation", allocation.topology_generation)
        .update_tagged_u64_le(b"charge-cursor", allocation.charge_cursor)
        .update_tagged_u64_le(b"charge-at", allocation.charge_at)
        .update_tagged_str(b"allocation-kind", allocation.kind)?
        .update_tagged_str(b"view-id", allocation.view_id)?
        .update_tagged_str(b"granularity", allocation.granularity)?
        .update_ordered_strs(b"claims", &claims)?
        .update_optional_str(b"shared-serving-ref", allocation.shared_serving_ref)?
        .update_tagged_str(b"port-id", allocation.port_id)?;
    Ok(())
}

/// Canonical inputs of one brokered model route. Expiry and credential values
/// are excluded; only the credential reference is framed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelRouteInput<'a> {
    pub schema: &'a str,
    pub endpoint: &'a str,
    pub model: &'a str,
    pub credential_provider: Option<&'a str>,
    pub credential_key: Option<&'a str>,
    pub credential_field: Option<&'a str>,
}

/// Build the `lattice-model-route-v1` identity, spelled `blake3:<hex>`.
// r[impl lattice.execution_binding.identity]
pub fn model_route(input: ModelRouteInput<'_>) -> Result<String, content_identity_core::IdentityError> {
    let mut hasher =
        content_identity_core::IdentityHasher::new(content_identity_core::Domain::new(MODEL_ROUTE_DOMAIN)?);
    hasher
        .update_tagged_str(b"schema", input.schema)?
        .update_tagged_str(b"endpoint", input.endpoint)?
        .update_tagged_str(b"model", input.model)?
        .update_optional_str(b"credential-provider", input.credential_provider)?
        .update_optional_str(b"credential-key", input.credential_key)?
        .update_optional_str(b"credential-field", input.credential_field)?;
    Ok(format!("{BLAKE3_REFERENCE_PREFIX}{}", hasher.finish().to_hex()))
}

/// Digest over `(artifact name, artifact identity)` pairs of retained state.
// r[impl lattice.execution_binding.instances]
pub fn worker_instance_state(artifacts: &[(&str, &str)]) -> Result<String, content_identity_core::IdentityError> {
    let mut hasher =
        content_identity_core::IdentityHasher::new(content_identity_core::Domain::new(WORKER_INSTANCE_STATE_DOMAIN)?);
    hasher.update_sorted_map(b"artifacts", artifacts)?;
    Ok(format!("{BLAKE3_REFERENCE_PREFIX}{}", hasher.finish().to_hex()))
}
