//! Pure admission of one exact execution binding.
//!
//! `Rejected` is a definite mismatch and `Blocked` is an unknown or
//! not-yet-current fact. Neither permits an effect, and a blocked allocation is
//! never retried into a different allocation.

// r[impl lattice.execution_binding.admission]
// r[impl lattice.execution_binding.admission.scenario.cpu_remote_model]
// r[impl lattice.execution_binding.admission.scenario.shared_serving]
// r[impl lattice.execution_binding.admission.scenario.hosted_and_no_model]
// r[impl lattice.execution_binding.admission.scenario.direct_device]
// r[impl lattice.execution_binding.admission.scenario.crossed]

mod accelerator;
pub mod bound;
pub mod facts;

type Observation = super::model::ComputeAllocationObservation;

/// What the injected read-only port returned for the declared allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderReading<'a> {
    /// No port call was made (host-local compute).
    NotRead,
    Observed(&'a Observation),
    NotFound,
    /// The provider is not configured or reported itself unavailable.
    Unavailable,
    /// Transport failure, timeout or malformed port reply.
    PortIssue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Decision {
    Admitted(Box<bound::Admitted>),
    Rejected(Rejection),
    Blocked(Block),
}

/// Definite mismatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Rejection {
    UnsupportedDeclaration {
        defect: super::request::DeclarationDefect,
    },
    UnsupportedServing {
        defect: super::serving::Defect,
    },
    RequirementMismatch,
    AllocationNotFound,
    CrossedAttempt,
    CrossedOwner {
        field: &'static str,
    },
    ServingDeploymentAsExecution,
    Released,
    Expired,
    InsufficientDevice,
    GranularityNotAccepted,
    IncompatibleRuntime {
        field: &'static str,
    },
    EnforcementTooWeak,
    ModelBindingMismatch,
    WrongPlacementGeneration {
        field: &'static str,
    },
    ReservationReplaced {
        field: &'static str,
    },
    EnforcementDowngraded,
    StaleObservationSequence,
    Malformed {
        observation: super::validation::ObservationRejection,
    },
}

/// Unknown or not-yet-current facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum Block {
    ProviderUnavailable,
    PortIssue,
    NotActive,
    Quarantined,
    ExpiryUnknown,
    ReadbackStale,
    HostUnhealthy,
    DeviceEvidenceUnknown,
}

pub(crate) enum Stop {
    Rejected(Rejection),
    Blocked(Block),
}

pub(crate) fn reject_if(condition: bool, rejection: Rejection) -> Result<(), Stop> {
    if condition {
        Err(Stop::Rejected(rejection))
    } else {
        Ok(())
    }
}

/// Admit one binding from the declaration, the profile requirement, the
/// attempt identities and, for provider allocations, one port reading.
#[must_use]
pub fn admit(
    declaration: &super::request::Declaration,
    requirement: &super::request::WorkerRequirement,
    attempt: super::request::AttemptContext<'_>,
    reading: ProviderReading<'_>,
) -> Decision {
    match admit_checked(declaration, requirement, attempt, reading) {
        Ok(admitted) => Decision::Admitted(Box::new(admitted)),
        Err(Stop::Rejected(rejection)) => Decision::Rejected(rejection),
        Err(Stop::Blocked(block)) => Decision::Blocked(block),
    }
}

fn admit_checked(
    declaration: &super::request::Declaration,
    requirement: &super::request::WorkerRequirement,
    attempt: super::request::AttemptContext<'_>,
    reading: ProviderReading<'_>,
) -> Result<bound::Admitted, Stop> {
    let unsupported = |defect| Stop::Rejected(Rejection::UnsupportedDeclaration { defect });
    super::request::validate_declaration(declaration).map_err(unsupported)?;
    super::request::validate_requirement(requirement).map_err(unsupported)?;
    validate_attempt(attempt).map_err(unsupported)?;
    check_routes(&declaration.model_routes, &requirement.model)?;
    if let Some(serving) = &declaration.serving {
        super::serving::validate(serving, &declaration.model_routes)
            .map_err(|defect| Stop::Rejected(Rejection::UnsupportedServing { defect }))?;
    }
    let compute = admit_compute(&declaration.compute, &requirement.compute, attempt, reading)?;
    let mut admitted = bound::Admitted {
        schema: bound::ADMITTED_SCHEMA.to_owned(),
        binding_identity: String::new(),
        profile_revision: attempt.profile_revision.to_owned(),
        attempt: bound::AttemptIdentity {
            work_id: attempt.work_id.to_owned(),
            attempt_id: attempt.attempt_id.to_owned(),
            fence: attempt.fence,
            envelope_identity: attempt.envelope_identity.to_owned(),
        },
        compute,
        model_routes: declaration.model_routes.clone(),
        serving: declaration.serving.clone(),
    };
    let identity_defect = super::request::DeclarationDefect::Token {
        field: "binding_identity",
    };
    admitted.binding_identity = bound::identity(&admitted).map_err(|_| unsupported(identity_defect))?;
    debug_assert!(bound::is_intact(&admitted), "a fresh binding recomputes its own identity");
    debug_assert_eq!(admitted.attempt.fence, attempt.fence);
    Ok(admitted)
}

fn validate_attempt(attempt: super::request::AttemptContext<'_>) -> Result<(), super::request::DeclarationDefect> {
    for (value, field) in [
        (attempt.work_id, "work_id"),
        (attempt.attempt_id, "attempt_id"),
        (attempt.envelope_identity, "envelope_identity"),
        (attempt.profile_revision, "profile_revision"),
    ] {
        if !super::is_token(value) {
            return Err(super::request::DeclarationDefect::Token { field });
        }
    }
    if attempt.fence == 0 {
        return Err(super::request::DeclarationDefect::Zero { field: "fence" });
    }
    Ok(())
}

/// Declared routes must equal the profile's named routes exactly.
fn check_routes(
    declared: &super::request::RouteDeclaration,
    required: &super::request::ModelRequirement,
) -> Result<(), Stop> {
    let is_equal = match (declared, required) {
        (super::request::RouteDeclaration::NoModel {}, super::request::ModelRequirement::NoModel {}) => true,
        (super::request::RouteDeclaration::Routes { routes }, super::request::ModelRequirement::Routes { names }) => {
            let declared_names =
                routes.iter().map(|route| route.name.as_str()).collect::<std::collections::BTreeSet<_>>();
            let required_names = names.iter().map(String::as_str).collect::<std::collections::BTreeSet<_>>();
            declared_names == required_names
        }
        (super::request::RouteDeclaration::NoModel {}, super::request::ModelRequirement::Routes { .. })
        | (super::request::RouteDeclaration::Routes { .. }, super::request::ModelRequirement::NoModel {}) => false,
    };
    reject_if(!is_equal, Rejection::ModelBindingMismatch)
}

fn admit_compute(
    declared: &super::request::ComputeDeclaration,
    required: &super::request::ComputeRequirement,
    attempt: super::request::AttemptContext<'_>,
    reading: ProviderReading<'_>,
) -> Result<bound::AdmittedCompute, Stop> {
    match (declared, required) {
        (super::request::ComputeDeclaration::HostLocal {}, super::request::ComputeRequirement::HostLocal {}) => {
            reject_if(reading != ProviderReading::NotRead, Rejection::RequirementMismatch)?;
            Ok(bound::AdmittedCompute::HostLocal {})
        }
        (
            super::request::ComputeDeclaration::ProviderAllocation { .. },
            super::request::ComputeRequirement::ProviderExecution { direct_accelerator },
        ) => {
            let observation = observed(reading)?;
            check_owner(declared, &observation.allocation)?;
            reject_if(observation.allocation.attempt_id != attempt.attempt_id, Rejection::CrossedAttempt)?;
            check_current(observation)?;
            if let Some(accelerator) = direct_accelerator {
                accelerator::require_chips(accelerator, observation)?;
            }
            Ok(bound::AdmittedCompute::ProviderAllocation(Box::new(bound::freeze(observation))))
        }
        (
            super::request::ComputeDeclaration::HostLocal {},
            super::request::ComputeRequirement::ProviderExecution { .. },
        )
        | (
            super::request::ComputeDeclaration::ProviderAllocation { .. },
            super::request::ComputeRequirement::HostLocal {},
        ) => Err(Stop::Rejected(Rejection::RequirementMismatch)),
    }
}

/// Map a port reading to a validated observation or a typed stop.
pub(crate) fn observed(reading: ProviderReading<'_>) -> Result<&Observation, Stop> {
    let observation = match reading {
        ProviderReading::Observed(observation) => observation,
        ProviderReading::NotFound => return Err(Stop::Rejected(Rejection::AllocationNotFound)),
        ProviderReading::NotRead | ProviderReading::Unavailable => {
            return Err(Stop::Blocked(Block::ProviderUnavailable));
        }
        ProviderReading::PortIssue => return Err(Stop::Blocked(Block::PortIssue)),
    };
    super::validation::validate_observation(observation)
        .map_err(|rejection| Stop::Rejected(Rejection::Malformed { observation: rejection }))?;
    Ok(observation)
}

fn check_owner(
    declared: &super::request::ComputeDeclaration,
    allocation: &super::model::AllocationRecord,
) -> Result<(), Stop> {
    let super::request::ComputeDeclaration::ProviderAllocation {
        allocation_id,
        host_id,
        consumer_id,
        owner_generation,
    } = declared
    else {
        return Err(Stop::Rejected(Rejection::RequirementMismatch));
    };
    for (is_equal, field) in [
        (&allocation.id == allocation_id, "allocation_id"),
        (&allocation.host_id == host_id, "host_id"),
        (&allocation.consumer_id == consumer_id, "consumer_id"),
        (allocation.owner_generation == *owner_generation, "owner_generation"),
    ] {
        reject_if(!is_equal, Rejection::CrossedOwner { field })?;
    }
    debug_assert_eq!(&allocation.id, allocation_id);
    debug_assert_eq!(allocation.owner_generation, *owner_generation);
    Ok(())
}

/// Kind, status, logical time and readback must describe a current execution
/// allocation. Definite rejections are reported before unknown facts.
pub(crate) fn check_current(observation: &Observation) -> Result<(), Stop> {
    let allocation = &observation.allocation;
    let is_serving = allocation.kind == super::model::AllocationKind::ServingDeployment;
    reject_if(is_serving, Rejection::ServingDeploymentAsExecution)?;
    reject_if(allocation.status == super::model::ReservationStatus::Released, Rejection::Released)?;
    let classification = facts::classify(observation);
    reject_if(classification.expiry == facts::Expiry::Expired, Rejection::Expired)?;
    let block = match allocation.status {
        super::model::ReservationStatus::Intent => Some(Block::NotActive),
        super::model::ReservationStatus::Quarantined => Some(Block::Quarantined),
        super::model::ReservationStatus::Active | super::model::ReservationStatus::Released => None,
    }
    .or((classification.expiry == facts::Expiry::Unknown).then_some(Block::ExpiryUnknown))
    .or((classification.freshness == facts::Freshness::Unknown).then_some(Block::ReadbackStale))
    .or((classification.health != facts::HostHealth::Healthy).then_some(Block::HostUnhealthy));
    block.map_or(Ok(()), |block| Err(Stop::Blocked(block)))
}
