//! Compose one inert workflow-agent bundle from a compiled workflow-intent
//! plan and an exported implementation-attempt request.
//!
//! The plan is compiled in process from a typed intent that names this exact
//! compiler executable; a different executable is refused, never rebound.
//! Composition admits, prepares and writes nothing. The pure pairing,
//! validation and identities are `lattice_core::agent_revision::workflow`.

// r[impl lattice.agent_revisions.workflow]
// r[impl lattice.agent_revisions.workflow.inspect]

type Bundle = lattice_core::agent_revision::facts::Bundle;
type Plan = lattice_core::agent_revision::workflow::Plan;

/// A composed workflow-agent bundle and the worker bundle it binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Composed {
    pub definition: String,
    pub revision: String,
    pub bundle: Bundle,
    pub bundle_identity: String,
    pub worker: super::bundle::Resolved,
}

/// Compile a typed workflow intent bound to this compiler executable.
pub fn compile(intent: &std::path::Path) -> Result<crate::workflow::intent::CompiledPlan, String> {
    let request = crate::workflow::intent::cli::load(intent)?;
    crate::workflow::intent::compile(request).map_err(|error| error.to_string())
}

fn graph_identity(plan: &crate::workflow::intent::CompiledPlan) -> Result<String, String> {
    let graph = plan.graph_identity();
    let is_machine = graph.domain == choregraph_kamacite::identity::ArtifactDomain::Machine;
    if !is_machine || graph.algorithm != "blake3" {
        return Err("compiled graph identity is not a BLAKE3 machine identity".to_owned());
    }
    Ok(format!("blake3:{}", graph.hex))
}

/// The identities of one compiled plan, as the intent carries them.
pub fn plan(compiled: &crate::workflow::intent::CompiledPlan) -> Result<Plan, String> {
    let cohort = &compiled.intent().cohort;
    let obligations = compiled
        .intent()
        .obligations
        .iter()
        .map(|obligation| lattice_core::agent_revision::workflow::Obligation {
            id: obligation.id.clone(),
            obligation_identity: obligation.obligation_identity.clone(),
            guidance_identity: obligation.guidance_identity.clone(),
            remaining_condition_identity: obligation.remaining_condition_identity.clone(),
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(obligations.len(), compiled.nodes().len());
    debug_assert!(obligations.windows(2).all(|pair| pair[0].id < pair[1].id));
    Ok(Plan {
        schema: compiled.schema().to_owned(),
        compiler_revision: compiled.compiler_revision().to_owned(),
        compiler_artifact_identity: compiled.facts().compiler_artifact_identity.clone(),
        identity: compiled.identity().to_owned(),
        graph_identity: graph_identity(compiled)?,
        runtime_profile_identity: compiled.facts().runtime_profile_identity.clone(),
        cohort: lattice_core::agent_revision::workflow::Cohort {
            producer_identity: cohort.producer_identity.clone(),
            selection_identity: cohort.selection_identity.clone(),
            obligations_identity: cohort.obligations_identity.clone(),
            profile_identity: cohort.profile_identity.clone(),
            context_identity: cohort.context_identity.clone(),
            source_identity: cohort.source_identity.clone(),
            policy_identity: cohort.policy_identity.clone(),
            dependency_identity: cohort.dependency_identity.clone(),
        },
        obligations,
    })
}

/// Pair `compiled` with the resolved worker under `name`.
pub fn compose(
    name: &str,
    compiled: &crate::workflow::intent::CompiledPlan,
    worker: super::bundle::Resolved,
) -> Result<Composed, String> {
    let bundle = lattice_core::agent_revision::workflow::compose(name, plan(compiled)?, &worker.bundle)
        .map_err(|refusal| format!("workflow-agent bundle is unsupported: {refusal:?}"))?;
    let bundle_identity = lattice_core::agent_revision::facts::identity(&bundle)
        .map_err(|refusal| format!("workflow-agent bundle is unsupported: {refusal:?}"))?;
    debug_assert_ne!(bundle.revision, worker.revision);
    debug_assert!(bundle.workflow.is_some());
    Ok(Composed {
        definition: bundle.definition.clone(),
        revision: bundle.revision.clone(),
        bundle,
        bundle_identity,
        worker,
    })
}

/// Resolve the request and compile the intent, then compose them.
pub fn resolve(args: &super::WorkflowArgs) -> Result<Composed, String> {
    let worker = super::bundle::resolve(&super::bundle::load(&args.request)?)?;
    compose(&args.name, &compile(&args.intent)?, worker)
}

pub(super) fn resolve_command(args: &super::WorkflowArgs) -> Result<serde_json::Value, String> {
    let composed = resolve(args)?;
    let definition = composed.definition.clone();
    let header = super::commands::Header {
        command: "resolve-workflow",
        definition: &definition,
    };
    Ok(super::commands::document(header, Ok(composed)))
}

/// Resolve and compose before opening the store, so a refused intent or
/// request creates no store.
pub(super) fn desire_command(args: &super::WorkflowArgs, store: &std::path::Path) -> Result<serde_json::Value, String> {
    let composed = resolve(args)?;
    let store = super::commands::open(store)?;
    let definition = composed.definition.as_str();
    let result = super::commands::transact(&store, definition, super::commands::Seed::Create, |lifecycle| {
        lattice_core::agent_revision::ops::prepare::desire(lifecycle, &composed.bundle)
    })?;
    let header = super::commands::Header {
        command: "desire-workflow",
        definition,
    };
    Ok(super::commands::document(header, result))
}

/// The Kiln #84 workflow-agent slots of a stored lifecycle, or `None` for a
/// lifecycle of plain agent bundles.
pub(super) fn slots(
    lifecycle: &lattice_core::agent_revision::state::Lifecycle,
) -> Result<Option<serde_json::Value>, String> {
    let slots = lattice_core::agent_revision::workflow::publication::slots(lifecycle)
        .map_err(|refusal| format!("workflow-agent slots are inconsistent: {refusal:?}"))?;
    slots.map(|slots| serde_json::to_value(slots).map_err(|error| error.to_string())).transpose()
}
