//! `lattice.workflow-agent-revision-slots.v1`: the desired, last usable,
//! selected and pinned workflow-agent revisions of one definition, with the
//! workflow and worker revisions of each kept apart, for Kiln #84.
//!
//! Only identities are projected: no prompt, path, endpoint, credential,
//! guidance text or restricted context. A lifecycle of plain agent bundles
//! has no workflow slots; a workflow-agent lifecycle whose referenced
//! revision is missing or not a workflow-agent bundle is refused.

// r[impl lattice.agent_revisions.workflow.projection]

type Lifecycle = crate::agent_revision::state::Lifecycle;
type Refusal = crate::agent_revision::Refusal;

pub const SLOTS_SCHEMA: &str = "lattice.workflow-agent-revision-slots.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    pub schema: String,
    pub definition: String,
    pub name: String,
    pub desired: Option<Entry>,
    pub last_usable: Option<Entry>,
    pub selected_for_new_sessions: Option<Selected>,
    /// Sorted by session.
    pub pinned: Vec<Pinned>,
}

/// One workflow-agent revision and the two revisions it binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub revision: String,
    pub workflow: Compiled,
    pub worker: super::Worker,
}

/// The workflow revision: compiler, plan, graph and Cairn identities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compiled {
    pub compiler_revision: String,
    pub compiler_artifact_identity: String,
    pub plan_identity: String,
    pub graph_identity: String,
    pub selection_identity: String,
    pub obligations_identity: String,
    pub policy_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selected {
    pub bundle: Entry,
    pub rollout_generation: u64,
    pub kind: crate::agent_revision::state::SelectionKind,
    pub qualification_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pinned {
    pub session: String,
    pub bundle: Entry,
    pub status: crate::agent_revision::state::SessionStatus,
    /// The session may perform no further effect under its pin.
    pub is_blocked: bool,
}

fn entry(lifecycle: &Lifecycle, revision: &str) -> Result<Entry, Refusal> {
    let bundle = &crate::agent_revision::ops::candidate(lifecycle, revision)?.bundle;
    let binding = super::binding(bundle)?;
    let (plan, cohort) = (&binding.plan, &binding.plan.cohort);
    debug_assert_eq!(bundle.revision, revision);
    debug_assert_eq!(bundle.definition, lifecycle.definition);
    Ok(Entry {
        revision: revision.to_owned(),
        workflow: Compiled {
            compiler_revision: plan.compiler_revision.clone(),
            compiler_artifact_identity: plan.compiler_artifact_identity.clone(),
            plan_identity: plan.identity.clone(),
            graph_identity: plan.graph_identity.clone(),
            selection_identity: cohort.selection_identity.clone(),
            obligations_identity: cohort.obligations_identity.clone(),
            policy_identity: cohort.policy_identity.clone(),
        },
        worker: binding.worker.clone(),
    })
}

fn optional(lifecycle: &Lifecycle, revision: Option<&str>) -> Result<Option<Entry>, Refusal> {
    revision.map(|revision| entry(lifecycle, revision)).transpose()
}

/// Project a workflow-agent lifecycle; `None` when it holds no
/// workflow-agent bundle.
pub fn slots(lifecycle: &Lifecycle) -> Result<Option<Slots>, Refusal> {
    let Some(name) = lifecycle.candidates.values().find_map(|candidate| candidate.bundle.workflow.as_ref()) else {
        return Ok(None);
    };
    let name = name.name.clone();
    let selected = lifecycle
        .selected
        .as_ref()
        .map(|selection| {
            Ok(Selected {
                bundle: entry(lifecycle, &selection.revision)?,
                rollout_generation: selection.rollout_generation,
                kind: selection.kind,
                qualification_ref: selection.qualification_ref.clone(),
            })
        })
        .transpose()?;
    let pinned = lifecycle
        .sessions
        .iter()
        .map(|(session, pin)| {
            Ok(Pinned {
                session: session.clone(),
                bundle: entry(lifecycle, &pin.revision)?,
                status: pin.status,
                is_blocked: crate::agent_revision::ops::sessions::effect_permitted(lifecycle, session).is_err(),
            })
        })
        .collect::<Result<Vec<_>, Refusal>>()?;
    debug_assert_eq!(pinned.len(), lifecycle.sessions.len());
    debug_assert!(crate::agent_revision::is_token(&name));
    Ok(Some(Slots {
        schema: SLOTS_SCHEMA.to_owned(),
        definition: lifecycle.definition.clone(),
        name,
        desired: optional(lifecycle, lifecycle.desired.as_ref().map(|desired| desired.revision.as_str()))?,
        last_usable: optional(lifecycle, lifecycle.last_usable.as_deref())?,
        selected_for_new_sessions: selected,
        pinned,
    }))
}
