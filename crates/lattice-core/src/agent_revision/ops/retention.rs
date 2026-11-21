//! Retention holds and cleanup planning.
//!
//! A revision's artifacts stay retained while it is desired (unless its
//! preparation failed), selected or
//! last usable, while a live session pins it, while a checkpoint or recovery
//! evidence holds it, or while its preparation is pending or unknown. Only an
//! observed `released` cleanup frees artifacts; a failed or unknown cleanup
//! keeps them occupied and can be retried. Storage and closure effects stay
//! with the preparation owner.

// r[impl lattice.agent_revisions.retention]
// r[impl lattice.agent_revisions.retention.scenario.uncertain]

type Lifecycle = crate::agent_revision::state::Lifecycle;
type Refusal = crate::agent_revision::Refusal;

/// Why a revision's artifacts are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Desired,
    Selected,
    LastUsable,
    LiveSession,
    Checkpoint,
    RecoveryEvidence,
    PreparationInFlight,
    PreparationUncertain,
}

const MAX_REASONS: usize = 8;

/// Every reason `revision` is retained, sorted and deduplicated.
#[must_use]
pub fn retained_by(lifecycle: &Lifecycle, revision: &str) -> Vec<Reason> {
    let mut reasons = std::collections::BTreeSet::new();
    let is_named = |value: Option<&String>| value.is_some_and(|value| value == revision);
    let is_failed = lifecycle.candidates.get(revision).is_some_and(|candidate| {
        matches!(candidate.preparation, crate::agent_revision::state::Preparation::Failed { .. })
    });
    if is_named(lifecycle.desired.as_ref().map(|desired| &desired.revision)) && !is_failed {
        reasons.insert(Reason::Desired);
    }
    if is_named(lifecycle.selected.as_ref().map(|selected| &selected.revision)) {
        reasons.insert(Reason::Selected);
    }
    if is_named(lifecycle.last_usable.as_ref()) {
        reasons.insert(Reason::LastUsable);
    }
    let is_pinned_live = lifecycle
        .sessions
        .values()
        .any(|pin| pin.revision == revision && pin.status == crate::agent_revision::state::SessionStatus::Live);
    if is_pinned_live {
        reasons.insert(Reason::LiveSession);
    }
    if let Some(candidate) = lifecycle.candidates.get(revision) {
        reasons.extend(candidate.holds.iter().map(|hold| match hold.kind {
            crate::agent_revision::state::HoldKind::Checkpoint => Reason::Checkpoint,
            crate::agent_revision::state::HoldKind::RecoveryEvidence => Reason::RecoveryEvidence,
        }));
        match candidate.preparation {
            crate::agent_revision::state::Preparation::Pending { .. } => {
                reasons.insert(Reason::PreparationInFlight);
            }
            crate::agent_revision::state::Preparation::Unknown { .. } => {
                reasons.insert(Reason::PreparationUncertain);
            }
            crate::agent_revision::state::Preparation::Unprepared {}
            | crate::agent_revision::state::Preparation::Prepared { .. }
            | crate::agent_revision::state::Preparation::Superseded { .. }
            | crate::agent_revision::state::Preparation::Failed { .. } => {}
        }
    }
    debug_assert!(reasons.iter().all(|reason| *reason <= Reason::PreparationUncertain));
    debug_assert!(reasons.len() <= MAX_REASONS);
    reasons.into_iter().collect()
}

/// Add a checkpoint or recovery-evidence hold to `revision`.
pub fn hold(
    lifecycle: &Lifecycle,
    revision: &str,
    hold: &crate::agent_revision::state::Hold,
) -> Result<super::Applied<bool>, Refusal> {
    if !crate::agent_revision::is_reference(&hold.reference) {
        return Err(Refusal::Malformed);
    }
    let candidate = super::candidate(lifecycle, revision)?;
    if candidate.holds.contains(hold) {
        return Ok(super::unchanged(lifecycle, false));
    }
    super::room(candidate.holds.len(), crate::agent_revision::MAX_REFERENCES)?;
    let mut next = lifecycle.clone();
    super::candidate_mut(&mut next, revision)?.holds.push(hold.clone());
    Ok(super::Applied {
        lifecycle: next,
        outcome: true,
    })
}

/// Remove a hold once its checkpoint or evidence no longer needs the revision.
/// A closed session none of whose checkpoints is still held is forgotten.
pub fn release(
    lifecycle: &Lifecycle,
    revision: &str,
    hold: &crate::agent_revision::state::Hold,
) -> Result<super::Applied<bool>, Refusal> {
    let candidate = super::candidate(lifecycle, revision)?;
    if !candidate.holds.contains(hold) {
        return Ok(super::unchanged(lifecycle, false));
    }
    let mut next = lifecycle.clone();
    super::candidate_mut(&mut next, revision)?.holds.retain(|held| held != hold);
    super::sessions::forget_unheld(&mut next, revision);
    Ok(super::Applied {
        lifecycle: next,
        outcome: true,
    })
}

/// One cleanup the preparation owner may perform.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    pub definition: String,
    pub revision: String,
    pub artifacts: Vec<String>,
}

/// Revisions whose occupied artifacts nothing retains, in revision order.
#[must_use]
pub fn plan(lifecycle: &Lifecycle) -> Vec<Sweep> {
    lifecycle
        .candidates
        .iter()
        .filter(|(_, candidate)| !candidate.artifacts.is_empty())
        .filter(|(revision, _)| retained_by(lifecycle, revision).is_empty())
        .map(|(revision, candidate)| Sweep {
            definition: lifecycle.definition.clone(),
            revision: revision.clone(),
            artifacts: candidate.artifacts.clone(),
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Swept {
    Released {
        artifacts: Vec<String>,
    },
    /// The artifacts stay occupied until a later cleanup is observed.
    Uncertain {
        artifacts: Vec<String>,
    },
}

/// Record a cleanup observation. Nothing is released unless the owner
/// observed release, and a retained revision is refused before the record.
pub fn record(
    lifecycle: &Lifecycle,
    revision: &str,
    observation: crate::agent_revision::ports::Cleanup,
) -> Result<super::Applied<Swept>, Refusal> {
    if !retained_by(lifecycle, revision).is_empty() {
        return Err(Refusal::Retained);
    }
    let artifacts = super::candidate(lifecycle, revision)?.artifacts.clone();
    let mut next = lifecycle.clone();
    let candidate = super::candidate_mut(&mut next, revision)?;
    let outcome = match observation {
        crate::agent_revision::ports::Cleanup::Released => {
            candidate.artifacts.clear();
            candidate.cleanup = crate::agent_revision::state::CleanupStatus::Released;
            if matches!(candidate.preparation, crate::agent_revision::state::Preparation::Prepared { .. }) {
                candidate.preparation = crate::agent_revision::state::Preparation::Unprepared {};
            }
            Swept::Released { artifacts }
        }
        crate::agent_revision::ports::Cleanup::Failed | crate::agent_revision::ports::Cleanup::Unknown => {
            candidate.cleanup = crate::agent_revision::state::CleanupStatus::Uncertain;
            Swept::Uncertain { artifacts }
        }
    };
    debug_assert!(next.candidates.contains_key(revision));
    debug_assert_eq!(next.sessions, lifecycle.sessions);
    Ok(super::Applied {
        lifecycle: next,
        outcome,
    })
}
