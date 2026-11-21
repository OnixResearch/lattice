//! `lattice.workflow-agent-bundle.v1`: one immutable bundle that binds a
//! compiled workflow-intent plan to one agent profile revision (Lattice #25,
//! "Pin compiled workflow and agent revisions to Cairn campaigns").
//!
//! The bundle is an ordinary lifecycle [`Bundle`] whose `workflow` field
//! holds the [`Binding`]. Its definition is the
//! `lattice-workflow-agent-definition-v1` identity over the binding's name and
//! the worker's definition, so it never collides with a worker definition.
//! Its revision is the bare-hex `lattice-workflow-agent-revision-v1` digest
//! of the canonical binding: the plan (compiler, plan, graph and Cairn
//! cohort identities as the intent carries them) and the worker bundle
//! identity. Any change to either is a new revision, managed by the existing
//! desire, prepare, promote, admit and rollback transitions. The worker's own
//! fields are copied into the bundle so the worker bundle can be rebuilt and
//! checked against its recorded identity; a bundle that disagrees with its
//! binding does not validate.

// r[impl lattice.agent_revisions.workflow]
// r[impl lattice.agent_revisions.workflow.scenario.identity]
// r[impl lattice.agent_revisions.workflow.scenario.lifecycle]
// r[impl lattice.agent_revisions.workflow.scenario.evaluation]

pub mod adaptation;
pub mod publication;

type Bundle = crate::agent_revision::facts::Bundle;
type Refusal = crate::agent_revision::Refusal;

pub const BUNDLE_SCHEMA: &str = "lattice.workflow-agent-bundle.v1";
/// Obligations one plan binding carries, the workflow-intent compiler's bound.
pub const MAX_OBLIGATIONS: u32 = 128;
const PLAN_PREFIX: &str = "b3:";

/// The workflow and worker one bundle pairs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    /// The operator's name for the pairing; with the worker definition it
    /// names the bundle's definition.
    pub name: String,
    pub plan: Plan,
    pub worker: Worker,
}

/// Identities of one compiled `lattice.workflow-intent-plan.v1` plan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: String,
    pub compiler_revision: String,
    /// The measured compiler executable the plan was compiled by.
    pub compiler_artifact_identity: String,
    /// `b3:` plan identity.
    pub identity: String,
    /// `blake3:` Choregraph machine identity of the selected graph.
    pub graph_identity: String,
    /// `b3:` runtime profile the plan binds by default.
    pub runtime_profile_identity: String,
    pub cohort: Cohort,
    /// Sorted by identifier, distinct.
    pub obligations: Vec<Obligation>,
}

/// The host-normalized Cairn cohort the intent carries, all `b3:`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cohort {
    pub producer_identity: String,
    pub selection_identity: String,
    pub obligations_identity: String,
    pub profile_identity: String,
    pub context_identity: String,
    pub source_identity: String,
    pub policy_identity: String,
    pub dependency_identity: String,
}

/// One Cairn obligation's identities, all `b3:`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    pub id: String,
    pub obligation_identity: String,
    /// The guidance (prompt) identity of the obligation.
    pub guidance_identity: String,
    pub remaining_condition_identity: String,
}

/// The worker bundle the pairing binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worker {
    /// `blake3:` worker definition identity.
    pub definition: String,
    pub definition_name: String,
    /// Bare 64-hex profile revision.
    pub revision: String,
    /// `lattice-agent-bundle-v1` identity of the worker bundle.
    pub bundle_identity: String,
}

fn is_plan_identity(value: &str) -> bool {
    value.strip_prefix(PLAN_PREFIX).is_some_and(crate::agent_revision::is_bare_digest)
}

fn is_cohort(cohort: &Cohort) -> bool {
    let identities = [
        &cohort.producer_identity,
        &cohort.selection_identity,
        &cohort.obligations_identity,
        &cohort.profile_identity,
        &cohort.context_identity,
        &cohort.source_identity,
        &cohort.policy_identity,
        &cohort.dependency_identity,
    ];
    identities.iter().all(|value| is_plan_identity(value))
}

fn is_obligation(obligation: &Obligation) -> bool {
    let identities = [
        &obligation.obligation_identity,
        &obligation.guidance_identity,
        &obligation.remaining_condition_identity,
    ];
    crate::agent_revision::is_token(&obligation.id) && identities.iter().all(|value| is_plan_identity(value))
}

fn is_plan(plan: &Plan) -> bool {
    let tokens = [&plan.schema, &plan.compiler_revision, &plan.compiler_artifact_identity];
    let is_sorted = plan.obligations.windows(2).all(|pair| pair[0].id < pair[1].id);
    let checks = [
        tokens.iter().all(|value| crate::agent_revision::is_token(value)),
        is_plan_identity(&plan.identity),
        crate::agent_revision::is_reference(&plan.graph_identity),
        is_plan_identity(&plan.runtime_profile_identity),
        is_cohort(&plan.cohort),
        !plan.obligations.is_empty(),
        crate::agent_revision::is_within(plan.obligations.len(), MAX_OBLIGATIONS),
        is_sorted,
        plan.obligations.iter().all(is_obligation),
    ];
    checks.iter().all(|check| *check)
}

fn is_worker(worker: &Worker) -> bool {
    let checks = [
        crate::agent_revision::is_reference(&worker.definition),
        crate::agent_revision::is_token(&worker.definition_name),
        crate::agent_revision::is_bare_digest(&worker.revision),
        crate::agent_revision::is_reference(&worker.bundle_identity),
    ];
    checks.iter().all(|check| *check)
}

/// The worker bundle `bundle` was composed from, rebuilt from its binding.
#[must_use]
pub fn worker_bundle(bundle: &Bundle, binding: &Binding) -> Bundle {
    let worker = Bundle {
        schema: crate::agent_revision::facts::BUNDLE_SCHEMA.to_owned(),
        definition: binding.worker.definition.clone(),
        definition_name: binding.worker.definition_name.clone(),
        revision: binding.worker.revision.clone(),
        workflow: None,
        ..bundle.clone()
    };
    debug_assert!(worker.workflow.is_none());
    debug_assert_eq!(worker.revision, binding.worker.revision);
    worker
}

fn identities(binding: &Binding) -> Result<(String, String), Refusal> {
    let canonical = serde_json::to_vec(binding).map_err(|_| Refusal::Malformed)?;
    let definition = crate::agent_revision::identity::pairing(binding).map_err(|_| Refusal::Malformed)?;
    let revision = crate::agent_revision::identity::composite(&canonical).map_err(|_| Refusal::Malformed)?;
    debug_assert!(crate::agent_revision::is_reference(&definition));
    debug_assert!(crate::agent_revision::is_bare_digest(&revision));
    Ok((definition, revision))
}

/// Check a workflow-agent bundle against its binding: canonical identities,
/// the definition and revision the binding frames, and the worker bundle
/// identity it records.
pub fn validate(bundle: &Bundle, binding: &Binding) -> Result<(), Refusal> {
    let shape = [
        bundle.schema == BUNDLE_SCHEMA,
        crate::agent_revision::is_token(&binding.name),
        is_plan(&binding.plan),
        is_worker(&binding.worker),
    ];
    if !shape.iter().all(|check| *check) {
        return Err(Refusal::Malformed);
    }
    let (definition, revision) = identities(binding)?;
    let worker = worker_bundle(bundle, binding);
    let worker_identity = crate::agent_revision::facts::identity(&worker)?;
    let framing = [
        bundle.definition == definition,
        bundle.definition_name == binding.name,
        bundle.revision == revision,
        worker_identity == binding.worker.bundle_identity,
    ];
    if framing.iter().all(|check| *check) {
        debug_assert!(crate::agent_revision::is_bare_digest(&bundle.revision));
        debug_assert_eq!(bundle.definition_name, binding.name);
        Ok(())
    } else {
        Err(Refusal::Malformed)
    }
}

/// Compose the workflow-agent bundle that pairs `plan` with the plain worker
/// bundle `worker` under `name`.
pub fn compose(name: &str, plan: Plan, worker: &Bundle) -> Result<Bundle, Refusal> {
    if worker.workflow.is_some() {
        return Err(Refusal::Malformed);
    }
    let bundle_identity = crate::agent_revision::facts::identity(worker)?;
    let binding = Binding {
        name: name.to_owned(),
        plan,
        worker: Worker {
            definition: worker.definition.clone(),
            definition_name: worker.definition_name.clone(),
            revision: worker.revision.clone(),
            bundle_identity,
        },
    };
    let (definition, revision) = identities(&binding)?;
    let composed = Bundle {
        schema: BUNDLE_SCHEMA.to_owned(),
        definition,
        definition_name: name.to_owned(),
        revision,
        workflow: Some(binding),
        ..worker.clone()
    };
    crate::agent_revision::facts::validate(&composed)?;
    debug_assert!(composed.workflow.is_some());
    debug_assert_ne!(composed.definition, worker.definition);
    Ok(composed)
}

/// The binding of a validated workflow-agent bundle.
pub fn binding(bundle: &Bundle) -> Result<&Binding, Refusal> {
    let binding = bundle.workflow.as_ref().ok_or(Refusal::Malformed)?;
    crate::agent_revision::facts::validate(bundle)?;
    Ok(binding)
}

// r[impl lattice.agent_revisions.workflow.attempt]
/// The seam an attempt request will use once it names a workflow-agent
/// bundle: a session of `bundle` may run only the exact worker revision the
/// bundle binds. Admission then pins the bundle's own definition and
/// revision; the worker's profile revision alone never admits it.
pub fn check_attempt(bundle: &Bundle, profile_revision: &str) -> Result<(), Refusal> {
    let binding = binding(bundle)?;
    if !crate::agent_revision::is_bare_digest(profile_revision) {
        return Err(Refusal::Malformed);
    }
    if binding.worker.revision != profile_revision {
        return Err(Refusal::RevisionNotSelected);
    }
    debug_assert!(crate::agent_revision::is_bare_digest(&bundle.revision));
    Ok(())
}
