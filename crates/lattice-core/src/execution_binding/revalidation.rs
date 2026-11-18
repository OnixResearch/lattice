//! Pre-spawn revalidation of a persisted binding against a newer observation.
//!
//! Every frozen allocation fact must be unchanged, the observation sequence
//! must strictly advance, the allocation must still be current and enforcement
//! may not weaken. Host-local bindings never consult the port.

// r[impl lattice.execution_binding.persistence]
// r[impl lattice.execution_binding.persistence.scenario.revalidation]

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Recheck {
    Current,
    Rejected(super::admission::Rejection),
    Blocked(super::admission::Block),
}

/// Recheck one admitted binding immediately before the worker would spawn.
#[must_use]
pub fn revalidate(
    admitted: &super::admission::bound::Admitted,
    reading: super::admission::ProviderReading<'_>,
) -> Recheck {
    match recheck(admitted, reading) {
        Ok(()) => Recheck::Current,
        Err(super::admission::Stop::Rejected(rejection)) => Recheck::Rejected(rejection),
        Err(super::admission::Stop::Blocked(block)) => Recheck::Blocked(block),
    }
}

fn recheck(
    admitted: &super::admission::bound::Admitted,
    reading: super::admission::ProviderReading<'_>,
) -> Result<(), super::admission::Stop> {
    let super::admission::bound::AdmittedCompute::ProviderAllocation(frozen) = &admitted.compute else {
        let is_unread = reading == super::admission::ProviderReading::NotRead;
        return super::admission::reject_if(!is_unread, super::admission::Rejection::RequirementMismatch);
    };
    let observation = super::admission::observed(reading)?;
    let allocation = &observation.allocation;
    for (is_equal, field) in [
        (allocation.id == frozen.allocation_id, "allocation_id"),
        (allocation.consumer_id == frozen.consumer_id, "consumer_id"),
        (allocation.owner_generation == frozen.owner_generation, "owner_generation"),
        (observation.port.port_id == frozen.port_id, "port_id"),
    ] {
        super::admission::reject_if(!is_equal, super::admission::Rejection::CrossedOwner { field })?;
    }
    let is_same_attempt = allocation.attempt_id == admitted.attempt.attempt_id;
    super::admission::reject_if(!is_same_attempt, super::admission::Rejection::CrossedAttempt)?;
    check_placement(&frozen.placement, allocation)?;
    check_reservation(frozen, allocation)?;
    let has_advanced = observation.port.observation_sequence > frozen.admission_sequence;
    super::admission::reject_if(!has_advanced, super::admission::Rejection::StaleObservationSequence)?;
    let is_weaker = allocation.enforcement < frozen.enforcement_at_bind;
    super::admission::reject_if(is_weaker, super::admission::Rejection::EnforcementDowngraded)?;
    debug_assert_eq!(allocation.charge_cursor, frozen.charge_cursor);
    debug_assert!(has_advanced);
    super::admission::check_current(observation)
}

fn check_placement(
    placement: &super::admission::bound::Placement,
    allocation: &super::model::AllocationRecord,
) -> Result<(), super::admission::Stop> {
    for (is_equal, field) in [
        (allocation.host_id == placement.host_id, "host_id"),
        (allocation.host_incarnation == placement.host_incarnation, "host_incarnation"),
        (allocation.device_incarnation == placement.device_incarnation, "device_incarnation"),
        (allocation.topology_generation == placement.topology_generation, "topology_generation"),
    ] {
        super::admission::reject_if(!is_equal, super::admission::Rejection::WrongPlacementGeneration { field })?;
    }
    Ok(())
}

/// A different charge cursor identifies a reservation readmitted after the
/// producer's replay horizon evicted the original id.
fn check_reservation(
    frozen: &super::admission::bound::AllocationProvenance,
    allocation: &super::model::AllocationRecord,
) -> Result<(), super::admission::Stop> {
    super::admission::reject_if(
        allocation.kind != frozen.kind,
        super::admission::Rejection::ServingDeploymentAsExecution,
    )?;
    for (is_equal, field) in [
        (allocation.charge_cursor == frozen.charge_cursor, "charge_cursor"),
        (allocation.charge_at == frozen.charge_at, "charge_at"),
        (allocation.view_id == frozen.view_id, "view_id"),
        (allocation.granularity == frozen.granularity, "granularity"),
        (allocation.claims == frozen.claims, "claims"),
        (allocation.serving_allocation_id == frozen.shared_serving_ref, "shared_serving_ref"),
    ] {
        super::admission::reject_if(!is_equal, super::admission::Rejection::ReservationReplaced { field })?;
    }
    Ok(())
}
