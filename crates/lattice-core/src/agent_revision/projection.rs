//! `lattice.agent-revision-slots.v1`: the bounded desired, prepared,
//! qualified, selected and pinned facts of one definition, for a later
//! inventory schema and Kiln #60. Bundles, artifacts, holds and receipts are
//! not projected; nothing here carries a path, endpoint, prompt or secret.

// r[impl lattice.agent_revisions.projection]

pub const SLOTS_SCHEMA: &str = "lattice.agent-revision-slots.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slots {
    pub schema: String,
    pub definition: String,
    pub desired: Option<crate::agent_revision::state::Desired>,
    /// Sorted revisions prepared for their desired generation.
    pub prepared: Vec<String>,
    pub qualified: Vec<Qualified>,
    pub last_usable: Option<String>,
    pub selected_for_new_sessions: Option<Selected>,
    /// Sorted by session.
    pub pinned: Vec<Pinned>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qualified {
    pub revision: String,
    pub task_family: String,
    pub qualification_ref: String,
    pub evidence_class: crate::agent_revision::ports::EvidenceClass,
    /// `fixture` or `live`, as the producer labelled the attachment.
    pub provenance: crate::agent_revision::ports::Provenance,
    pub is_revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selected {
    pub revision: String,
    pub rollout_generation: u64,
    pub kind: crate::agent_revision::state::SelectionKind,
    pub qualification_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pinned {
    pub session: String,
    pub revision: String,
    pub status: crate::agent_revision::state::SessionStatus,
    /// The session may perform no further effect under its pin.
    pub is_blocked: bool,
}

/// The pin an embedder recorded with one session at admission. Closing a
/// session without a held checkpoint forgets its pin, so the embedder's
/// record is the only remaining source afterwards.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedPin {
    pub revision: String,
    pub qualification_ref: String,
    pub rollout_generation: u64,
    pub pin_identity: String,
}

/// Where one session's pin stands now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinStatus {
    Live,
    /// Closed with a held checkpoint, so the lifecycle still keeps the pin.
    Closed,
    /// Closed and forgotten by the lifecycle; only the recorded pin remains.
    Released,
}

/// One session's pin with its current status.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPin {
    pub revision: String,
    pub qualification_ref: String,
    pub rollout_generation: u64,
    pub pin_identity: String,
    pub status: PinStatus,
    /// A live session may perform no further effect, or a closed session's
    /// retained output may no longer be published.
    pub is_blocked: bool,
}

/// The pin of `session`: from the lifecycle while it keeps the pin, else from
/// the embedder's recorded pin. `None` when neither knows the session.
#[must_use]
pub fn session_pin(
    lifecycle: &crate::agent_revision::state::Lifecycle,
    session: &str,
    recorded: Option<&RecordedPin>,
) -> Option<SessionPin> {
    if let Some(pin) = lifecycle.sessions.get(session) {
        let pin_identity = crate::agent_revision::identity::pin(lifecycle, session, pin).ok()?;
        let (status, is_blocked) = match pin.status {
            crate::agent_revision::state::SessionStatus::Live => {
                (PinStatus::Live, crate::agent_revision::ops::sessions::effect_permitted(lifecycle, session).is_err())
            }
            crate::agent_revision::state::SessionStatus::Closed => (
                PinStatus::Closed,
                retained_blocked(lifecycle, session, &crate::agent_revision::ops::sessions::Permit {
                    revision: pin.revision.clone(),
                    qualification_ref: pin.qualification_ref.clone(),
                    rollout_generation: pin.rollout_generation,
                }),
            ),
        };
        debug_assert!(recorded.is_none_or(|recorded| recorded.pin_identity == pin_identity));
        return Some(SessionPin {
            revision: pin.revision.clone(),
            qualification_ref: pin.qualification_ref.clone(),
            rollout_generation: pin.rollout_generation,
            pin_identity,
            status,
            is_blocked,
        });
    }
    let recorded = recorded?;
    Some(SessionPin {
        revision: recorded.revision.clone(),
        qualification_ref: recorded.qualification_ref.clone(),
        rollout_generation: recorded.rollout_generation,
        pin_identity: recorded.pin_identity.clone(),
        status: PinStatus::Released,
        is_blocked: retained_blocked(lifecycle, session, &crate::agent_revision::ops::sessions::Permit {
            revision: recorded.revision.clone(),
            qualification_ref: recorded.qualification_ref.clone(),
            rollout_generation: recorded.rollout_generation,
        }),
    })
}

fn retained_blocked(
    lifecycle: &crate::agent_revision::state::Lifecycle,
    session: &str,
    permit: &crate::agent_revision::ops::sessions::Permit,
) -> bool {
    crate::agent_revision::ops::sessions::retained_permitted(lifecycle, session, permit).is_err()
}

/// Project one lifecycle. The output is bounded by the lifecycle's bounds.
#[must_use]
pub fn slots(lifecycle: &crate::agent_revision::state::Lifecycle) -> Slots {
    let prepared = lifecycle
        .candidates
        .iter()
        .filter(|(_, candidate)| {
            matches!(candidate.preparation, crate::agent_revision::state::Preparation::Prepared { .. })
        })
        .map(|(revision, _)| revision.clone())
        .collect::<Vec<_>>();
    let mut qualified = lifecycle
        .qualifications
        .iter()
        .map(|admitted| Qualified {
            revision: admitted.revision.clone(),
            task_family: admitted.task_family.clone(),
            qualification_ref: admitted.attachment.qualification_ref.clone(),
            evidence_class: admitted.attachment.evidence_class,
            provenance: admitted.attachment.provenance,
            is_revoked: lifecycle.revoked.qualifications.contains(&admitted.attachment.qualification_ref),
        })
        .collect::<Vec<_>>();
    qualified.sort_by(|left, right| (&left.revision, &left.task_family).cmp(&(&right.revision, &right.task_family)));
    let pinned = lifecycle
        .sessions
        .iter()
        .map(|(session, pin)| Pinned {
            session: session.clone(),
            revision: pin.revision.clone(),
            status: pin.status,
            is_blocked: crate::agent_revision::ops::sessions::effect_permitted(lifecycle, session).is_err(),
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(pinned.len(), lifecycle.sessions.len());
    debug_assert!(prepared.len() <= lifecycle.candidates.len());
    Slots {
        schema: SLOTS_SCHEMA.to_owned(),
        definition: lifecycle.definition.clone(),
        desired: lifecycle.desired.clone(),
        prepared,
        qualified,
        last_usable: lifecycle.last_usable.clone(),
        selected_for_new_sessions: lifecycle.selected.as_ref().map(|selection| Selected {
            revision: selection.revision.clone(),
            rollout_generation: selection.rollout_generation,
            kind: selection.kind,
            qualification_ref: selection.qualification_ref.clone(),
        }),
        pinned,
    }
}
