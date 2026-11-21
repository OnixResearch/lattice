//! `implementation-attempt inspect`: `lattice.worker-inventory.v3` with
//! separate profile and instance facts.
//!
//! The command reads one attempt directory without creating it, takes no
//! claim, schedules nothing and never calls the compute port. It reads only
//! the latest snapshots of an injected serving-owner port, never admission or
//! recheck; without one, serving readiness names that cause. With
//! `--revision-store` it reads the definition's revision lifecycle without
//! writing it; without one every revision slot is `unmanaged`. Paths,
//! endpoints, prompts, commands, credential and grant references and backend
//! handles are not printed. An unreadable or contradictory artifact, a
//! malformed capability declaration or a managed attempt inspected without
//! its store is an error, never a guessed state.

// r[impl lattice.execution_binding.instances.scenario.projection]
// r[impl lattice.execution_binding.instances.scenario.unknown]
// r[impl lattice.worker_inventory.schema.scenario.cutover]
// r[impl lattice.worker_inventory.capability.scenario.redacted]
// r[impl lattice.worker_inventory.revisions.scenario.managed]

#[cfg(test)]
mod tests;

const INVENTORY_SCHEMA: &str = "lattice.worker-inventory.v3";
const MAX_SMALL_ARTIFACT_BYTES: usize = 262_144;

pub(super) fn run(
    request_path: &std::path::Path,
    state: &std::path::Path,
    serving: &mut dyn crate::execution_binding::serving::OwnerPort,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    println!("{}", project(request_path, state, serving, revisions)?);
    Ok(())
}

// r[impl lattice.serving_binding.readiness]
pub(super) fn project(
    request_path: &std::path::Path,
    state: &std::path::Path,
    serving: &mut dyn crate::execution_binding::serving::OwnerPort,
    revisions: super::session::revision::Store<'_>,
) -> Result<serde_json::Value, String> {
    let (request, bytes) = super::artifacts::load_request(request_path)?;
    request.envelope.validate().map_err(|error| error.to_string())?;
    request.validate_binding()?;
    let readiness = crate::execution_binding::serving::readiness::observe(
        request.execution_binding.serving.as_ref(),
        &request.envelope.attempt_id,
        serving,
    )?;
    let lifecycles = revisions.lifecycles(&request)?;
    let profile = profile_facts(&request, &readiness, lifecycles.profile())?;
    let identity = blake3::hash(&bytes).to_hex().to_string();
    debug_assert!(readiness.is_bounded(), "readiness comes from validated producer values");
    debug_assert_eq!(profile.serving_readiness, readiness, "profile and instance share one port read");
    let directory = super::artifacts::directory(state, &request);
    let instance = instance(&request, &identity, &profile, &directory, Retained {
        revisions,
        lifecycle: lifecycles.session(),
    })?;
    let model_requests = model_requests(&request, &directory)?;
    Ok(serde_json::json!({
        "schema": INVENTORY_SCHEMA,
        "profile": profile,
        "instance": instance,
        "model_requests": model_requests,
    }))
}

/// The retained per-request summary, read-only; `None` before any binding.
// r[impl lattice.serving_binding.overrun]
fn model_requests(
    request: &super::Request,
    directory: &std::path::Path,
) -> Result<Option<lattice_core::execution_binding::serving::usage::ModelRequestCoverage>, String> {
    if !is_present(directory)? {
        return Ok(None);
    }
    let Some(admitted) = super::binding::load(directory, request)? else {
        return Ok(None);
    };
    super::binding::requests::coverage(&admitted, directory).map(Some)
}

/// OMP attempt-worker facts; every value comes from the admitted request and
/// the definition's revision lifecycle.
pub(super) fn profile_facts(
    request: &super::Request,
    serving_readiness: &lattice_core::execution_binding::projection::ServingReadiness,
    lifecycle: Option<&lattice_core::agent_revision::state::Lifecycle>,
) -> Result<lattice_core::execution_binding::projection::inventory::WorkerProfileFacts, String> {
    debug_assert_eq!(request.schema, super::REQUEST_SCHEMA, "profile facts follow v3 validation");
    debug_assert_eq!(request.admitted_envelope, request.envelope.identity());
    let declared = lattice_core::execution_binding::projection::Offered::Declared;
    let model_routes = request.execution_binding.model_routes.clone();
    lattice_core::execution_binding::projection::inventory::project_profile(
        &lattice_core::execution_binding::projection::inventory::ProfileInput {
            owner: lattice_core::execution_binding::projection::inventory::ProfileOwner::ImplementationAttempt,
            revision: &request.envelope.worker_profile,
            default_definition: lattice_core::execution_binding::projection::inventory::ATTEMPT_DEFINITION,
            capability: request.worker_capability.as_ref(),
            lifecycle,
            adapter: declared("omp-jsonl"),
            protocol_revision: declared(&request.omp.revision),
            executable_identity: declared(&request.omp.executable_blake3),
            inputs: declared("workspace-snapshot"),
            outputs: declared("candidate-receipt"),
            sessions: declared("single-shot-no-session"),
            checkpoint: lattice_core::execution_binding::projection::Offered::Unavailable,
            workspace: declared("admitted-read-write-paths"),
            network: declared("attempt-local-model-broker"),
            isolation: declared("required-sandbox"),
            deadline_secs: lattice_core::execution_binding::projection::Offered::Declared(
                request.envelope.budget.max_seconds,
            ),
            max_output_bytes: lattice_core::execution_binding::projection::Offered::Declared(
                u64::try_from(request.output_limit_bytes).map_err(|error| error.to_string())?,
            ),
            model_routes: lattice_core::execution_binding::projection::Offered::Declared(&model_routes),
            compute: lattice_core::execution_binding::projection::Offered::Declared(&request.worker_compute),
            serving_readiness,
        },
    )
    .map_err(|defect| format!("worker profile facts are unsupported: {defect:?}"))
}

const NAMED_ARTIFACTS: [&str; 7] = [
    super::binding::ADMITTED_FILE,
    super::session::intent::FILE,
    super::binding::REFUSAL_FILE,
    super::session::revision::BLOCK_FILE,
    "cancel.request",
    "raw-events.jsonl",
    "omp-observation.json",
];

/// The revision store and the session subject's lifecycle: the worker
/// definition's, read once for both profile and instance, or the named
/// workflow-agent definition's.
struct Retained<'a> {
    revisions: super::session::revision::Store<'a>,
    lifecycle: Option<&'a lattice_core::agent_revision::state::Lifecycle>,
}

fn instance(
    request: &super::Request,
    request_identity: &str,
    profile: &lattice_core::execution_binding::projection::inventory::WorkerProfileFacts,
    directory: &std::path::Path,
    retained: Retained<'_>,
) -> Result<Option<lattice_core::execution_binding::projection::retained::WorkerInstanceObservation>, String> {
    if !is_present(directory)? {
        return Ok(None);
    }
    let admitted = super::binding::load(directory, request)?;
    let identities = artifact_identities(directory, request.output_limit_bytes)?;
    let present = |name: &str| identities.iter().any(|(artifact, _)| *artifact == name);
    let session = super::session::revision::identifier(request);
    let subject = super::session::intent::Subject {
        request_identity,
        session: &session,
    };
    let recorded = match super::session::intent::read(directory, subject)? {
        None => None,
        Some(super::session::intent::Record::Owned(recorded)) => Some(recorded),
        // A refusal before the intent, or recovery, closed the attempt before
        // any execution: it has no worker instance.
        Some(super::session::intent::Record::Closed) => return Ok(None),
        Some(super::session::intent::Record::Foreign) => {
            return Err("retained intent belongs to a different request".into());
        }
    };
    if let Some(recorded) = &recorded {
        retained.revisions.require(recorded)?;
    }
    let admission = recorded.as_ref().map_or(
        lattice_core::execution_binding::projection::lineage::RecordedAdmission::Unrecorded,
        super::session::intent::Recorded::inventory_admission,
    );
    let pinned = lattice_core::execution_binding::projection::lineage::pinned(retained.lifecycle, &session, admission);
    let disposition = terminal_disposition(directory, &request.envelope.attempt_id)?;
    let pairs = identities.iter().map(|(name, identity)| (*name, identity.as_str())).collect::<Vec<_>>();
    debug_assert!(identities.len() <= NAMED_ARTIFACTS.len());
    debug_assert_eq!(present(super::binding::ADMITTED_FILE), admitted.is_some());
    let state = lattice_core::execution_binding::projection::retained::AttemptState {
        admitted: admitted.as_ref(),
        request_identity,
        definition_identity: &profile.definition.identity,
        admitted_until_epoch_ms: request.expires_at_ms,
        has_intent: recorded.is_some(),
        has_refusal: present(super::binding::REFUSAL_FILE),
        has_revision_block: present(super::session::revision::BLOCK_FILE),
        terminal_disposition: disposition.as_deref(),
        has_cancel_request: present("cancel.request"),
        artifacts: &pairs,
        serving_readiness: &profile.serving_readiness,
        pinned: &pinned,
    };
    let observed_at_epoch_ms = i64::try_from(super::now_ms()?).map_err(|error| error.to_string())?;
    lattice_core::execution_binding::projection::retained::project_instance(&state, observed_at_epoch_ms)
        .map_err(|defect| format!("retained attempt state is inconsistent: {defect:?}"))
}

fn artifact_identities(
    directory: &std::path::Path,
    output_limit_bytes: usize,
) -> Result<Vec<(&'static str, String)>, String> {
    let mut identities = Vec::with_capacity(NAMED_ARTIFACTS.len());
    for name in NAMED_ARTIFACTS {
        let bound_bytes = if name == "raw-events.jsonl" {
            output_limit_bytes
        } else {
            MAX_SMALL_ARTIFACT_BYTES
        };
        if let Some(bytes) = super::artifacts::read_optional(&directory.join(name), bound_bytes)? {
            identities.push((name, blake3::hash(&bytes).to_hex().to_string()));
        }
    }
    Ok(identities)
}

/// The published disposition label, or `None` while no terminal result exists.
fn terminal_disposition(directory: &std::path::Path, attempt_id: &str) -> Result<Option<String>, String> {
    let store_path = directory.join("candidate.redb");
    if !is_present(&store_path)? {
        return Ok(None);
    }
    let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
    let crate::candidate_worker::Recovery::Published(result) =
        crate::candidate_worker::reconcile_attempt(&store, attempt_id).map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    let label = serde_json::to_value(result.disposition).map_err(|error| error.to_string())?;
    Ok(label.as_str().map(str::to_owned))
}

/// Only a missing path is absent; every other metadata failure is an error.
fn is_present(path: &std::path::Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("retained attempt state is unreadable".into()),
    }
}
