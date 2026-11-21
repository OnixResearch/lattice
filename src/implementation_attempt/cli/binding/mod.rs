//! Execution binding steps of one implementation attempt.
//!
//! The admitted binding and its admission observation are written once, with
//! fsync, before `intent.json`. After the intent and immediately before the
//! worker spawns, a provider binding is re-observed and revalidated; any other
//! result than `current` is written once and the worker never spawns.
//! Recovery, cancellation and inspection never call the port.

// r[impl lattice.execution_binding.persistence]
// r[impl lattice.execution_binding.persistence.scenario.persisted_first]
// r[impl lattice.execution_binding.persistence.scenario.revalidation]
// r[impl lattice.execution_binding.persistence.scenario.changed_binding]
// r[impl lattice.execution_binding.port_boundary.scenario.unavailable]
// r[impl lattice.execution_binding.recovery]

pub(super) mod recheck;
pub(super) mod requests;
#[cfg(test)]
pub(super) mod tests;
pub(super) mod usage;

pub(super) const ADMITTED_FILE: &str = "execution-binding.json";
pub(super) const REFUSAL_FILE: &str = "binding-revalidation.json";
const PERSISTED_SCHEMA: &str = "lattice.persisted-execution-binding.v1";
const REFUSAL_SCHEMA: &str = "lattice.binding-revalidation.v1";
/// The admitted binding plus one bounded admission observation.
const MAX_PERSISTED_BYTES: usize = 262_144;
const MAX_REFUSAL_BYTES: usize = 16_384;
/// The OMP worker has exactly one brokered route with this name.
pub(super) const PRIMARY_ROUTE: &str = "primary";

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
type Observation = lattice_core::execution_binding::model::ComputeAllocationObservation;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Persisted {
    schema: String,
    binding: Admitted,
    admission_observation: Option<Observation>,
}

/// The `primary` route identity derived from the brokered model profile.
pub(super) fn primary_route(
    request: &super::Request,
) -> Result<lattice_core::execution_binding::request::NamedRoute, String> {
    let broker = &request.omp.model_broker;
    let credential = broker.credential.as_ref();
    let route_identity = crate::content_identity::model_route(crate::content_identity::ModelRouteInput {
        schema: &broker.schema,
        endpoint: &broker.endpoint,
        model: &broker.model,
        credential_provider: credential.map(|reference| reference.provider.as_str()),
        credential_key: credential.map(|reference| reference.key.as_str()),
        credential_field: credential.and_then(|reference| reference.field.as_deref()),
    })
    .map_err(|error| error.to_string())?;
    Ok(lattice_core::execution_binding::request::NamedRoute {
        name: PRIMARY_ROUTE.to_owned(),
        route_identity,
    })
}

/// Preparation derives the identity of a single declared `primary` route from
/// the brokered profile, like every other derived identity it normalizes. Any
/// other route shape is left untouched and refused by validation.
pub(super) fn normalize_primary_route(request: &mut super::Request) -> Result<(), String> {
    let derived = primary_route(request)?;
    if let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &mut request.execution_binding.model_routes
        && let [route] = routes.as_mut_slice()
        && route.name == PRIMARY_ROUTE
    {
        *route = derived;
    }
    Ok(())
}

/// The OMP worker's compute requirement and its single named model route.
pub(super) fn requirement(request: &super::Request) -> lattice_core::execution_binding::request::WorkerRequirement {
    lattice_core::execution_binding::request::WorkerRequirement {
        compute: request.worker_compute.clone(),
        model: lattice_core::execution_binding::request::ModelRequirement::Routes {
            names: vec![PRIMARY_ROUTE.to_owned()],
        },
    }
}

/// Refuse unsupported declarations before any binding, intent or worker effect.
pub(super) fn validate_request(request: &super::Request) -> Result<(), String> {
    lattice_core::execution_binding::request::validate_requirement(&requirement(request))
        .and_then(|()| lattice_core::execution_binding::request::validate_declaration(&request.execution_binding))
        .map_err(|defect| format!("execution binding declaration is unsupported: {defect:?}"))?;
    let lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } =
        &request.execution_binding.model_routes
    else {
        return Err(
            "OMP attempt workers require their brokered model route; no-model declarations are unsupported".into()
        );
    };
    debug_assert!(!routes.is_empty(), "validated route declarations are nonempty");
    debug_assert!(routes.len() <= 8, "validated route declarations are bounded");
    if routes.len() != 1 {
        return Err("OMP attempt workers declare exactly one model route; extra routes are unsupported".into());
    }
    if routes.first() != Some(&primary_route(request)?) {
        return Err("declared model route differs from the brokered primary route".into());
    }
    let is_host_local_requirement =
        matches!(request.worker_compute, lattice_core::execution_binding::request::ComputeRequirement::HostLocal {});
    let is_host_local_declaration = matches!(
        request.execution_binding.compute,
        lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {}
    );
    if is_host_local_requirement != is_host_local_declaration {
        return Err("declared compute placement differs from the worker compute requirement".into());
    }
    requests::validate(request)
}

pub(super) fn context(request: &super::Request) -> lattice_core::execution_binding::request::AttemptContext<'_> {
    lattice_core::execution_binding::request::AttemptContext {
        work_id: &request.envelope.work_id,
        attempt_id: &request.envelope.attempt_id,
        fence: request.envelope.fence,
        envelope_identity: &request.admitted_envelope,
        profile_revision: &request.envelope.worker_profile,
    }
}

fn query(request: &super::Request) -> Option<crate::execution_binding::AllocationQuery> {
    match &request.execution_binding.compute {
        lattice_core::execution_binding::request::ComputeDeclaration::HostLocal {} => None,
        lattice_core::execution_binding::request::ComputeDeclaration::ProviderAllocation {
            allocation_id,
            host_id,
            consumer_id,
            ..
        } => Some(crate::execution_binding::AllocationQuery {
            allocation_id: allocation_id.clone(),
            host_id: host_id.clone(),
            consumer_id: consumer_id.clone(),
            attempt_id: request.envelope.attempt_id.clone(),
        }),
    }
}

/// Admit with at most one port read; host-local compute makes no port call.
pub(super) fn admit(
    request: &super::Request,
    port: &mut dyn crate::execution_binding::ComputeAllocationPort,
) -> Result<(Admitted, Option<Observation>), lattice_core::execution_binding::admission::Decision> {
    let reply = query(request).map(|query| port.observe_allocation(&query));
    let is_provider = matches!(
        request.execution_binding.compute,
        lattice_core::execution_binding::request::ComputeDeclaration::ProviderAllocation { .. }
    );
    debug_assert_eq!(reply.is_some(), is_provider, "only provider allocations read the port");
    debug_assert!(is_provider || query(request).is_none(), "host-local compute has no allocation query");
    let reading = reply.as_ref().map_or(
        lattice_core::execution_binding::admission::ProviderReading::NotRead,
        crate::execution_binding::reading,
    );
    match lattice_core::execution_binding::admission::admit(
        &request.execution_binding,
        &requirement(request),
        context(request),
        reading,
    ) {
        lattice_core::execution_binding::admission::Decision::Admitted(admitted) => {
            let observation = match reply {
                Some(Ok(crate::execution_binding::AllocationReply::Observed(observation))) => Some(*observation),
                Some(_) | None => None,
            };
            Ok((*admitted, observation))
        }
        decision @ (lattice_core::execution_binding::admission::Decision::Rejected(_)
        | lattice_core::execution_binding::admission::Decision::Blocked(_)) => Err(decision),
    }
}

/// Admit and persist before the intent. An unadmitted decision is printed and
/// nothing is written.
pub(super) fn admit_and_persist(
    request: &super::Request,
    directory: &std::path::Path,
    port: &mut dyn crate::execution_binding::ComputeAllocationPort,
) -> Result<Admitted, String> {
    let (admitted, observation) = admit(request, port).map_err(|decision| {
        println!("{}", serde_json::json!({ "execution_binding": decision }));
        "execution binding was not admitted; nothing was persisted".to_string()
    })?;
    persist(directory, &admitted, observation)?;
    Ok(admitted)
}

/// Write once. A repeated run must re-admit an identical binding identity; a
/// different binding is refused and the persisted one is never overwritten.
pub(super) fn persist(
    directory: &std::path::Path,
    admitted: &Admitted,
    observation: Option<Observation>,
) -> Result<(), String> {
    let persisted = Persisted {
        schema: PERSISTED_SCHEMA.to_owned(),
        binding: admitted.clone(),
        admission_observation: observation,
    };
    let bytes = serde_json::to_vec(&persisted).map_err(|error| error.to_string())?;
    let path = directory.join(ADMITTED_FILE);
    if super::artifacts::create_new(&path, &bytes)? {
        return Ok(());
    }
    let existing = read_persisted(&path)?;
    if existing.binding.binding_identity != admitted.binding_identity {
        return Err("attempt state holds a different execution binding; a new attempt identity is required".into());
    }
    Ok(())
}

fn read_persisted(path: &std::path::Path) -> Result<Persisted, String> {
    let bytes = super::artifacts::read_bounded(path, MAX_PERSISTED_BYTES)?;
    let persisted: Persisted =
        serde_json::from_slice(&bytes).map_err(|_| "persisted execution binding is malformed")?;
    let is_intact = persisted.schema == PERSISTED_SCHEMA
        && lattice_core::execution_binding::admission::bound::is_intact(&persisted.binding);
    if !is_intact {
        return Err("persisted execution binding is not intact".into());
    }
    Ok(persisted)
}

/// Load the retained binding and require that it binds this exact request.
pub(super) fn load(directory: &std::path::Path, request: &super::Request) -> Result<Option<Admitted>, String> {
    let path = directory.join(ADMITTED_FILE);
    if super::artifacts::read_optional(&path, MAX_PERSISTED_BYTES)?.is_none() {
        return Ok(None);
    }
    let admitted = read_persisted(&path)?.binding;
    let attempt = &admitted.attempt;
    let is_same_attempt = attempt.work_id == request.envelope.work_id
        && attempt.attempt_id == request.envelope.attempt_id
        && attempt.fence == request.envelope.fence;
    let is_same_profile = attempt.envelope_identity == request.admitted_envelope
        && admitted.profile_revision == request.envelope.worker_profile;
    if !is_same_attempt || !is_same_profile {
        return Err("persisted execution binding does not bind this request".into());
    }
    Ok(Some(admitted))
}

/// Re-observe and revalidate a provider binding immediately before spawn.
pub(super) fn revalidate(
    directory: &std::path::Path,
    admitted: &Admitted,
    port: &mut dyn crate::execution_binding::ComputeAllocationPort,
) -> Result<(), String> {
    let Err(record) = recheck::current(admitted, port) else {
        return Ok(());
    };
    let bytes = serde_json::to_vec(&record).map_err(|error| error.to_string())?;
    super::artifacts::write_once(&directory.join(REFUSAL_FILE), &bytes)?;
    println!("{record}");
    Err("execution binding failed pre-spawn revalidation; the worker was not dispatched".into())
}

/// The retained pre-spawn refusal, if any.
pub(super) fn refusal(directory: &std::path::Path) -> Result<Option<serde_json::Value>, String> {
    let Some(bytes) = super::artifacts::read_optional(&directory.join(REFUSAL_FILE), MAX_REFUSAL_BYTES)? else {
        return Ok(None);
    };
    let record: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "retained binding revalidation is malformed")?;
    if record.get("schema").and_then(serde_json::Value::as_str) != Some(REFUSAL_SCHEMA) {
        return Err("retained binding revalidation is malformed".into());
    }
    Ok(Some(record))
}
