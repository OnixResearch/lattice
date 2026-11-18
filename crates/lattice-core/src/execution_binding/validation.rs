//! Pure, bounded validation and unknown-fact classification of the DTO.
//!
//! Validation mirrors the producer's own record invariants and adds
//! cross-record consistency. Lattice never compares producer logical time with
//! its wall clock.

// r[impl lattice.execution_binding.allocation_observation.scenario.malformed]

type Allocation = super::model::AllocationRecord;
type Observation = super::model::ComputeAllocationObservation;

/// Typed reason a DTO is rejected; no partial observation is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum ObservationRejection {
    Oversized,
    Malformed,
    Schema,
    ProducerLabel { field: &'static str },
    Token { field: &'static str },
    Reference { field: &'static str },
    Zero { field: &'static str },
    Bound { field: &'static str },
    IdNotOperation,
    RepeatedPool,
    AuthorityCount,
    ServingReferencesServing,
    ActivationEvidence,
    ReadbackRow { field: &'static str },
    Offer { field: &'static str },
}

/// Decode at most 65,536 bytes of JSON and validate the result.
pub fn decode_observation(bytes: &[u8]) -> Result<Observation, ObservationRejection> {
    let is_bounded = u64::try_from(bytes.len()).is_ok_and(|length_bytes| length_bytes <= super::MAX_OBSERVATION_BYTES);
    require(is_bounded, ObservationRejection::Oversized)?;
    let observation = serde_json::from_slice(bytes).map_err(|_| ObservationRejection::Malformed)?;
    validate_observation(&observation)?;
    Ok(observation)
}

/// Producer-equivalent invariants, cross-record consistency and the size bound.
pub fn validate_observation(observation: &Observation) -> Result<(), ObservationRejection> {
    require(observation.schema == super::model::OBSERVATION_SCHEMA, ObservationRejection::Schema)?;
    validate_cohort(&observation.producer)?;
    validate_port(&observation.port)?;
    validate_allocation(&observation.allocation)?;
    validate_readback(&observation.readback, &observation.allocation)?;
    if let Some(offer) = &observation.offer {
        validate_offer(offer, &observation.allocation)?;
    }
    require(is_within_bound(observation), ObservationRejection::Oversized)
}

fn validate_cohort(cohort: &super::model::ProducerCohort) -> Result<(), ObservationRejection> {
    for (value, expected, field) in [
        (&cohort.allocation_label, super::model::ALLOCATION_LABEL, "allocation_label"),
        (&cohort.readback_label, super::model::READBACK_LABEL, "readback_label"),
        (&cohort.offer_label, super::model::OFFER_LABEL, "offer_label"),
        (&cohort.reservation_schema, super::model::RESERVATION_SCHEMA, "reservation_schema"),
        (&cohort.offer_schema, super::model::OFFER_SCHEMA, "offer_schema"),
        (&cohort.time_domain, super::model::TIME_DOMAIN, "time_domain"),
    ] {
        require(value == expected, ObservationRejection::ProducerLabel { field })?;
    }
    Ok(())
}

fn validate_port(port: &super::model::PortProvenance) -> Result<(), ObservationRejection> {
    require(super::is_token(&port.port_id), ObservationRejection::Token { field: "port_id" })?;
    reference(&port.port_profile_ref, ObservationRejection::Reference {
        field: "port_profile_ref",
    })?;
    require(port.observation_sequence > 0, ObservationRejection::Zero {
        field: "observation_sequence",
    })
}

fn validate_allocation(allocation: &Allocation) -> Result<(), ObservationRejection> {
    reference(&allocation.record_ref, ObservationRejection::Reference { field: "record_ref" })?;
    reference(&allocation.grant_ref, ObservationRejection::Reference { field: "grant_ref" })?;
    let is_schema = allocation.schema == super::model::RESERVATION_SCHEMA;
    require(is_schema, ObservationRejection::ProducerLabel { field: "schema" })?;
    validate_allocation_tokens(allocation)?;
    for (value, field) in [
        (allocation.offer_revision, "offer_revision"),
        (allocation.topology_generation, "topology_generation"),
        (allocation.charge_cursor, "charge_cursor"),
        (allocation.valid_until, "valid_until"),
        (allocation.owner_generation, "owner_generation"),
    ] {
        require(value > 0, ObservationRejection::Zero { field })?;
    }
    require(allocation.id == allocation.operation_id, ObservationRejection::IdNotOperation)?;
    validate_claims(allocation)?;
    let is_serving = allocation.kind == super::model::AllocationKind::ServingDeployment;
    require(
        !is_serving || allocation.serving_allocation_id.is_none(),
        ObservationRejection::ServingReferencesServing,
    )?;
    debug_assert_eq!(allocation.id, allocation.operation_id);
    debug_assert!(!allocation.claims.is_empty());
    require(has_activation_evidence(allocation), ObservationRejection::ActivationEvidence)
}

fn validate_allocation_tokens(allocation: &Allocation) -> Result<(), ObservationRejection> {
    for (value, field) in [
        (&allocation.id, "id"),
        (&allocation.host_id, "host_id"),
        (&allocation.host_incarnation, "host_incarnation"),
        (&allocation.device_incarnation, "device_incarnation"),
        (&allocation.view_id, "view_id"),
        (&allocation.consumer_id, "consumer_id"),
        (&allocation.operation_id, "operation_id"),
        (&allocation.attempt_id, "attempt_id"),
    ] {
        require(super::is_token(value), ObservationRejection::Token { field })?;
    }
    for (value, field) in [
        (&allocation.serving_allocation_id, "serving_allocation_id"),
        (&allocation.activation_backend_ref, "activation_backend_ref"),
    ] {
        require(value.as_deref().is_none_or(super::is_token), ObservationRejection::Token { field })?;
    }
    debug_assert!(super::is_token(&allocation.id));
    debug_assert!(super::is_token(&allocation.attempt_id));
    Ok(())
}

fn validate_claims(allocation: &Allocation) -> Result<(), ObservationRejection> {
    let claims = &allocation.claims;
    let is_bounded = (1..=super::MAX_ITEMS).contains(&claims.len());
    require(is_bounded, ObservationRejection::Bound { field: "claims" })?;
    require(allocation.authority_ids.len() == claims.len(), ObservationRejection::AuthorityCount)?;
    let has_tokens = allocation.authority_ids.iter().all(|id| super::is_token(id));
    require(has_tokens, ObservationRejection::Token { field: "authority_ids" })?;
    require(claims.iter().all(|claim| super::is_token(&claim.pool_id)), ObservationRejection::Token {
        field: "pool_id",
    })?;
    require(
        super::is_distinct(claims.iter().map(|claim| claim.pool_id.as_str())),
        ObservationRejection::RepeatedPool,
    )?;
    require(claims.iter().all(|claim| claim.quantity > 0), ObservationRejection::Zero { field: "quantity" })
}

/// `active` requires activation evidence and `intent` forbids it.
fn has_activation_evidence(allocation: &Allocation) -> bool {
    match allocation.status {
        super::model::ReservationStatus::Active => allocation.activation_backend_ref.is_some(),
        super::model::ReservationStatus::Intent => allocation.activation_backend_ref.is_none(),
        super::model::ReservationStatus::Quarantined | super::model::ReservationStatus::Released => true,
    }
}

fn validate_readback(
    readback: &super::model::ReadbackProjection,
    allocation: &Allocation,
) -> Result<(), ObservationRejection> {
    reference(&readback.record_ref, ObservationRejection::Reference {
        field: "readback.record_ref",
    })?;
    require(readback.host_id == allocation.host_id, ObservationRejection::ReadbackRow { field: "host_id" })?;
    let is_current = readback.revision > 0 && readback.revision >= allocation.offer_revision;
    require(is_current, ObservationRejection::ReadbackRow { field: "revision" })?;
    references(&readback.qualification_refs, "readback.qualification_refs")?;
    let maximum_rows =
        u64::try_from(super::MAX_ITEMS).map_err(|_| ObservationRejection::Bound { field: "row_count" })?;
    let is_bounded = (1..=maximum_rows).contains(&readback.row_count);
    require(is_bounded, ObservationRejection::Bound { field: "row_count" })?;
    validate_row(&readback.row, allocation)
}

fn validate_row(row: &super::model::ReadbackRow, allocation: &Allocation) -> Result<(), ObservationRejection> {
    for (is_equal, field) in [
        (row.id == allocation.id, "id"),
        (row.consumer_id == allocation.consumer_id, "consumer_id"),
        (row.owner_generation == allocation.owner_generation, "owner_generation"),
        (row.granularity == allocation.granularity, "granularity"),
        (row.status == allocation.status, "status"),
        (row.enforcement == allocation.enforcement, "enforcement"),
    ] {
        require(is_equal, ObservationRejection::ReadbackRow { field })?;
    }
    let is_fenced = allocation.status != super::model::ReservationStatus::Released;
    require(row.is_logically_fenced == is_fenced, ObservationRejection::ReadbackRow {
        field: "is_logically_fenced",
    })
}

fn validate_offer(offer: &super::model::OfferPoolFacts, allocation: &Allocation) -> Result<(), ObservationRejection> {
    reference(&offer.record_ref, ObservationRejection::Reference {
        field: "offer.record_ref",
    })?;
    for (is_equal, field) in [
        (offer.host_id == allocation.host_id, "host_id"),
        (offer.host_incarnation == allocation.host_incarnation, "host_incarnation"),
        (offer.device_incarnation == allocation.device_incarnation, "device_incarnation"),
        (offer.revision == allocation.offer_revision, "revision"),
        (offer.topology_generation == allocation.topology_generation, "topology_generation"),
    ] {
        require(is_equal, ObservationRejection::Offer { field })?;
    }
    let compatibility = &offer.compatibility;
    for value in [
        &compatibility.backend,
        &compatibility.runtime,
        &compatibility.architecture,
        &compatibility.topology,
    ] {
        require(value.as_deref().is_none_or(super::is_token), ObservationRejection::Token { field: "compatibility" })?;
    }
    let has_qualification = !compatibility.qualification_refs.is_empty();
    require(has_qualification, ObservationRejection::Bound {
        field: "offer.qualification_refs",
    })?;
    references(&compatibility.qualification_refs, "offer.qualification_refs")?;
    debug_assert_eq!(offer.revision, allocation.offer_revision);
    debug_assert_eq!(offer.host_id, allocation.host_id);
    validate_claimed_pools(offer, allocation)
}

fn validate_claimed_pools(
    offer: &super::model::OfferPoolFacts,
    allocation: &Allocation,
) -> Result<(), ObservationRejection> {
    let pools = &offer.claimed_pools;
    let has_each_claim = pools.len() == allocation.claims.len()
        && allocation.claims.iter().all(|claim| pools.iter().any(|pool| pool.pool_id == claim.pool_id));
    require(has_each_claim, ObservationRejection::Offer { field: "claimed_pools" })?;
    require(
        super::is_distinct(pools.iter().map(|pool| pool.pool_id.as_str())),
        ObservationRejection::RepeatedPool,
    )?;
    let offered = pools.iter().map(|pool| pool.authority_id.as_str()).collect::<std::collections::BTreeSet<_>>();
    let allocated = allocation.authority_ids.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
    require(offered == allocated, ObservationRejection::Offer { field: "authority_id" })
}

fn reference(value: &str, rejection: ObservationRejection) -> Result<(), ObservationRejection> {
    require(super::is_reference(value), rejection)
}

fn references(values: &[String], field: &'static str) -> Result<(), ObservationRejection> {
    require(values.len() <= super::MAX_ITEMS, ObservationRejection::Bound { field })?;
    require(values.iter().all(|value| super::is_reference(value)), ObservationRejection::Reference { field })?;
    require(super::is_distinct(values.iter().map(String::as_str)), ObservationRejection::Reference { field })
}

fn require(condition: bool, rejection: ObservationRejection) -> Result<(), ObservationRejection> {
    if condition { Ok(()) } else { Err(rejection) }
}

/// Counts serialized bytes without allocating and stops past the ceiling.
struct ByteBudget {
    remaining_bytes: u64,
}

impl std::io::Write for ByteBudget {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let written_bytes = u64::try_from(buffer.len()).map_err(std::io::Error::other)?;
        self.remaining_bytes = self
            .remaining_bytes
            .checked_sub(written_bytes)
            .ok_or_else(|| std::io::Error::other("observation exceeds its byte ceiling"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn is_within_bound(observation: &Observation) -> bool {
    let mut budget = ByteBudget {
        remaining_bytes: super::MAX_OBSERVATION_BYTES,
    };
    serde_json::to_writer(&mut budget, observation).is_ok()
}
