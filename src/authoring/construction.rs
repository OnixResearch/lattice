//! Concrete Lattice shell target for Choregraph construction v3.
//! The registry is selected by the host; Nickel's registry label is never authority.

pub mod lowering;
mod materialized;
pub(crate) mod store;

const TARGET: &str = "lattice-local-shell-v1";
const PROFILE: &str = "lattice-local-shell-registry-v1";
pub const RESOURCE: &str = "lattice://process/construction-shell-v1";
const ABILITY: &str = "process/execute";
/// Every node of the shell profile runs with this fixed timeout.
const SHELL_TIMEOUT_SECS: u64 = 60;

/// Construction bounds selected by Lattice; they restate the producer's reviewed profile.
const LIMITS: choregraph_nickel::construction::model::profile::Limits =
    choregraph_nickel::construction::model::profile::Limits {
        max_candidate_bytes: 1_048_576,
        max_nodes: choregraph_core::model::capacity::DEFAULT_MAX_NODES,
        max_ports: 16_384,
        max_edges: choregraph_core::model::capacity::DEFAULT_MAX_EDGES,
        max_effects: choregraph_core::model::capacity::DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_instances: choregraph_core::model::capacity::DEFAULT_MAX_NODES,
        max_depth: 64,
        max_boundaries: 16_384,
        max_identifier_bytes: choregraph_core::model::capacity::DEFAULT_MAX_IDENTIFIER_BYTES,
        max_origin_bytes: 1_048_576,
        max_parameter_bytes: 1_048_576,
        max_output_bytes: 8_388_608,
    };

#[cfg(target_os = "linux")]
pub fn selected_runtime() -> Result<crate::engine::workflow::ConstructionRuntimeBinding, &'static str> {
    let shell_path = store::reviewed_shell()?;
    let shell_artifact = store::immutable_executable(&shell_path)?;
    let runtime_artifact = store::running_artifact()?;
    Ok(crate::engine::workflow::ConstructionRuntimeBinding {
        runtime_artifact,
        shell_path: shell_path.to_str().ok_or("construction shell path is not UTF-8")?.to_owned(),
        shell_implementation: format!("{}:{shell_artifact}", executor_implementation()),
        shell_artifact,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn selected_runtime() -> Result<crate::engine::workflow::ConstructionRuntimeBinding, &'static str> {
    Err("construction profile requires Linux executable identity")
}

pub fn check_runtime(workflow: &crate::engine::Workflow) -> Result<(), &'static str> {
    if !workflow.construction_profile {
        return Ok(());
    }
    if workflow.construction_runtime.as_ref() != Some(&selected_runtime()?) {
        return Err("selected construction runtime or shell backend changed");
    }
    Ok(())
}

/// Value types of the shell target.
#[derive(Clone, Copy)]
enum HostType {
    Unit,
    Text,
    ExitCode,
}

impl HostType {
    const fn name(self) -> &'static str {
        match self {
            Self::Unit => "Unit",
            Self::Text => "string",
            Self::ExitCode => "exitcode",
        }
    }
}

fn boundary(name: &str, value_type: HostType) -> choregraph_nickel::construction::model::interface::Boundary {
    choregraph_nickel::construction::model::interface::Boundary {
        name: name.into(),
        value_type: value_type.name().into(),
    }
}

/// Both shell operations share one executor implementation.
fn executor_implementation() -> String {
    format!("lattice-shell-executor-v1-{}", env!("LATTICE_RUNTIME_IMPLEMENTATION_BLAKE3"))
}

fn signature(
    operation: &str,
    inputs: Vec<choregraph_nickel::construction::model::interface::Boundary>,
) -> choregraph_nickel::construction::model::host::OperationSignature {
    choregraph_nickel::construction::model::host::OperationSignature {
        operation: operation.into(),
        implementation: executor_implementation(),
        inputs,
        outputs: vec![
            boundary("stdout", HostType::Text),
            boundary("stderr", HostType::Text),
            boundary("exit", HostType::ExitCode),
        ],
        invocation: vec![
            boundary("dispatch-request", HostType::Text),
            boundary("resolved-payload", HostType::Text),
        ],
        configuration: vec![choregraph_nickel::construction::model::host::ConfigurationField {
            name: "command".into(),
            value_type: HostType::Text.name().into(),
            references: false,
            operators: vec![],
        }],
        effects: vec![choregraph_nickel::construction::model::host::EffectRule {
            from_state: "ready".into(),
            action: "start".into(),
            resource: RESOURCE.into(),
            ability: ABILITY.into(),
        }],
    }
}

// r[impl lattice.graph_construction.profile]
// r[impl lattice.graph_construction.instances.sharing]
pub fn registry() -> choregraph_nickel::construction::model::host::TargetRegistry {
    choregraph_nickel::construction::model::host::TargetRegistry {
        schema: choregraph_nickel::construction::model::profile::REGISTRY_SCHEMA.into(),
        target: TARGET.into(),
        profile: PROFILE.into(),
        implementation: format!("lattice-shell-host-v1-{}", env!("LATTICE_RUNTIME_IMPLEMENTATION_BLAKE3")),
        types: [HostType::Unit, HostType::Text, HostType::ExitCode]
            .into_iter()
            .map(|value_type| choregraph_nickel::construction::model::host::ValueType {
                name: value_type.name().into(),
                copyable: true,
                droppable: true,
            })
            .collect(),
        operations: vec![
            signature("shell-source", vec![]),
            signature("shell-stdin", vec![boundary("stdin", HostType::Text)]),
        ],
    }
}

/// Admit a v3 projection against a host registry under the selected bounds.
fn admit_projection(
    projection: &choregraph_nickel::construction::model::projection::LatticeV3,
    selected: &choregraph_nickel::construction::model::host::TargetRegistry,
) -> Result<choregraph_nickel::construction::admission::ProjectionOutput, &'static str> {
    let bytes = serde_json::to_vec(projection).map_err(|_| "projection encoding failed")?;
    choregraph_nickel::construction::admission::lower_projection_json(&bytes, selected, LIMITS)
        .map_err(|_| "v3 projection admission rejected workflow")
}

// r[impl lattice.graph_construction.rejection]
/// Native materialized admission for explicit forks and expanded interchange.
/// This creates no source or origin claim and cannot reuse the compiled receipt.
pub fn admit_materialized(
    workflow: &crate::engine::Workflow,
) -> Result<choregraph_nickel::construction::admission::ProjectionOutput, &'static str> {
    if !workflow.construction_profile {
        return Err("workflow does not select construction profile");
    }
    check_runtime(workflow)?;
    let projection = materialized::projection(workflow, registry())?;
    admit_projection(&projection, &registry()).map_err(|_| "materialized v3 admission rejected workflow")
}

/// Construction programs are compiled natively by the selected shell target.
pub(super) const fn is_native_kind(kind: super::ProgramKind) -> bool {
    matches!(kind, super::ProgramKind::Construction | super::ProgramKind::ParameterizedConstruction)
}

/// Retained construction lineage cannot be reclassified as a legacy flat workflow.
// r[impl lattice.graph_construction.compatibility]
// r[impl lattice.graph_construction.compatibility.legacy]
pub(crate) fn has_retained_construction_lineage(workflow: &crate::engine::Workflow) -> bool {
    workflow
        .authoring
        .as_ref()
        .is_some_and(|source| source.construction.is_some() || is_native_kind(source.program.kind))
}

// r[impl lattice.graph_construction.drift]
// r[impl lattice.graph_construction.drift.registry]
// r[impl lattice.graph_construction.closure.reopen]
/// Reconstruct from validated observed bytes and the current host implementation.
/// An old translation receipt, graph hash or registry label alone never admits execution.
pub fn admit_runtime(
    workflow: &crate::engine::Workflow,
) -> Result<Option<choregraph_nickel::construction::admission::ProjectionOutput>, &'static str> {
    if !workflow.construction_profile && has_retained_construction_lineage(workflow) {
        return Err("retained construction lineage requires construction profile");
    }
    check_runtime(workflow)?;
    let Some(source) = &workflow.authoring else {
        return admit_selected_profile(workflow);
    };
    admit_source(workflow, source)
}

fn admit_source(
    workflow: &crate::engine::Workflow,
    source: &super::SourceAssociation,
) -> Result<Option<choregraph_nickel::construction::admission::ProjectionOutput>, &'static str> {
    source.validate()?;
    if source.state == super::AssociationState::MaterializedFork {
        return admit_selected_profile(workflow);
    }
    if source.state == super::AssociationState::CapturedModules {
        verify_captured_association(workflow, source)?;
    }
    let Some(retained) = &source.construction else {
        verify_flat_lineage(source)?;
        return admit_selected_profile(workflow);
    };
    readmit_retained(workflow, source, retained).map(Some)
}

/// Without retained construction facts, only the selected profile decides admission.
fn admit_selected_profile(
    workflow: &crate::engine::Workflow,
) -> Result<Option<choregraph_nickel::construction::admission::ProjectionOutput>, &'static str> {
    if workflow.construction_profile {
        admit_materialized(workflow).map(Some)
    } else {
        Ok(None)
    }
}

fn verify_captured_association(
    workflow: &crate::engine::Workflow,
    source: &super::SourceAssociation,
) -> Result<(), &'static str> {
    if source.compiler != super::CompilerIdentity::current()?
        || super::expanded_identity(workflow)? != source.expanded_artifact
    {
        return Err("captured association no longer matches admitted workflow");
    }
    let observation = source.observation.as_ref().ok_or("construction evaluator observation missing")?;
    super::shell::verify_observation(&source.program, observation)
}

/// A flat captured workflow must still decode to the retained expanded artifact.
fn verify_flat_lineage(source: &super::SourceAssociation) -> Result<(), &'static str> {
    if is_native_kind(source.program.kind) {
        return Err("construction projection is missing");
    }
    if source.state != super::AssociationState::CapturedModules {
        return Ok(());
    }
    let observation = source.observation.as_ref().ok_or("captured evaluator observation missing")?;
    let value = serde_json::from_str(&observation.result.json).map_err(|_| "captured workflow output is invalid")?;
    let rebuilt = crate::workflow_format::from_nickel_value(value, &source.program.files)
        .map_err(|_| "captured workflow no longer admits")?;
    if super::expanded_identity(&rebuilt)? != source.expanded_artifact {
        return Err("captured workflow differs from observed output");
    }
    Ok(())
}

const CHANGED: &str = "construction binding or materialized workflow changed";

/// Recompile the observed evaluation and require every retained construction fact.
fn readmit_retained(
    workflow: &crate::engine::Workflow,
    source: &super::SourceAssociation,
    retained: &super::ConstructionAssociation,
) -> Result<choregraph_nickel::construction::admission::ProjectionOutput, &'static str> {
    if source.state != super::AssociationState::CapturedModules {
        return Err("construction association lacks captured evaluation");
    }
    let observation = source.observation.as_ref().ok_or("construction evaluator observation missing")?;
    let (rebuilt, projection, binding, origins) = lowering::compile(observation)?;
    if rebuilt.meta.name != workflow.meta.name || super::expanded_identity(&rebuilt)? != source.expanded_artifact {
        return Err(CHANGED);
    }
    let is_retained = serde_json::to_vec(&projection).map_err(|_| "projection encoding failed")?
        == retained.projection_json
        && serde_json::to_vec(&binding).map_err(|_| "binding encoding failed")? == retained.binding_json
        && origins == retained.origins;
    if !is_retained || lowering::port_map(&rebuilt, &projection)? != retained.port_map {
        return Err(CHANGED);
    }
    choregraph_nickel::construction::admission::lower_projection_json(&retained.projection_json, &registry(), LIMITS)
        .map_err(|_| "current v3 admission rejected workflow")
}

#[cfg(test)]
mod tests {
    #[test]
    fn selected_limits_restate_reviewed_producer_profile() {
        assert_eq!(super::LIMITS, choregraph_nickel::construction::model::profile::Limits::default());
    }
}
