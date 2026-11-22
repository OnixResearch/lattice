//! Explicit operator-selected standalone authority; absence remains fail-closed.

pub mod core;
mod shell;

pub use shell::OperatorRunContextSource;
pub use shell::SubjectRefusal;
pub use shell::check_subject;

/// Read-only requirements for authoring a profile, not an admitted grant or receipt.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProfileInspection {
    pub schema: String,
    pub workflow_revision: String,
    pub machine_identity: String,
    pub policy_ref: String,
    pub grants: Vec<core::Grant>,
}

// r[impl lattice.standalone_effect_authority.hosts]
// r[impl lattice.graph_construction.authority]
// r[impl lattice.graph_construction.authority.same_graph]
pub fn inspect_workflow(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<ProfileInspection, super::execution::Error> {
    let constructed = crate::authoring::construction::admit_runtime(workflow)
        .map_err(|error| super::execution::Error::Lowering(error.into()))?;
    if constructed.is_none() && !workflow.control_dependencies.is_empty() {
        return Err(super::execution::Error::Lowering("control dependencies require v3 admission".into()));
    }
    let portable = crate::workflow_format::workflow_to_portable(workflow);
    let machine = if let Some(admitted) = &constructed {
        super::execution::canonical_machine_identity(&admitted.machine)?
    } else {
        let lowered = crate::workflow_format::choregraph::lower_portable(&portable)
            .map_err(|error| super::execution::Error::Lowering(error.to_string()))?;
        super::execution::canonical_machine_identity(&lowered.machine)?
    };
    let revision =
        crate::orchestration::workflow_revision(workflow).map_err(|_| super::execution::Error::WorkflowRevision)?;
    let grants = if let Some(admitted) = &constructed {
        admitted
            .projection
            .effects
            .iter()
            .map(|effect| {
                let node = admitted
                    .projection
                    .nodes
                    .iter()
                    .find(|node| node.id == effect.node_id)
                    .ok_or(super::execution::Error::UnexpectedBindings)?;
                Ok(core::Grant {
                    node_id: effect.node_id,
                    operation: format!("lattice/{}", node.operation),
                    resource: effect.resource.clone(),
                    ability: effect.ability.clone(),
                })
            })
            .collect::<Result<Vec<_>, super::execution::Error>>()?
    } else {
        portable
            .configs
            .iter()
            .filter_map(|config| {
                let (prefix, ability) = crate::workflow_format::choregraph::effect_contract(config)?;
                Some(core::Grant {
                    node_id: config.node_id,
                    operation: format!("lattice/{}", config.kind),
                    resource: format!("{prefix}{}", config.node_id),
                    ability: ability.to_owned(),
                })
            })
            .collect()
    };
    Ok(ProfileInspection {
        schema: core::PROFILE_SCHEMA.to_owned(),
        workflow_revision: format!("blake3:{}", revision.content_hash),
        machine_identity: machine.as_str().to_owned(),
        policy_ref: super::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF.to_owned(),
        grants,
    })
}
