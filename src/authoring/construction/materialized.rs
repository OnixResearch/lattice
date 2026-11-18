//! Native v3 projection of a materialized shell workflow, for explicit forks and
//! expanded interchange. Pure: it creates no source or origin claim.

pub(super) fn projection(
    workflow: &crate::engine::Workflow,
    selected: choregraph_nickel::construction::model::host::TargetRegistry,
) -> Result<choregraph_nickel::construction::model::projection::LatticeV3, &'static str> {
    let (nodes, effects) = workflow
        .graph
        .nodes()
        .map(|node| shell_projection(workflow, node, &selected.operations))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();
    Ok(choregraph_nickel::construction::model::projection::LatticeV3 {
        schema: choregraph_nickel::construction::model::profile::PROJECTION_SCHEMA.into(),
        target: selected.target,
        registry: selected.profile,
        nodes,
        edges: data_edges(workflow),
        control_edges: control_edges(workflow),
        effects,
        interfaces: vec![],
    })
}

fn data_edges(workflow: &crate::engine::Workflow) -> Vec<choregraph_nickel::construction::model::projection::DataEdge> {
    workflow
        .graph
        .edges()
        .iter()
        .map(|edge| choregraph_nickel::construction::model::projection::DataEdge {
            source: edge.source.0,
            target: edge.target.0,
        })
        .collect()
}

fn control_edges(
    workflow: &crate::engine::Workflow,
) -> Vec<choregraph_nickel::construction::model::projection::ControlEdge> {
    workflow
        .control_dependencies
        .iter()
        .map(|&(source, target)| choregraph_nickel::construction::model::projection::ControlEdge { source, target })
        .collect()
}

fn shell_projection(
    workflow: &crate::engine::Workflow,
    node: &crate::engine::workflow::Node,
    signatures: &[choregraph_nickel::construction::model::host::OperationSignature],
) -> Result<
    (
        choregraph_nickel::construction::model::projection::ExpandedNode,
        choregraph_nickel::construction::model::projection::ExpandedEffect,
    ),
    &'static str,
> {
    let (command, is_stdin_piped) = profile_command(workflow, node)?;
    let operation = operation(node, is_stdin_piped)?;
    let signature = signatures
        .iter()
        .find(|signature| signature.operation == operation)
        .ok_or("selected shell signature missing")?;
    let scoped_id = format!("materialized/node-{}", node.id.0);
    let effect = invoke_effect(node.id.0, &scoped_id);
    let projected = choregraph_nickel::construction::model::projection::ExpandedNode {
        id: node.id.0,
        scoped_id,
        operation: operation.into(),
        implementation: signature.implementation.clone(),
        ports: node.input_ports.iter().chain(&node.output_ports).map(expanded_port).collect(),
        invocation: signature.invocation.clone(),
        configuration: std::collections::BTreeMap::from([(
            "command".into(),
            choregraph_nickel::construction::model::projection::ExpandedExpression::Literal {
                value: serde_json::Value::String(command.to_owned()),
            },
        )]),
    };
    debug_assert_eq!(projected.ports.len(), node.input_ports.len().saturating_add(node.output_ports.len()));
    debug_assert_eq!(effect.node_id, projected.id, "the invoke effect belongs to its node");
    Ok((projected, effect))
}

/// The shell command, when the node uses exactly the selected profile's configuration.
fn profile_command<'a>(
    workflow: &'a crate::engine::Workflow,
    node: &crate::engine::workflow::Node,
) -> Result<(&'a str, bool), &'static str> {
    let config = workflow.node_config(node.id).ok_or("constructed node config missing")?;
    let crate::engine::workflow::NodeConfig::Shell {
        command, pipe_stdin, ..
    } = config
    else {
        return Err("construction target only supports shell nodes");
    };
    let has_credentials = workflow.node_credentials(node.id).is_some_and(|credentials| !credentials.env.is_empty());
    if !is_profile_shell(config) || has_credentials {
        return Err("shell implementation configuration differs from selected profile");
    }
    if workflow.node_runner_settings(node.id).is_some_and(|settings| !is_profile_runner(settings, config)) {
        return Err("runtime binding or policy differs from selected shell profile");
    }
    Ok((command, *pipe_stdin))
}

/// A literal shell command with the profile's working directory, environment and timeout.
fn is_profile_shell(config: &crate::engine::workflow::NodeConfig) -> bool {
    let crate::engine::workflow::NodeConfig::Shell {
        command,
        cwd,
        env,
        timeout_secs,
        ..
    } = config
    else {
        return false;
    };
    let has_profile_context = cwd.is_none() && env.is_empty();
    has_profile_context && *timeout_secs == super::SHELL_TIMEOUT_SECS && !command.contains("{{")
}

/// The selected profile binds the default shell runtime and declares no runner policy.
fn is_profile_runner(
    settings: &crate::engine::workflow::NodeRunnerSettings,
    config: &crate::engine::workflow::NodeConfig,
) -> bool {
    let actual = crate::engine::runner::runtime_binding_for_config(config);
    let has_policy = !settings.effect_policy.is_empty()
        || !settings.cache_policy.is_empty()
        || !settings.retry_policy.is_empty()
        || !settings.artifact_policy.is_empty();
    settings.runtime_kind == actual.kind && settings.runtime_key == actual.key && !has_policy
}

fn operation(node: &crate::engine::workflow::Node, is_stdin_piped: bool) -> Result<&'static str, &'static str> {
    match (node.input_ports.len(), is_stdin_piped) {
        (0, false) => Ok("shell-source"),
        (1, true) => Ok("shell-stdin"),
        _ => Err("shell input boundary differs from selected signature"),
    }
}

fn invoke_effect(node_id: u64, scoped_id: &str) -> choregraph_nickel::construction::model::projection::ExpandedEffect {
    choregraph_nickel::construction::model::projection::ExpandedEffect {
        declaration_key: format!("{scoped_id}/invoke"),
        node_id,
        from_state: "ready".into(),
        action: "start".into(),
        resource: super::RESOURCE.into(),
        ability: super::ABILITY.into(),
        request_binding: choregraph_nickel::construction::model::projection::ExpandedBinding::Slot {
            slot_id: format!("{scoped_id}/dispatch-request"),
            source_ref: format!("node/{node_id}/invocation/dispatch-request"),
        },
        payload_binding: choregraph_nickel::construction::model::projection::ExpandedBinding::Slot {
            slot_id: format!("{scoped_id}/resolved-payload"),
            source_ref: format!("node/{node_id}/invocation/resolved-payload"),
        },
    }
}

fn expanded_port(
    port: &crate::engine::workflow::Port,
) -> choregraph_nickel::construction::model::projection::ExpandedPort {
    choregraph_nickel::construction::model::projection::ExpandedPort {
        id: port.id.0,
        name: port.label.clone(),
        value_type: port.type_tag.clone(),
        direction: match port.direction {
            crate::engine::workflow::PortDirection::Input => {
                choregraph_nickel::construction::model::interface::Direction::Input
            }
            crate::engine::workflow::PortDirection::Output => {
                choregraph_nickel::construction::model::interface::Direction::Output
            }
        },
    }
}
