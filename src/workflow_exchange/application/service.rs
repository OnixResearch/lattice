#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the composition root names its profile, handler, and error boundary explicitly"
)]

use super::ApplicationError;
use super::ApplicationHandler;
use super::HandlerProfile;

pub fn load_profile(path: &std::path::Path) -> Result<HandlerProfile, ApplicationError> {
    let value = crate::nickel::evaluate_file(path).map_err(|_| ApplicationError::Profile)?;
    let profile: HandlerProfile = serde_json::from_value(value).map_err(|_| ApplicationError::Profile)?;
    super::validate_handler_profile(&profile).map_err(|_| ApplicationError::Profile)?;
    Ok(profile)
}

// r[impl lattice.workflow_interchange.application_command]
#[cfg(target_os = "linux")]
pub fn serve(
    store: crate::store::Store,
    profile: HandlerProfile,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
) -> Result<lattice_workflow_transport::ServeReport, ApplicationError> {
    let socket_path = profile.socket_path.clone();
    let transport_profile = profile.transport.clone();
    let handler = ApplicationHandler::new(store, profile, agent_runtime)?;
    let mut service = lattice_workflow_transport::WorkflowExchangeService::new(handler);
    let endpoint = lattice_workflow_transport::UnixEndpoint {
        socket_path: &socket_path,
        profile: &transport_profile,
    };
    let report = lattice_workflow_transport::serve(endpoint, &mut service).map_err(|_| ApplicationError::Transport)?;
    service.handler_mut().wait_for_workers();
    Ok(report)
}

#[cfg(not(target_os = "linux"))]
pub fn serve(
    _store: crate::store::Store,
    _profile: HandlerProfile,
    _agent_runtime: crate::agent_node::AgentRuntimeConfig,
) -> Result<lattice_workflow_transport::ServeReport, ApplicationError> {
    Err(ApplicationError::UnsupportedHost)
}
