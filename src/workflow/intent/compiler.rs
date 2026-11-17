// r[impl lattice.workflow_intent.planning]
// r[impl lattice.workflow_intent.bindings]
pub fn compile(mut request: super::CompileRequest) -> Result<super::CompiledPlan, super::Diagnostic> {
    super::validation::normalize(&mut request)?;
    let selected = request
        .intent
        .selected_alternative
        .as_ref()
        .and_then(|selected| request.intent.alternatives.iter().find(|alternative| &alternative.id == selected));
    // A selected implementation may change a binding, never erase required nodes.
    let nodes = request
        .intent
        .obligations
        .iter()
        .map(|obligation| {
            let implementation = selected
                .filter(|alternative| alternative.obligations.contains(&obligation.id))
                .map_or(&request.facts.runtime_profile_identity, |alternative| &alternative.implementation_identity);
            let operations = vec![format!(
                "{}/{}/{}",
                obligation.operation_class, obligation.action_identity, implementation
            )];
            choregraph_core::NodeSpec::new(&obligation.id, operations)
        })
        .collect::<Vec<_>>();
    let edges = request
        .intent
        .obligations
        .iter()
        .flat_map(|obligation| {
            obligation.prerequisites.iter().map(|source| choregraph_core::EdgeSpec::new(source, &obligation.id))
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(nodes.len(), request.intent.obligations.len());
    debug_assert!(edges.len() <= super::MAX_DEPENDENCIES);
    let node_ids = nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
    let limits = choregraph_core::AdmissionLimits {
        max_nodes: super::MAX_OBLIGATIONS,
        max_edges: super::MAX_DEPENDENCIES,
        max_operations: choregraph_core::DEFAULT_MAX_OPERATIONS,
        max_effect_declarations: choregraph_core::DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_identifier_bytes: choregraph_core::DEFAULT_MAX_IDENTIFIER_BYTES,
        max_reference_bytes: choregraph_core::DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: choregraph_core::DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: choregraph_core::DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition: choregraph_core::DEFAULT_MAX_INTENTS_PER_TRANSITION,
    };
    let machine = choregraph_core::AdmittedMachine::admit_v2(choregraph_core::MachineSpecV2::new(nodes, edges), limits)
        .map_err(|_| super::Diagnostic::new(super::DiagnosticCode::GraphAdmission, "dependency_dag"))?;
    let artifact = choregraph_kamacite::build_concrete_machine_artifact(&machine)
        .map_err(|_| super::Diagnostic::new(super::DiagnosticCode::Encoding, "dependency_dag"))?;
    let identity = plan_identity(&request, &artifact.identity)?;
    Ok(super::CompiledPlan {
        schema: super::PLAN_SCHEMA,
        compiler_revision: super::COMPILER_REVISION,
        identity,
        graph_identity: artifact.identity,
        graph: artifact.descriptor,
        intent: request.intent,
        facts: request.facts,
        nodes: node_ids,
        disposition: "planning_only_requires_separate_execution_admission",
    })
}

fn plan_identity(
    request: &super::CompileRequest,
    graph: &choregraph_kamacite::ArtifactIdentity,
) -> Result<String, super::Diagnostic> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice.workflow-intent-plan.v1\0");
    serde_json::to_writer(&mut hasher, &(super::COMPILER_REVISION, super::PLAN_SCHEMA, request, graph))
        .map_err(|_| super::Diagnostic::new(super::DiagnosticCode::Encoding, "plan"))?;
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}
