#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    reason = "explicit bounded profile and route branches reject untrusted operator and consumer values"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::WorkflowRequest;
use lattice_workflow_transport::TransportLimits;
use lattice_workflow_transport::validate_transport_profile;
use lattice_workflow_transport::validate_vsock_transport_profile;

use super::HandlerControls;
use super::HandlerListener;
use super::HandlerProfile;
use super::HandlerProfileV2;
use super::MAXIMUM_ROUTE_NODES;
use super::MAXIMUM_ROUTES;
use super::MAXIMUM_WORKFLOW_NAME_BYTES;
use super::ProfileError;
use super::REQUIRED_NON_CLAIMS;
use super::RouteError;
use super::WorkflowFacts;
use super::WorkflowRoute;

const HANDLER_PROFILE_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-handler-profile.v1";
const HANDLER_PROFILE_V2_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-handler-profile.v2";

// r[impl lattice.workflow_interchange.application_profile]
pub fn validate_handler_profile(profile: &HandlerProfile) -> Result<(), ProfileError> {
    if profile.schema != super::HANDLER_PROFILE_SCHEMA {
        return Err(ProfileError::Schema);
    }
    if !profile.socket_path.is_absolute() || profile.socket_path.as_os_str().is_empty() {
        return Err(ProfileError::SocketPath);
    }
    validate_transport_profile(&profile.transport).map_err(|_| ProfileError::Transport)?;
    validate_handler_body(HandlerBody {
        limits: profile.transport.limits,
        maximum_active_executions: profile.maximum_active_executions,
        controls: profile.controls,
        routes: &profile.routes,
        non_claims: &profile.non_claims,
    })
}

/// Validates a version-two profile: the version-one rules plus the selected listener.
/// Workflow pause and resume stay refused whatever the listener or VMM can do.
// r[impl lattice.workflow_interchange.handler_profile_v2]
pub fn validate_handler_profile_v2(profile: &HandlerProfileV2) -> Result<(), ProfileError> {
    if profile.schema != super::HANDLER_PROFILE_V2_SCHEMA {
        return Err(ProfileError::Schema);
    }
    let limits = match &profile.listener {
        HandlerListener::Unix { socket_path, transport } => {
            if !socket_path.is_absolute() || socket_path.as_os_str().is_empty() {
                return Err(ProfileError::SocketPath);
            }
            validate_transport_profile(transport).map_err(|_| ProfileError::Transport)?;
            transport.limits
        }
        HandlerListener::Vsock { transport } => {
            validate_vsock_transport_profile(transport).map_err(|_| ProfileError::Transport)?;
            transport.limits
        }
    };
    validate_handler_body(HandlerBody {
        limits,
        maximum_active_executions: profile.maximum_active_executions,
        controls: profile.controls,
        routes: &profile.routes,
        non_claims: &profile.non_claims,
    })
}

struct HandlerBody<'a> {
    limits: TransportLimits,
    maximum_active_executions: u32,
    controls: HandlerControls,
    routes: &'a [WorkflowRoute],
    non_claims: &'a [String],
}

fn validate_handler_body(body: HandlerBody<'_>) -> Result<(), ProfileError> {
    if body.maximum_active_executions == 0 || body.maximum_active_executions > body.limits.maximum_connections {
        return Err(ProfileError::ActiveLimit);
    }
    if body.controls.pause || body.controls.resume {
        return Err(ProfileError::Controls);
    }
    let route_count = u32::try_from(body.routes.len()).map_err(|_| ProfileError::RouteLimit)?;
    if route_count == 0 || route_count > MAXIMUM_ROUTES {
        return Err(ProfileError::RouteLimit);
    }
    let mut previous_route = None;
    for route in body.routes {
        let route_key = (route.profile_identity.as_str(), route.graph_identity.as_str());
        if previous_route.is_some_and(|previous| previous >= route_key) {
            return Err(ProfileError::RouteOrder);
        }
        previous_route = Some(route_key);
        validate_route_value(route)?;
    }
    if body.non_claims.len() != REQUIRED_NON_CLAIMS.len()
        || body.non_claims.iter().zip(REQUIRED_NON_CLAIMS).any(|(observed, required)| observed != required)
    {
        return Err(ProfileError::NonClaims);
    }
    Ok(())
}

fn validate_route_value(route: &WorkflowRoute) -> Result<(), ProfileError> {
    if route.workflow_name.is_empty()
        || route.workflow_name.len() > MAXIMUM_WORKFLOW_NAME_BYTES
        || !route
            .workflow_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ProfileError::RouteValue);
    }
    let node_count = u32::try_from(route.nodes.len()).map_err(|_| ProfileError::NodeValue)?;
    if node_count == 0 || node_count > MAXIMUM_ROUTE_NODES {
        return Err(ProfileError::NodeValue);
    }
    let mut previous_node = None;
    let mut workflow_nodes = BTreeSet::new();
    for node in &route.nodes {
        if previous_node.is_some_and(|previous| previous >= node.request_node_id.as_str()) {
            return Err(ProfileError::NodeOrder);
        }
        previous_node = Some(node.request_node_id.as_str());
        if !workflow_nodes.insert(node.workflow_node_id) {
            return Err(ProfileError::NodeValue);
        }
    }
    Ok(())
}

pub fn handler_profile_identity(profile: &HandlerProfile) -> Result<Identity, ProfileError> {
    validate_handler_profile(profile)?;
    super::core::framed_identity(HANDLER_PROFILE_IDENTITY_DOMAIN, profile).map_err(|_| ProfileError::Encoding)
}

// r[impl lattice.workflow_interchange.handler_profile_v2]
pub fn handler_profile_v2_identity(profile: &HandlerProfileV2) -> Result<Identity, ProfileError> {
    validate_handler_profile_v2(profile)?;
    super::core::framed_identity(HANDLER_PROFILE_V2_IDENTITY_DOMAIN, profile).map_err(|_| ProfileError::Encoding)
}

/// Either admitted handler profile version; each keeps its own schema, validation and identity
/// domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedHandlerProfile {
    V1(HandlerProfile),
    V2(HandlerProfileV2),
}

impl From<HandlerProfile> for VersionedHandlerProfile {
    fn from(profile: HandlerProfile) -> Self {
        Self::V1(profile)
    }
}

impl From<HandlerProfileV2> for VersionedHandlerProfile {
    fn from(profile: HandlerProfileV2) -> Self {
        Self::V2(profile)
    }
}

impl VersionedHandlerProfile {
    pub fn validate(&self) -> Result<(), ProfileError> {
        match self {
            Self::V1(profile) => validate_handler_profile(profile),
            Self::V2(profile) => validate_handler_profile_v2(profile),
        }
    }

    /// The identity under the profile's own version domain; a version-one identity is unchanged.
    pub fn identity(&self) -> Result<Identity, ProfileError> {
        match self {
            Self::V1(profile) => handler_profile_identity(profile),
            Self::V2(profile) => handler_profile_v2_identity(profile),
        }
    }

    #[must_use]
    pub const fn maximum_active_executions(&self) -> u32 {
        match self {
            Self::V1(profile) => profile.maximum_active_executions,
            Self::V2(profile) => profile.maximum_active_executions,
        }
    }

    #[must_use]
    pub fn routes(&self) -> &[WorkflowRoute] {
        match self {
            Self::V1(profile) => &profile.routes,
            Self::V2(profile) => &profile.routes,
        }
    }

    pub fn select_route(&self, request: &WorkflowRequest) -> Result<&WorkflowRoute, RouteError> {
        route_for(self.routes(), request)
    }
}

pub fn select_route<'a>(
    profile: &'a HandlerProfile,
    request: &WorkflowRequest,
) -> Result<&'a WorkflowRoute, RouteError> {
    route_for(&profile.routes, request)
}

fn route_for<'a>(routes: &'a [WorkflowRoute], request: &WorkflowRequest) -> Result<&'a WorkflowRoute, RouteError> {
    routes
        .iter()
        .find(|route| {
            route.profile_identity == request.profile_identity && route.graph_identity == request.graph.identity
        })
        .ok_or(RouteError::Missing)
}

pub fn validate_route(
    route: &WorkflowRoute,
    request: &WorkflowRequest,
    facts: &WorkflowFacts,
) -> Result<(), RouteError> {
    if route.workflow_revision != facts.revision {
        return Err(RouteError::WorkflowRevision);
    }
    match (route.source_binding, request.source.is_some()) {
        (super::SourceBindingMode::Unavailable, true) => return Err(RouteError::SourceUnexpected),
        (super::SourceBindingMode::ShellEnvironment, false) => return Err(RouteError::SourceRequired),
        (super::SourceBindingMode::Unavailable, false) | (super::SourceBindingMode::ShellEnvironment, true) => {}
    }
    if route.nodes.len() != request.graph.nodes.len() || facts.node_ids.len() != route.nodes.len() {
        return Err(RouteError::NodeCount);
    }
    let workflow_node_ids = facts.node_ids.iter().copied().collect::<BTreeSet<_>>();
    let mapping = route
        .nodes
        .iter()
        .map(|node| (node.request_node_id.as_str(), node.workflow_node_id))
        .collect::<BTreeMap<_, _>>();
    for (expected, mapped) in request.graph.nodes.iter().zip(&route.nodes) {
        if expected.id != mapped.request_node_id
            || expected.operation_identity != mapped.operation_identity
            || !workflow_node_ids.contains(&mapped.workflow_node_id)
        {
            return Err(RouteError::NodeMapping);
        }
    }
    let expected_edges = request
        .graph
        .edges
        .iter()
        .map(|edge| {
            let source = mapping.get(edge.source.as_str()).copied().ok_or(RouteError::EdgeMapping)?;
            let target = mapping.get(edge.target.as_str()).copied().ok_or(RouteError::EdgeMapping)?;
            Ok((source, target))
        })
        .collect::<Result<BTreeSet<_>, RouteError>>()?;
    let actual_edges = facts.edges.iter().copied().collect::<BTreeSet<_>>();
    if expected_edges != actual_edges {
        return Err(RouteError::EdgeMapping);
    }
    Ok(())
}
