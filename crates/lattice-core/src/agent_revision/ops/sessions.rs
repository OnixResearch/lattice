//! Session pins, revocation and the effect check.
//!
//! Admission pins the revision selected for new sessions. A pin preserves
//! configuration, not privileges: [`effect_permitted`] refuses every
//! subsequent effect of a session whose own record, pinned revision or
//! pinned qualification is revoked, on old and new sessions alike.
//! Cancellation, deadlines, grants and budgets keep their existing owners and
//! are checked there in addition to this.

// r[impl lattice.agent_revisions.sessions]
// r[impl lattice.agent_revisions.sessions.scenario.pinned]
// r[impl lattice.agent_revisions.revocation]
// r[impl lattice.agent_revisions.revocation.scenario.effects]

type Lifecycle = crate::agent_revision::state::Lifecycle;
type Refusal = crate::agent_revision::Refusal;
type Pin = crate::agent_revision::state::Pin;

/// One session asking to be admitted under one revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry<'a> {
    pub session: &'a str,
    /// The bare-hex revision of the session's own configuration.
    pub revision: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Admitted {
    Pinned {
        pin: Pin,
        pin_identity: String,
    },
    /// The session was admitted before under the same revision.
    AlreadyPinned {
        pin: Pin,
        pin_identity: String,
    },
}

/// Pin `entry.session` to `entry.revision`, which must be the current
/// selection. Re-admitting a pinned session returns its pin only while the
/// session is live and its revision and qualification are not revoked.
pub fn admit(lifecycle: &Lifecycle, entry: Entry<'_>) -> Result<super::Applied<Admitted>, Refusal> {
    let checks = [
        crate::agent_revision::is_token(entry.session),
        crate::agent_revision::is_bare_digest(entry.revision),
    ];
    if !checks.iter().all(|check| *check) {
        return Err(Refusal::Malformed);
    }
    if lifecycle.revoked.sessions.contains(entry.session) {
        return Err(Refusal::SessionRevoked);
    }
    let identify =
        |pin: &Pin| crate::agent_revision::identity::pin(lifecycle, entry.session, pin).map_err(|_| Refusal::Malformed);
    if let Some(pin) = lifecycle.sessions.get(entry.session) {
        if pin.revision != entry.revision {
            return Err(Refusal::SessionConflict);
        }
        if pin.status == crate::agent_revision::state::SessionStatus::Closed {
            return Err(Refusal::SessionClosed);
        }
        super::usable(lifecycle, &pin.revision)?;
        super::attested(lifecycle, &pin.qualification_ref)?;
        let outcome = Admitted::AlreadyPinned {
            pin: pin.clone(),
            pin_identity: identify(pin)?,
        };
        return Ok(super::unchanged(lifecycle, outcome));
    }
    let selected = lifecycle.selected.as_ref().ok_or(Refusal::NoSelection)?;
    if selected.revision != entry.revision {
        return Err(Refusal::RevisionNotSelected);
    }
    super::usable(lifecycle, entry.revision)?;
    super::attested(lifecycle, &selected.qualification_ref)?;
    let live = lifecycle
        .sessions
        .values()
        .filter(|pin| pin.status == crate::agent_revision::state::SessionStatus::Live)
        .count();
    super::room(live, crate::agent_revision::MAX_SESSIONS)?;
    let pin = Pin {
        revision: entry.revision.to_owned(),
        qualification_ref: selected.qualification_ref.clone(),
        rollout_generation: selected.rollout_generation,
        status: crate::agent_revision::state::SessionStatus::Live,
        checkpoints: Vec::new(),
    };
    let pin_identity = identify(&pin)?;
    let mut next = lifecycle.clone();
    next.sessions.insert(entry.session.to_owned(), pin.clone());
    debug_assert_eq!(pin.revision, selected.revision);
    debug_assert!(next.sessions.contains_key(entry.session));
    Ok(super::Applied {
        lifecycle: next,
        outcome: Admitted::Pinned { pin, pin_identity },
    })
}

/// What a session may still use for its next protected effect.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Permit {
    pub revision: String,
    pub qualification_ref: String,
    pub rollout_generation: u64,
}

/// Whether `session` may perform another protected effect under its pin.
pub fn effect_permitted(lifecycle: &Lifecycle, session: &str) -> Result<Permit, Refusal> {
    let pin = lifecycle.sessions.get(session).ok_or(Refusal::UnknownSession)?;
    if pin.status == crate::agent_revision::state::SessionStatus::Closed {
        return Err(Refusal::SessionClosed);
    }
    if lifecycle.revoked.sessions.contains(session) {
        return Err(Refusal::SessionRevoked);
    }
    super::usable(lifecycle, &pin.revision)?;
    super::attested(lifecycle, &pin.qualification_ref)?;
    Ok(Permit {
        revision: pin.revision.clone(),
        qualification_ref: pin.qualification_ref.clone(),
        rollout_generation: pin.rollout_generation,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Closed {
    Closed { held_checkpoints: Vec<String> },
    AlreadyClosed { held_checkpoints: Vec<String> },
}

/// Close `session`. Its checkpoints become holds on the pinned revision, so
/// the revision's artifacts stay retained. The closed pin is kept only while
/// one of its checkpoints is held; a session closed without a held checkpoint
/// is forgotten, so repeating its close answers `unknown_session`.
pub fn close(lifecycle: &Lifecycle, session: &str, checkpoints: &[String]) -> Result<super::Applied<Closed>, Refusal> {
    if !crate::agent_revision::ports::is_reference_set(checkpoints) {
        return Err(Refusal::Malformed);
    }
    let pin = lifecycle.sessions.get(session).ok_or(Refusal::UnknownSession)?;
    let kept = pin.checkpoints.iter().chain(checkpoints).cloned().collect::<std::collections::BTreeSet<_>>();
    if !crate::agent_revision::is_within(kept.len(), crate::agent_revision::MAX_REFERENCES) {
        return Err(Refusal::CapacityExhausted);
    }
    let mut next = lifecycle.clone();
    let candidate = super::candidate_mut(&mut next, &pin.revision)?;
    let fresh = checkpoints
        .iter()
        .filter(|checkpoint| !candidate.holds.iter().any(|hold| hold.reference == **checkpoint))
        .map(|checkpoint| crate::agent_revision::state::Hold {
            kind: crate::agent_revision::state::HoldKind::Checkpoint,
            reference: checkpoint.clone(),
        })
        .collect::<Vec<_>>();
    let count = candidate.holds.len().checked_add(fresh.len()).ok_or(Refusal::CapacityExhausted)?;
    if !crate::agent_revision::is_within(count, crate::agent_revision::MAX_REFERENCES) {
        return Err(Refusal::CapacityExhausted);
    }
    let is_closed = pin.status == crate::agent_revision::state::SessionStatus::Closed;
    let held = checkpoints.to_vec();
    if fresh.is_empty() && is_closed && kept.len() == pin.checkpoints.len() {
        return Ok(super::unchanged(lifecycle, Closed::AlreadyClosed { held_checkpoints: held }));
    }
    candidate.holds.extend(fresh);
    if let Some(closing) = next.sessions.get_mut(session) {
        closing.status = crate::agent_revision::state::SessionStatus::Closed;
        closing.checkpoints = kept.into_iter().collect();
    }
    forget_unheld(&mut next, &pin.revision);
    debug_assert!(held.len() <= count);
    debug_assert!(next.sessions.get(session).is_none_or(|closed| !closed.checkpoints.is_empty()));
    Ok(super::Applied {
        lifecycle: next,
        outcome: Closed::Closed { held_checkpoints: held },
    })
}

/// Forget closed sessions pinned to `revision` none of whose checkpoints is
/// still held there, and drop each released checkpoint from the pins kept.
pub(crate) fn forget_unheld(lifecycle: &mut Lifecycle, revision: &str) {
    let held = lifecycle.candidates.get(revision).map_or_else(std::collections::BTreeSet::new, |candidate| {
        candidate.holds.iter().map(|hold| hold.reference.clone()).collect()
    });
    lifecycle.sessions.retain(|_, pin| {
        if pin.revision != revision || pin.status == crate::agent_revision::state::SessionStatus::Live {
            return true;
        }
        pin.checkpoints.retain(|checkpoint| held.contains(checkpoint));
        !pin.checkpoints.is_empty()
    });
}

/// What one revocation names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Target {
    Revision(String),
    Qualification(String),
    Session(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Revocation {
    Revoked,
    AlreadyRevoked,
}

/// Whether the lifecycle references `target`: a revision it retains, a
/// qualification it records or pins, or a session it pins.
fn is_referenced(lifecycle: &Lifecycle, target: &Target) -> bool {
    match target {
        Target::Revision(revision) => lifecycle.candidates.contains_key(revision),
        Target::Qualification(reference) => {
            let is_recorded =
                lifecycle.qualifications.iter().any(|admitted| admitted.attachment.qualification_ref == *reference);
            is_recorded || lifecycle.sessions.values().any(|pin| pin.qualification_ref == *reference)
        }
        Target::Session(session) => lifecycle.sessions.contains_key(session),
    }
}

/// Record a revocation. Revocation is monotonic and never rewrites pins or
/// history; a revoked last usable revision stops being usable. Revoking a
/// target the lifecycle references is never refused; the per-kind bound
/// applies only to targets it does not reference.
pub fn revoke(lifecycle: &Lifecycle, target: &Target) -> Result<super::Applied<Revocation>, Refusal> {
    let (value, is_wellformed) = match target {
        Target::Revision(value) => (value, crate::agent_revision::is_bare_digest(value)),
        Target::Qualification(value) => (value, crate::agent_revision::is_reference(value)),
        Target::Session(value) => (value, crate::agent_revision::is_token(value)),
    };
    if !is_wellformed {
        return Err(Refusal::Malformed);
    }
    let is_referenced = is_referenced(lifecycle, target);
    let mut next = lifecycle.clone();
    let set = match target {
        Target::Revision(_) => &mut next.revoked.revisions,
        Target::Qualification(_) => &mut next.revoked.qualifications,
        Target::Session(_) => &mut next.revoked.sessions,
    };
    if set.contains(value) {
        return Ok(super::unchanged(lifecycle, Revocation::AlreadyRevoked));
    }
    if !is_referenced {
        super::room(set.len(), crate::agent_revision::MAX_REVOCATIONS)?;
    }
    set.insert(value.clone());
    debug_assert!(set.contains(value));
    if next.last_usable.as_ref() == Some(value) && matches!(target, Target::Revision(_)) {
        next.last_usable = None;
    }
    debug_assert_eq!(next.sessions, lifecycle.sessions);
    Ok(super::Applied {
        lifecycle: next,
        outcome: Revocation::Revoked,
    })
}
