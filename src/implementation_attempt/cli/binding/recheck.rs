//! The non-writing half of pre-spawn revalidation.

// r[impl lattice.execution_binding.persistence.scenario.revalidation]

/// Re-observe a provider binding without writing anything; `Err` is the
/// revalidation record of a binding that is no longer current.
/// [`super::revalidate`] retains that record as a pre-spawn refusal; a
/// repair step after the first retains it in its post-dispatch disposition.
pub(in super::super) fn current(
    admitted: &super::Admitted,
    port: &mut dyn crate::execution_binding::ComputeAllocationPort,
) -> Result<(), serde_json::Value> {
    let reply = match &admitted.compute {
        lattice_core::execution_binding::admission::bound::AdmittedCompute::HostLocal {} => None,
        lattice_core::execution_binding::admission::bound::AdmittedCompute::ProviderAllocation(frozen) => {
            Some(port.observe_allocation(&crate::execution_binding::AllocationQuery {
                allocation_id: frozen.allocation_id.clone(),
                host_id: frozen.placement.host_id.clone(),
                consumer_id: frozen.consumer_id.clone(),
                attempt_id: admitted.attempt.attempt_id.clone(),
            }))
        }
    };
    let reading = reply.as_ref().map_or(
        lattice_core::execution_binding::admission::ProviderReading::NotRead,
        crate::execution_binding::reading,
    );
    let recheck = lattice_core::execution_binding::revalidation::revalidate(admitted, reading);
    if recheck == lattice_core::execution_binding::revalidation::Recheck::Current {
        return Ok(());
    }
    debug_assert_ne!(recheck, lattice_core::execution_binding::revalidation::Recheck::Current);
    debug_assert!(reply.is_some(), "host-local bindings are always current");
    Err(serde_json::json!({
        "schema": super::REFUSAL_SCHEMA,
        "binding_identity": admitted.binding_identity,
        "decision": recheck,
    }))
}
