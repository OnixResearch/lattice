//! The direct-accelerator rule. Chips come only from this allocation's own
//! claimed `chips` pools; the referenced serving deployment never counts.

// r[impl lattice.execution_binding.admission.scenario.direct_device]

pub(super) fn require_chips(
    accelerator: &super::super::request::DirectAccelerator,
    observation: &super::super::model::ComputeAllocationObservation,
) -> Result<(), super::Stop> {
    let super::facts::DeviceKind::Known { chips } = super::facts::classify(observation).device_kind else {
        return Err(super::Stop::Blocked(super::Block::DeviceEvidenceUnknown));
    };
    let allocation = &observation.allocation;
    super::reject_if(chips < accelerator.min_chips, super::Rejection::InsufficientDevice)?;
    let is_accepted = accelerator.accepted_granularities.contains(&allocation.granularity);
    super::reject_if(!is_accepted, super::Rejection::GranularityNotAccepted)?;
    let offered = observation.offer.as_ref().map(|offer| &offer.compatibility);
    for (required, actual, field) in [
        (&accelerator.backend, offered.and_then(|compatibility| compatibility.backend.as_deref()), "backend"),
        (&accelerator.runtime, offered.and_then(|compatibility| compatibility.runtime.as_deref()), "runtime"),
        (
            &accelerator.architecture,
            offered.and_then(|compatibility| compatibility.architecture.as_deref()),
            "architecture",
        ),
    ] {
        let is_compatible = required.as_deref().is_none_or(|value| actual == Some(value));
        super::reject_if(!is_compatible, super::Rejection::IncompatibleRuntime { field })?;
    }
    debug_assert!(chips >= accelerator.min_chips);
    debug_assert!(is_accepted);
    super::reject_if(allocation.enforcement < accelerator.minimum_enforcement, super::Rejection::EnforcementTooWeak)
}
