//! Native construction compilation, lowering of its v3 projection into a Lattice
//! shell workflow, and origin attribution through the retained port map. Its only
//! effects select and recheck the running runtime binding.

/// One native compilation: workflow, projection, compilation binding and origins.
pub(in crate::authoring) type Compiled = (
    crate::engine::Workflow,
    choregraph_nickel::construction::model::projection::LatticeV3,
    choregraph_nickel::construction::association::CompilationBinding,
    Vec<choregraph_nickel::construction::model::projection::Origin>,
);

// r[impl lattice.graph_construction.closure]
// r[impl lattice.graph_construction.rejection.overflow]
pub(in crate::authoring) fn compile(
    observation: &super::super::EvaluatorObservation,
) -> Result<Compiled, &'static str> {
    observation.validate()?;
    if !selects_reviewed_library(&observation.modules) {
        return Err("selected construction library or contract differs");
    }
    let selected = super::registry();
    let output = native_output(observation, &selected)?;
    super::admit_projection(&output.projection, &selected).map_err(|_| "materialized projection admission failed")?;
    let candidate: choregraph_nickel::construction::model::program::Candidate =
        serde_json::from_str(&observation.result.json).map_err(|_| "construction candidate is invalid")?;
    let workflow = to_workflow(&output.projection, &candidate.root)?;
    Ok((workflow, output.projection, output.binding, output.origins))
}

/// Compiles the observed candidate and checks that its binding covers exactly these materials.
fn native_output(
    observation: &super::super::EvaluatorObservation,
    selected: &choregraph_nickel::construction::model::host::TargetRegistry,
) -> Result<choregraph_nickel::construction::Output, &'static str> {
    let modules = captured_modules(&observation.modules);
    let binding = serde_json::to_vec(&observation.result.binding).map_err(|_| "evaluator binding encoding failed")?;
    let materials = materials(observation, &modules, &binding);
    let candidate = observation.result.json.as_bytes();
    let output = choregraph_nickel::construction::compile_json(candidate, selected, &materials, super::LIMITS)
        .map_err(|_| "construction admission failed")?;
    choregraph_nickel::construction::association::validate_compilation_binding(
        &output,
        candidate,
        selected,
        &materials,
        super::LIMITS,
    )
    .map_err(|_| "construction binding changed")?;
    Ok(output)
}

fn captured_modules(
    modules: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Vec<choregraph_nickel::construction::association::CapturedModule<'_>> {
    modules
        .iter()
        .map(|(logical_id, bytes)| choregraph_nickel::construction::association::CapturedModule { logical_id, bytes })
        .collect()
}

fn selects_reviewed_library(modules: &std::collections::BTreeMap<String, Vec<u8>>) -> bool {
    let library = modules.get(choregraph_nickel::construction::model::profile::LIBRARY_MODULE).map(Vec::as_slice);
    let contracts = modules.get(choregraph_nickel::construction::model::profile::CONTRACTS_MODULE).map(Vec::as_slice);
    library == Some(choregraph_nickel::construction::model::profile::LIBRARY_BYTES)
        && contracts == Some(choregraph_nickel::construction::model::profile::CONTRACTS_BYTES)
}

fn materials<'a>(
    observation: &'a super::super::EvaluatorObservation,
    modules: &'a [choregraph_nickel::construction::association::CapturedModule<'a>],
    evaluator_binding: &'a [u8],
) -> choregraph_nickel::construction::association::CompilationMaterials<'a> {
    choregraph_nickel::construction::association::CompilationMaterials {
        modules,
        entry: &observation.entry,
        evaluator_identity: &observation.worker_artifact,
        evaluator_profile: crate::nickel::CAPTURED_PROFILE,
        evaluator_binding,
        evaluated_output_blake3: &observation.result.binding.output_blake3,
        library: choregraph_nickel::construction::model::profile::LIBRARY_BYTES,
        contracts: choregraph_nickel::construction::model::profile::CONTRACTS_BYTES,
        lowerer_identity: env!("LATTICE_WORKFLOW_LOWERER_BLAKE3"),
        toolchain_identity: env!("LATTICE_DEPENDENCY_LOCK_BLAKE3"),
    }
}

// r[impl lattice.graph_construction.reference]
// r[impl lattice.graph_construction.reference.values]
// r[impl lattice.graph_construction.instances]
// r[impl lattice.graph_construction.instances.remap]
// r[impl lattice.graph_construction.control_order]
/// Lowers an already admitted projection; the caller owns v3 projection admission.
fn to_workflow(
    projection: &choregraph_nickel::construction::model::projection::LatticeV3,
    name: &str,
) -> Result<crate::engine::Workflow, &'static str> {
    let mut workflow = crate::engine::Workflow::new(name, crate::engine::workflow::WorkflowTimestamp::epoch());
    workflow.construction_profile = true;
    workflow.construction_runtime = Some(super::selected_runtime()?);
    install_nodes(&mut workflow, &projection.nodes)?;
    for edge in &projection.edges {
        workflow
            .graph
            .add_edge(crate::engine::workflow::PortId(edge.source), crate::engine::workflow::PortId(edge.target))
            .map_err(|_| "constructed graph edge rejected")?;
    }
    workflow.control_dependencies = control_order(projection)?;
    let diagnostics = crate::workflow_validation::validate_workflow(&workflow);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err("constructed workflow validation failed");
    }
    Ok(workflow)
}

fn install_nodes(
    workflow: &mut crate::engine::Workflow,
    nodes: &[choregraph_nickel::construction::model::projection::ExpandedNode],
) -> Result<(), &'static str> {
    let (nodes, configs): (Vec<_>, Vec<_>) =
        nodes.iter().map(shell_node).collect::<Result<Vec<_>, _>>()?.into_iter().unzip();
    for (node, config) in nodes.iter().zip(configs) {
        workflow.set_node_config(node.id, config);
    }
    let nodes = nodes.into_iter().map(|node| (node.id, node)).collect();
    workflow.graph = crate::engine::workflow::WorkflowGraph::from_admitted_nodes(nodes)
        .map_err(|_| "constructed graph identities are invalid")?;
    Ok(())
}

fn shell_node(
    node: &choregraph_nickel::construction::model::projection::ExpandedNode,
) -> Result<(crate::engine::workflow::Node, crate::engine::workflow::NodeConfig), &'static str> {
    let command = literal_command(node)?;
    let is_source = match node.operation.as_str() {
        "shell-source" => true,
        "shell-stdin" => false,
        _ => return Err("unknown shell implementation"),
    };
    let config = crate::engine::workflow::NodeConfig::Shell {
        command: command.to_owned(),
        cwd: None,
        env: std::collections::HashMap::new(),
        timeout_secs: super::SHELL_TIMEOUT_SECS,
        pipe_stdin: !is_source,
    };
    let ports = |direction| node.ports.iter().filter(|port| port.direction == direction).map(runtime_port).collect();
    let lowered = crate::engine::workflow::Node {
        id: crate::engine::workflow::NodeId(node.id),
        label: node.scoped_id.clone(),
        x: 0,
        y: 0,
        input_ports: ports(choregraph_nickel::construction::model::interface::Direction::Input),
        output_ports: ports(choregraph_nickel::construction::model::interface::Direction::Output),
    };
    debug_assert!(!command.contains("{{"), "runtime expression references never reach a shell command");
    debug_assert_eq!(lowered.input_ports.iter().chain(&lowered.output_ports).count(), node.ports.len());
    Ok((lowered, config))
}

fn literal_command(
    node: &choregraph_nickel::construction::model::projection::ExpandedNode,
) -> Result<&str, &'static str> {
    const UNSUPPORTED: &str = "shell command must be a literal without runtime expression references";
    let Some(choregraph_nickel::construction::model::projection::ExpandedExpression::Literal {
        value: serde_json::Value::String(text),
    }) = node.configuration.get("command")
    else {
        return Err(UNSUPPORTED);
    };
    if text.contains("{{") {
        return Err(UNSUPPORTED);
    }
    Ok(text)
}

fn runtime_port(
    port: &choregraph_nickel::construction::model::projection::ExpandedPort,
) -> crate::engine::workflow::Port {
    crate::engine::workflow::Port {
        id: crate::engine::workflow::PortId(port.id),
        label: port.name.clone(),
        type_tag: port.value_type.clone(),
        direction: match port.direction {
            choregraph_nickel::construction::model::interface::Direction::Input => {
                crate::engine::workflow::PortDirection::Input
            }
            choregraph_nickel::construction::model::interface::Direction::Output => {
                crate::engine::workflow::PortDirection::Output
            }
        },
    }
}

fn control_order(
    projection: &choregraph_nickel::construction::model::projection::LatticeV3,
) -> Result<Vec<(u64, u64)>, &'static str> {
    let control = projection
        .control_edges
        .iter()
        .map(|edge| (edge.source, edge.target))
        .collect::<std::collections::BTreeSet<_>>();
    if control.len() != projection.control_edges.len() {
        return Err("duplicate control dependency");
    }
    Ok(control.into_iter().collect())
}

/// Runtime port ID to native projection port ID, matched by position, name and type.
pub(in crate::authoring) fn port_map(
    workflow: &crate::engine::Workflow,
    projection: &choregraph_nickel::construction::model::projection::LatticeV3,
) -> Result<std::collections::BTreeMap<u64, u64>, &'static str> {
    let mut pairs = Vec::with_capacity(projection.nodes.iter().map(|node| node.ports.len()).sum());
    for node in &projection.nodes {
        pairs.extend(node_ports(workflow, node)?);
    }
    let map = pairs.iter().copied().collect::<std::collections::BTreeMap<_, _>>();
    if map.len() != pairs.len() {
        return Err("constructed port correspondence differs");
    }
    Ok(map)
}

fn node_ports(
    workflow: &crate::engine::Workflow,
    node: &choregraph_nickel::construction::model::projection::ExpandedNode,
) -> Result<Vec<(u64, u64)>, &'static str> {
    let actual = workflow
        .graph
        .node(crate::engine::workflow::NodeId(node.id))
        .ok_or("constructed node correspondence missing")?;
    let ports = actual.input_ports.iter().chain(&actual.output_ports);
    if ports.clone().count() != node.ports.len() {
        return Err("constructed port correspondence differs");
    }
    ports.zip(&node.ports).map(|(actual, projected)| port_pair(actual, projected)).collect()
}

fn port_pair(
    actual: &crate::engine::workflow::Port,
    projected: &choregraph_nickel::construction::model::projection::ExpandedPort,
) -> Result<(u64, u64), &'static str> {
    let is_same_boundary = actual.label == projected.name && actual.type_tag == projected.value_type;
    is_same_boundary
        .then_some((actual.id.0, projected.id))
        .ok_or("constructed port correspondence differs")
}

// r[impl lattice.graph_construction.origins]
/// Diagnostic attribution only. Never infer a span from a generated numeric ID.
pub fn origin_for(
    workflow: &crate::engine::Workflow,
    element: &str,
) -> Option<choregraph_nickel::construction::model::projection::Origin> {
    let source = workflow.authoring.as_ref()?;
    let is_current_capture = source.state == super::super::AssociationState::CapturedModules
        && super::super::expanded_identity(workflow).ok()? == source.expanded_artifact;
    if !is_current_capture {
        return None;
    }
    super::admit_runtime(workflow).ok()??;
    let construction = source.construction.as_ref()?;
    let native = native_element(construction, element)?;
    let origin = construction.origins.iter().find(|origin| origin.element == native)?.clone();
    if let Some(location) = &origin.location {
        retained_span(source, location)?;
    }
    Some(origin)
}

/// Runtime port elements are renamed through the retained port map; nodes keep their ID.
fn native_element(construction: &super::super::ConstructionAssociation, element: &str) -> Option<String> {
    let Some(runtime) = element.strip_prefix("port/") else {
        return Some(element.to_owned());
    };
    let native = construction.port_map.get(&runtime.parse::<u64>().ok()?)?;
    Some(format!("port/{native}"))
}

/// The span lies on character boundaries of a retained captured module.
fn retained_span(
    source: &super::super::SourceAssociation,
    location: &choregraph_nickel::construction::model::program::SourceLocation,
) -> Option<()> {
    let bytes = source.observation.as_ref()?.modules.get(&location.logical_module)?;
    let text = std::str::from_utf8(bytes).ok()?;
    let is_ordered = location.start <= location.end;
    (is_ordered && text.is_char_boundary(location.start) && text.is_char_boundary(location.end)).then_some(())
}
