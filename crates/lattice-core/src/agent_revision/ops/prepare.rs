//! Desired revision, preparation tickets and preparation results.
//!
//! A ticket binds one attempt to the desired generation it served. A result
//! for a generation that is no longer desired is kept for retention and
//! cleanup only: it never changes the desired revision, the last usable
//! revision or the selection, and it never makes the revision promotable.

// r[impl lattice.agent_revisions.preparation]
// r[impl lattice.agent_revisions.preparation.scenario.failed]
// r[impl lattice.agent_revisions.preparation.scenario.late]

type Lifecycle = crate::agent_revision::state::Lifecycle;
type Refusal = crate::agent_revision::Refusal;
type Preparation = crate::agent_revision::state::Preparation;

/// The outcome of recording a desired revision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Desire {
    Recorded { revision: String, generation: u64 },
    Unchanged { revision: String, generation: u64 },
}

/// One preparation attempt for the preparation port.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub definition: String,
    pub revision: String,
    pub generation: u64,
    pub preparation_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Begin {
    Started {
        ticket: Ticket,
    },
    /// An attempt for this generation is pending or unknown; reconcile it
    /// with the same ticket instead of starting another.
    Reissued {
        ticket: Ticket,
    },
    /// Already prepared: nothing is prepared again.
    AlreadyPrepared {
        generation: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Recorded {
    Applied {
        preparation: Preparation,
        last_usable: Option<String>,
    },
    Unchanged {
        preparation: Preparation,
    },
}

const fn is_retry(preparation: &Preparation) -> bool {
    matches!(preparation, Preparation::Failed { .. } | Preparation::Superseded { .. })
}

/// Record `bundle` as the desired revision. Desiring the current desired
/// revision again is unchanged unless its preparation failed or was
/// superseded, which starts a new generation.
pub fn desire(
    lifecycle: &Lifecycle,
    bundle: &crate::agent_revision::facts::Bundle,
) -> Result<super::Applied<Desire>, Refusal> {
    let bundle_identity = crate::agent_revision::facts::identity(bundle)?;
    if bundle.definition != lifecycle.definition {
        return Err(Refusal::DefinitionMismatch);
    }
    let revision = bundle.revision.clone();
    if lifecycle.revoked.revisions.contains(&revision) {
        return Err(Refusal::RevisionRevoked);
    }
    let existing = lifecycle.candidates.get(&revision);
    if existing.is_some_and(|candidate| candidate.bundle_identity != bundle_identity) {
        return Err(Refusal::Malformed);
    }
    let is_retrying = existing.is_some_and(|candidate| is_retry(&candidate.preparation));
    if let Some(desired) = lifecycle.desired.as_ref().filter(|desired| desired.revision == revision && !is_retrying) {
        let generation = desired.generation;
        return Ok(super::unchanged(lifecycle, Desire::Unchanged { revision, generation }));
    }
    let generation = super::next(lifecycle.change_generation)?;
    let mut next = lifecycle.clone();
    if existing.is_none() {
        if super::room(next.candidates.len(), crate::agent_revision::MAX_CANDIDATES).is_err() {
            super::forget_unreferenced(&mut next);
        }
        super::room(next.candidates.len(), crate::agent_revision::MAX_CANDIDATES)?;
    }
    next.change_generation = generation;
    next.desired = Some(crate::agent_revision::state::Desired {
        revision: revision.clone(),
        generation,
    });
    let entry = next.candidates.entry(revision.clone()).or_insert_with(|| crate::agent_revision::state::Candidate {
        bundle: bundle.clone(),
        bundle_identity,
        preparation: Preparation::Unprepared {},
        artifacts: Vec::new(),
        cleanup: crate::agent_revision::state::CleanupStatus::NotRequested,
        holds: Vec::new(),
    });
    if is_retry(&entry.preparation) {
        entry.preparation = Preparation::Unprepared {};
    }
    debug_assert!(generation > lifecycle.change_generation);
    debug_assert!(next.candidates.contains_key(&revision));
    Ok(super::Applied {
        lifecycle: next,
        outcome: Desire::Recorded { revision, generation },
    })
}

fn ticket(lifecycle: &Lifecycle, desired: &crate::agent_revision::state::Desired) -> Result<Ticket, Refusal> {
    let preparation_identity =
        crate::agent_revision::identity::preparation(&lifecycle.definition, desired).map_err(|_| Refusal::Malformed)?;
    Ok(Ticket {
        definition: lifecycle.definition.clone(),
        revision: desired.revision.clone(),
        generation: desired.generation,
        preparation_identity,
    })
}

/// Start, or reissue, preparation of the current desired revision. A revoked
/// revision never starts; its pending or unknown attempt is still reissued
/// so it can be reconciled and cleaned up.
pub fn begin(lifecycle: &Lifecycle, revision: &str) -> Result<super::Applied<Begin>, Refusal> {
    let desired = lifecycle
        .desired
        .as_ref()
        .filter(|desired| desired.revision == revision)
        .ok_or(Refusal::NotDesired)?;
    let generation = desired.generation;
    match super::candidate(lifecycle, revision)?.preparation {
        Preparation::Unprepared {} => {
            super::usable(lifecycle, revision)?;
            let mut next = lifecycle.clone();
            super::candidate_mut(&mut next, revision)?.preparation = Preparation::Pending { generation };
            let ticket = ticket(lifecycle, desired)?;
            debug_assert_eq!(ticket.generation, generation);
            debug_assert_eq!(ticket.revision, revision);
            Ok(super::Applied {
                lifecycle: next,
                outcome: Begin::Started { ticket },
            })
        }
        Preparation::Pending { generation: bound } | Preparation::Unknown { generation: bound } => {
            if bound != generation {
                return Err(Refusal::PreparationInFlight);
            }
            let ticket = ticket(lifecycle, desired)?;
            Ok(super::unchanged(lifecycle, Begin::Reissued { ticket }))
        }
        Preparation::Prepared { generation } => Ok(super::unchanged(lifecycle, Begin::AlreadyPrepared { generation })),
        Preparation::Failed { .. } => Err(Refusal::PreparationFailed),
        Preparation::Superseded { .. } => Err(Refusal::PreparationSuperseded),
    }
}

/// Reissue the ticket of `revision`'s pending or unknown attempt at the
/// generation it is bound to, whether or not the revision is still desired or
/// usable, so a late or abandoned attempt can be settled as superseded,
/// failed or unknown instead of blocking preparation for good.
pub fn reissue(lifecycle: &Lifecycle, revision: &str) -> Result<Ticket, Refusal> {
    match super::candidate(lifecycle, revision)?.preparation {
        Preparation::Pending { generation } | Preparation::Unknown { generation } => {
            let bound = crate::agent_revision::state::Desired {
                revision: revision.to_owned(),
                generation,
            };
            ticket(lifecycle, &bound)
        }
        Preparation::Unprepared {}
        | Preparation::Prepared { .. }
        | Preparation::Superseded { .. }
        | Preparation::Failed { .. } => Err(Refusal::StaleTicket),
    }
}

/// The ticket generation an attempt in `preparation` is bound to, and
/// whether it is still open for a result.
const fn binding(preparation: &Preparation) -> Option<(u64, bool)> {
    match *preparation {
        Preparation::Unprepared {} => None,
        Preparation::Pending { generation } | Preparation::Unknown { generation } => Some((generation, true)),
        Preparation::Prepared { generation }
        | Preparation::Superseded { generation }
        | Preparation::Failed { generation } => Some((generation, false)),
    }
}

fn settled(ticket: &Ticket, observation: &crate::agent_revision::ports::Prepared, is_current: bool) -> Preparation {
    let generation = ticket.generation;
    match observation {
        crate::agent_revision::ports::Prepared::Prepared { .. } if is_current => Preparation::Prepared { generation },
        crate::agent_revision::ports::Prepared::Prepared { .. } => Preparation::Superseded { generation },
        crate::agent_revision::ports::Prepared::Failed { .. } => Preparation::Failed { generation },
        crate::agent_revision::ports::Prepared::Unknown {} => Preparation::Unknown { generation },
    }
}

/// A `prepared` observation repeats a recorded `prepared` result, including
/// one recorded at a generation that is no longer desired.
const fn is_prepared_again(observation: &crate::agent_revision::ports::Prepared, recorded: &Preparation) -> bool {
    matches!(observation, crate::agent_revision::ports::Prepared::Prepared { .. })
        && matches!(recorded, Preparation::Prepared { .. } | Preparation::Superseded { .. })
}

fn artifacts(observation: &crate::agent_revision::ports::Prepared) -> &[String] {
    match observation {
        crate::agent_revision::ports::Prepared::Prepared { artifacts }
        | crate::agent_revision::ports::Prepared::Failed { artifacts } => artifacts,
        crate::agent_revision::ports::Prepared::Unknown {} => &[],
    }
}

/// Record the preparation owner's observation for `ticket`.
pub fn record(
    lifecycle: &Lifecycle,
    ticket: &Ticket,
    observation: &crate::agent_revision::ports::Prepared,
) -> Result<super::Applied<Recorded>, Refusal> {
    if ticket.definition != lifecycle.definition {
        return Err(Refusal::DefinitionMismatch);
    }
    let reported = artifacts(observation);
    if !crate::agent_revision::ports::is_reference_set(reported) {
        return Err(Refusal::Malformed);
    }
    let candidate = super::candidate(lifecycle, &ticket.revision)?;
    let is_current = lifecycle
        .desired
        .as_ref()
        .is_some_and(|desired| desired.revision == ticket.revision && desired.generation == ticket.generation);
    let preparation = settled(ticket, observation, is_current);
    let Some((bound, is_open)) = binding(&candidate.preparation).filter(|(bound, _)| *bound == ticket.generation)
    else {
        return Err(Refusal::StaleTicket);
    };
    debug_assert_eq!(bound, ticket.generation);
    if !is_open {
        let is_known = reported.iter().all(|artifact| candidate.artifacts.contains(artifact));
        let is_redelivered =
            preparation == candidate.preparation || is_prepared_again(observation, &candidate.preparation);
        return if is_redelivered && is_known {
            let preparation = candidate.preparation.clone();
            Ok(super::unchanged(lifecycle, Recorded::Unchanged { preparation }))
        } else {
            Err(Refusal::ResultConflict)
        };
    }
    let mut merged = candidate.artifacts.iter().chain(reported).cloned().collect::<std::collections::BTreeSet<_>>();
    if !crate::agent_revision::is_within(merged.len(), crate::agent_revision::MAX_REFERENCES) {
        return Err(Refusal::CapacityExhausted);
    }
    let mut next = lifecycle.clone();
    let is_usable =
        matches!(preparation, Preparation::Prepared { .. }) && !lifecycle.revoked.revisions.contains(&ticket.revision);
    if is_usable {
        next.last_usable = Some(ticket.revision.clone());
    }
    let entry = super::candidate_mut(&mut next, &ticket.revision)?;
    if !reported.is_empty() {
        entry.cleanup = crate::agent_revision::state::CleanupStatus::NotRequested;
    }
    entry.artifacts = std::mem::take(&mut merged).into_iter().collect();
    entry.preparation = preparation.clone();
    debug_assert!(merged.is_empty());
    Ok(super::Applied {
        outcome: Recorded::Applied {
            preparation,
            last_usable: next.last_usable.clone(),
        },
        lifecycle: next,
    })
}
