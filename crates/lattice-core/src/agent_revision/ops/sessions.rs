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

/// Pin `entry.session` to `entry.revision`, which must be the current selection.
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
    super::room(lifecycle.sessions.len(), crate::agent_revision::MAX_SESSIONS)?;
    let pin = Pin {
        revision: entry.revision.to_owned(),
        qualification_ref: selected.qualification_ref.clone(),
        rollout_generation: selected.rollout_generation,
        status: crate::agent_revision::state::SessionStatus::Live,
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
/// the revision's artifacts stay retained; the pin itself is never removed.
pub fn close(lifecycle: &Lifecycle, session: &str, checkpoints: &[String]) -> Result<super::Applied<Closed>, Refusal> {
    if !crate::agent_revision::ports::is_reference_set(checkpoints) {
        return Err(Refusal::Malformed);
    }
    let pin = lifecycle.sessions.get(session).ok_or(Refusal::UnknownSession)?;
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
    let is_noop = fresh.is_empty() && pin.status == crate::agent_revision::state::SessionStatus::Closed;
    candidate.holds.extend(fresh);
    let held = checkpoints.to_vec();
    if is_noop {
        return Ok(super::unchanged(lifecycle, Closed::AlreadyClosed { held_checkpoints: held }));
    }
    if let Some(closing) = next.sessions.get_mut(session) {
        closing.status = crate::agent_revision::state::SessionStatus::Closed;
    }
    debug_assert!(next.sessions.contains_key(session));
    debug_assert!(held.len() <= count);
    Ok(super::Applied {
        lifecycle: next,
        outcome: Closed::Closed { held_checkpoints: held },
    })
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

/// Record a revocation. Revocation is monotonic and never rewrites pins or
/// history; a revoked last usable revision stops being usable.
pub fn revoke(lifecycle: &Lifecycle, target: &Target) -> Result<super::Applied<Revocation>, Refusal> {
    let (value, is_wellformed) = match target {
        Target::Revision(value) => (value, crate::agent_revision::is_bare_digest(value)),
        Target::Qualification(value) => (value, crate::agent_revision::is_reference(value)),
        Target::Session(value) => (value, crate::agent_revision::is_token(value)),
    };
    if !is_wellformed {
        return Err(Refusal::Malformed);
    }
    let mut next = lifecycle.clone();
    let set = match target {
        Target::Revision(_) => &mut next.revoked.revisions,
        Target::Qualification(_) => &mut next.revoked.qualifications,
        Target::Session(_) => &mut next.revoked.sessions,
    };
    if set.contains(value) {
        return Ok(super::unchanged(lifecycle, Revocation::AlreadyRevoked));
    }
    super::room(set.len(), crate::agent_revision::MAX_REVOCATIONS)?;
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
