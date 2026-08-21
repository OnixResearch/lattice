#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    reason = "the small pure binder checks every fixed source field and names product workflow paths explicitly"
)]

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::WorkflowRequest;

use super::RouteError;
use super::SourceBindingMode;
use super::WorkflowRoute;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;

pub const SOURCE_PROVIDER_ENV: &str = "LATTICE_SOURCE_PROVIDER";
pub const SOURCE_REPOSITORY_ENV: &str = "LATTICE_SOURCE_REPOSITORY";
pub const SOURCE_REVISION_ENV: &str = "LATTICE_SOURCE_REVISION";

// r[impl lattice.workflow_interchange.source_binding]
// r[impl lattice.workflow_interchange.source_authority]
pub fn instantiate_workflow(
    route: &WorkflowRoute,
    request: &WorkflowRequest,
    mut workflow: Workflow,
) -> Result<(Workflow, Identity), RouteError> {
    match route.source_binding {
        SourceBindingMode::Unavailable => {
            if request.source.is_some() {
                return Err(RouteError::SourceUnexpected);
            }
        }
        SourceBindingMode::ShellEnvironment => {
            let source = request.source.as_ref().ok_or(RouteError::SourceRequired)?;
            for node in &route.nodes {
                let config = workflow.configs.get_mut(&node.workflow_node_id).ok_or(RouteError::SourceTarget)?;
                let NodeConfig::Shell { env, .. } = config else {
                    return Err(RouteError::SourceTarget);
                };
                if env.contains_key(SOURCE_PROVIDER_ENV)
                    || env.contains_key(SOURCE_REPOSITORY_ENV)
                    || env.contains_key(SOURCE_REVISION_ENV)
                {
                    return Err(RouteError::SourceCollision);
                }
                env.insert(SOURCE_PROVIDER_ENV.to_string(), source.provider.as_str().to_string());
                env.insert(SOURCE_REPOSITORY_ENV.to_string(), source.repository.as_str().to_string());
                env.insert(SOURCE_REVISION_ENV.to_string(), source.revision.as_str().to_string());
            }
        }
    }
    let revision = super::application::workflow_facts(&workflow).map_err(|_| RouteError::WorkflowRevision)?.revision;
    Ok((workflow, revision))
}
