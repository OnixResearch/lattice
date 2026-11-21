//! `lattice.workflow-agent-proposal.v1`: a proposed workflow adaptation,
//! prompt, tool, requirement or policy change as a new bundle identity with
//! an explicit applicability slot.
//!
//! A proposal is data. It names the active and proposed revisions and what
//! differs between them, and records the applicability decision as
//! `unknown`: Cairn (#14) and Kiln (#83) own that decision, and the type
//! admits no other value. Nothing here changes a lifecycle, so an active
//! selection or session pin is never mutated; a proposal reaches new
//! sessions only through the ordinary desire, prepare, qualify and promote
//! transitions under its own revision.

// r[impl lattice.agent_revisions.workflow.adaptation]

type Bundle = crate::agent_revision::facts::Bundle;
type Refusal = crate::agent_revision::Refusal;

pub const PROPOSAL_SCHEMA: &str = "lattice.workflow-agent-proposal.v1";
/// Owners of the applicability decision.
pub const APPLICABILITY_OWNERS: [&str; 2] = ["cairn#14", "kiln#83"];

/// What differs between an active and a proposed bundle.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    /// Plan schema, compiler revision or compiler artifact.
    Compiler,
    Plan,
    Graph,
    /// The Cairn selection identity.
    Selection,
    /// Obligations added, removed, or with another obligation or remaining
    /// condition, or another obligations identity.
    Requirements,
    /// An obligation's guidance identity.
    Prompt,
    /// Producer, profile, context, source or dependency identity.
    Context,
    Policy,
    RuntimeProfile,
    /// The worker's profile revision.
    WorkerRevision,
    Executable,
    Tools,
    ModelRoute,
    Compute,
    TaskFamilies,
    /// Adapter or protocol revision.
    Protocol,
}

/// Lattice records no applicability decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Unknown,
}

/// The applicability slot of one proposal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub decision: Decision,
    pub owners: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case", deny_unknown_fields)]
pub enum Proposal {
    /// The proposed bundle is the active one; there is nothing to decide.
    Unchanged { revision: String },
    Adaptation {
        schema: String,
        definition: String,
        active_revision: String,
        proposed_revision: String,
        /// Sorted, distinct, never empty.
        changes: Vec<Change>,
        applicability: Applicability,
    },
}

fn plan_changes(active: &super::Plan, proposed: &super::Plan) -> Vec<(bool, Change)> {
    debug_assert!(super::is_plan(active));
    debug_assert!(super::is_plan(proposed));
    let (before, after) = (&active.cohort, &proposed.cohort);
    let is_compiler = (&active.schema, &active.compiler_revision, &active.compiler_artifact_identity)
        != (&proposed.schema, &proposed.compiler_revision, &proposed.compiler_artifact_identity);
    let context = |cohort: &super::Cohort| {
        [
            cohort.producer_identity.clone(),
            cohort.profile_identity.clone(),
            cohort.context_identity.clone(),
            cohort.source_identity.clone(),
            cohort.dependency_identity.clone(),
        ]
    };
    let requirement = |obligation: &super::Obligation| {
        (
            obligation.id.clone(),
            obligation.obligation_identity.clone(),
            obligation.remaining_condition_identity.clone(),
        )
    };
    let guidance = |plan: &super::Plan| {
        plan.obligations
            .iter()
            .map(|obligation| (obligation.id.clone(), obligation.guidance_identity.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let (guided, proposed_guidance) = (guidance(active), guidance(proposed));
    let is_prompt = guided.iter().any(|(id, value)| proposed_guidance.get(id).is_some_and(|other| other != value));
    let is_requirement = before.obligations_identity != after.obligations_identity
        || !active.obligations.iter().map(requirement).eq(proposed.obligations.iter().map(requirement));
    vec![
        (is_compiler, Change::Compiler),
        (active.identity != proposed.identity, Change::Plan),
        (active.graph_identity != proposed.graph_identity, Change::Graph),
        (before.selection_identity != after.selection_identity, Change::Selection),
        (is_requirement, Change::Requirements),
        (is_prompt, Change::Prompt),
        (context(before) != context(after), Change::Context),
        (before.policy_identity != after.policy_identity, Change::Policy),
        (active.runtime_profile_identity != proposed.runtime_profile_identity, Change::RuntimeProfile),
    ]
}

fn worker_changes(active: &Bundle, proposed: &Bundle) -> Vec<(bool, Change)> {
    let protocol = |bundle: &Bundle| (bundle.adapter.clone(), bundle.protocol_revision.clone());
    let worker = |bundle: &Bundle| bundle.workflow.as_ref().map(|binding| binding.worker.revision.clone());
    vec![
        (worker(active) != worker(proposed), Change::WorkerRevision),
        (active.executable_identity != proposed.executable_identity, Change::Executable),
        (active.tools != proposed.tools, Change::Tools),
        (active.routes != proposed.routes, Change::ModelRoute),
        (active.compute != proposed.compute, Change::Compute),
        (active.task_families != proposed.task_families, Change::TaskFamilies),
        (protocol(active) != protocol(proposed), Change::Protocol),
    ]
}

/// Compare a proposed workflow-agent bundle with the active one of the same
/// definition.
pub fn propose(active: &Bundle, proposed: &Bundle) -> Result<Proposal, Refusal> {
    let (before, after) = (super::binding(active)?, super::binding(proposed)?);
    if active.definition != proposed.definition {
        return Err(Refusal::DefinitionMismatch);
    }
    if active.revision == proposed.revision {
        return Ok(Proposal::Unchanged {
            revision: active.revision.clone(),
        });
    }
    let mut flagged = plan_changes(&before.plan, &after.plan);
    flagged.extend(worker_changes(active, proposed));
    let changes = flagged
        .into_iter()
        .filter_map(|(is_changed, change)| is_changed.then_some(change))
        .collect::<std::collections::BTreeSet<_>>();
    if changes.is_empty() {
        return Err(Refusal::Malformed);
    }
    debug_assert_eq!(before.name, after.name);
    debug_assert_ne!(active.revision, proposed.revision);
    Ok(Proposal::Adaptation {
        schema: PROPOSAL_SCHEMA.to_owned(),
        definition: active.definition.clone(),
        active_revision: active.revision.clone(),
        proposed_revision: proposed.revision.clone(),
        changes: changes.into_iter().collect(),
        applicability: Applicability {
            decision: Decision::Unknown,
            owners: APPLICABILITY_OWNERS.iter().map(|owner| (*owner).to_owned()).collect(),
        },
    })
}
