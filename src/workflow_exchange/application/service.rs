#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the composition root names its profile, handler, and error boundary explicitly"
)]

use super::ApplicationError;
use super::ApplicationHandler;
#[cfg(target_os = "linux")]
use super::HandlerListener;
use super::HandlerProfile;
use super::VersionedHandlerProfile;

/// Loads a typed Nickel handler profile. The evaluated `schema` selects version
/// two; every other value is decoded and validated as version one.
// r[impl lattice.workflow_interchange.handler_profile_v2]
pub fn load_profile(path: &std::path::Path) -> Result<VersionedHandlerProfile, ApplicationError> {
    let value = crate::nickel::evaluate_file(path).map_err(|_| ApplicationError::Profile)?;
    let is_version_two =
        value.get("schema").and_then(serde_json::Value::as_str) == Some(super::HANDLER_PROFILE_V2_SCHEMA);
    let profile = if is_version_two {
        VersionedHandlerProfile::V2(serde_json::from_value(value).map_err(|_| ApplicationError::Profile)?)
    } else {
        let profile: HandlerProfile = serde_json::from_value(value).map_err(|_| ApplicationError::Profile)?;
        VersionedHandlerProfile::V1(profile)
    };
    profile.validate().map_err(|_| ApplicationError::Profile)?;
    Ok(profile)
}

/// The listener a profile selects: version one is always the Unix socket.
#[cfg(target_os = "linux")]
fn selected_listener(profile: &VersionedHandlerProfile) -> HandlerListener {
    match profile {
        VersionedHandlerProfile::V1(profile) => HandlerListener::Unix {
            socket_path: profile.socket_path.clone(),
            transport: profile.transport.clone(),
        },
        VersionedHandlerProfile::V2(profile) => profile.listener.clone(),
    }
}

// r[impl lattice.workflow_interchange.application_command]
// r[impl lattice.workflow_interchange.guest_listener]
/// Serve one handler profile. With a checkpoint binding the bound route publishes checkpoints,
/// and adopted executions that never started run here, observable and cancellable through this
/// listener like fresh ones.
#[cfg(target_os = "linux")]
pub fn serve(
    store: crate::store::Store,
    profile: impl Into<VersionedHandlerProfile>,
    agent_runtime: crate::agent_node::RuntimeOptions,
    checkpoint: Option<crate::checkpoint::CheckpointBinding>,
) -> Result<lattice_workflow_transport::ServeReport, ApplicationError> {
    let profile = profile.into();
    let listener = selected_listener(&profile);
    let handler = served_handler(ApplicationHandler::new(store, profile, agent_runtime)?, checkpoint)?;
    let mut service = lattice_workflow_transport::WorkflowExchangeService::new(handler);
    let served = match &listener {
        HandlerListener::Unix { socket_path, transport } => {
            let endpoint = lattice_workflow_transport::UnixEndpoint {
                socket_path,
                profile: transport,
            };
            lattice_workflow_transport::serve(endpoint, &mut service)
        }
        HandlerListener::Vsock { transport } => lattice_workflow_transport::serve_vsock(transport, &mut service),
    };
    service.handler_mut().wait_for_workers();
    served.map_err(|_| ApplicationError::Transport)
}

// r[impl lattice.application_checkpoint.resume]
/// Bind the served route's checkpoint and start the adopted executions before any connection.
#[cfg(target_os = "linux")]
fn served_handler(
    handler: ApplicationHandler,
    checkpoint: Option<crate::checkpoint::CheckpointBinding>,
) -> Result<ApplicationHandler, ApplicationError> {
    let mut handler = match checkpoint {
        Some(binding) => handler.with_application_checkpoint(binding)?,
        None => handler,
    };
    handler.resume_adopted()?;
    Ok(handler)
}

#[cfg(not(target_os = "linux"))]
pub fn serve(
    _store: crate::store::Store,
    _profile: impl Into<VersionedHandlerProfile>,
    _agent_runtime: crate::agent_node::RuntimeOptions,
    _checkpoint: Option<crate::checkpoint::CheckpointBinding>,
) -> Result<lattice_workflow_transport::ServeReport, ApplicationError> {
    Err(ApplicationError::UnsupportedHost)
}
