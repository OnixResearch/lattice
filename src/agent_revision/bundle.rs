//! Resolve one inert bundle from an exported implementation-attempt request.
//!
//! The revision is `Request::worker_profile_identity()`, used verbatim as an
//! opaque bare-hex value. The definition is the inventory's definition
//! identity. Resolution reads only the request; it admits nothing, calls no
//! port and writes nothing.

// r[impl lattice.agent_revisions.bundle]
// r[impl lattice.agent_revisions.bundle.scenario.hosted]

/// The OMP attempt worker's adapter name.
pub const ADAPTER: &str = "omp";
/// The single brokered route of the OMP attempt worker.
pub const PRIMARY_ROUTE: &str = "primary";
const MAX_REQUEST_BYTES: u64 = 2_097_152;

/// The definition, opaque revision and inert bundle of one request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Resolved {
    pub definition: lattice_core::execution_binding::projection::lineage::Definition,
    pub revision: String,
    pub bundle: lattice_core::agent_revision::facts::Bundle,
    pub bundle_identity: String,
}

/// Read a bounded exported request.
pub fn load(path: &std::path::Path) -> Result<crate::implementation_attempt::cli::Request, String> {
    let length_bytes = std::fs::metadata(path).map_err(|error| error.to_string())?.len();
    if length_bytes > MAX_REQUEST_BYTES {
        return Err("implementation-attempt request exceeds its size bound".to_owned());
    }
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| format!("implementation-attempt request is malformed: {error}"))
}

fn declared(set: Option<&lattice_core::execution_binding::capability::NameSet>) -> Option<Vec<String>> {
    match set {
        Some(lattice_core::execution_binding::capability::NameSet::Declared { names }) => Some(names.clone()),
        Some(lattice_core::execution_binding::capability::NameSet::Unavailable {}) | None => None,
    }
}

fn hosted_route(
    request: &crate::implementation_attempt::cli::Request,
) -> Result<lattice_core::agent_revision::facts::HostedRoute, String> {
    let broker = &request.omp.model_broker;
    let credential = broker.credential.as_ref();
    let route_identity = lattice_core::content_identity::model_route(lattice_core::content_identity::ModelRouteInput {
        schema: &broker.schema,
        endpoint: &broker.endpoint,
        model: &broker.model,
        credential_provider: credential.map(|reference| reference.provider.as_str()),
        credential_key: credential.map(|reference| reference.key.as_str()),
        credential_field: credential.and_then(|reference| reference.field.as_deref()),
    })
    .map_err(|error| error.to_string())?;
    let route = lattice_core::agent_revision::facts::HostedRoute {
        name: PRIMARY_ROUTE.to_owned(),
        route_identity,
        provider_model: broker.model.clone(),
        weight_identity: lattice_core::agent_revision::facts::Unexposed::Unknown,
        model_version: lattice_core::agent_revision::facts::Unexposed::Unknown,
    };
    debug_assert!(lattice_core::agent_revision::is_reference(&route.route_identity));
    debug_assert_eq!(route.weight_identity, lattice_core::agent_revision::facts::Unexposed::Unknown);
    Ok(route)
}

/// Resolve the request's definition, revision and bundle.
pub fn resolve(request: &crate::implementation_attempt::cli::Request) -> Result<Resolved, String> {
    let revision = request.worker_profile_identity()?;
    let capability = request
        .worker_capability
        .as_ref()
        .map(lattice_core::execution_binding::capability::Declaration::canonical)
        .transpose()
        .map_err(|defect| format!("worker capability declaration is unsupported: {defect:?}"))?;
    let definition = lattice_core::execution_binding::projection::lineage::definition(
        lattice_core::execution_binding::projection::inventory::ProfileOwner::ImplementationAttempt,
        lattice_core::execution_binding::projection::inventory::ATTEMPT_DEFINITION,
        capability.as_ref().map(|declared| declared.definition.as_str()),
    )
    .map_err(|_| "worker definition name is unsupported".to_owned())?;
    let bundle = lattice_core::agent_revision::facts::Bundle {
        schema: lattice_core::agent_revision::facts::BUNDLE_SCHEMA.to_owned(),
        definition: definition.identity.clone(),
        definition_name: definition.name.clone(),
        revision: revision.clone(),
        adapter: ADAPTER.to_owned(),
        protocol_revision: request.omp.revision.clone(),
        executable_identity: request.omp.executable_blake3.clone(),
        compute: request.worker_compute.clone(),
        task_families: declared(capability.as_ref().and_then(|declared| declared.task_families.as_ref())),
        tools: declared(capability.as_ref().and_then(|declared| declared.tools.as_ref())),
        routes: vec![hosted_route(request)?],
        workflow: None,
    };
    let bundle_identity = lattice_core::agent_revision::facts::identity(&bundle)
        .map_err(|refusal| format!("resolved bundle is unsupported: {refusal:?}"))?;
    debug_assert_eq!(bundle.revision, revision);
    debug_assert_eq!(bundle.definition, definition.identity);
    Ok(Resolved {
        definition,
        revision,
        bundle,
        bundle_identity,
    })
}
