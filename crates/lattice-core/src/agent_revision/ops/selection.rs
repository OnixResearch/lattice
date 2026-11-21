//! Explicit promotion and rollback of the revision selected for new sessions.
//!
//! Both name an exact revision, a `qualification_ref` and the expected
//! rollout generation. Both revalidate current revocation and the current
//! qualification answer, and both are idempotent by request identifier: a
//! repeated request returns its receipt instead of selecting again, so a lost
//! acknowledgement reconciles without another preparation or selection.
//! Concurrent requests that expect the same generation select once; every
//! other one is refused with `generation_conflict`. A favorable answer is
//! evidence for the decision, never the decision itself.

// r[impl lattice.agent_revisions.promotion]
// r[impl lattice.agent_revisions.promotion.scenario.idempotent]
// r[impl lattice.agent_revisions.promotion.scenario.concurrent]
// r[impl lattice.agent_revisions.promotion.scenario.unavailable]
// r[impl lattice.agent_revisions.rollback]

type Lifecycle = crate::agent_revision::state::Lifecycle;
type Refusal = crate::agent_revision::Refusal;
type Kind = crate::agent_revision::state::SelectionKind;
type Choice = crate::agent_revision::state::Selection;

/// One promotion or rollback request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub kind: Kind,
    pub request_id: String,
    pub revision: String,
    pub qualification_ref: String,
    pub policy: crate::agent_revision::state::Policy,
    pub expected_rollout_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Selected {
    Applied {
        selection: Choice,
        promotion_identity: String,
    },
    /// The request was applied before; nothing is selected again.
    AlreadyApplied {
        selection: Choice,
        promotion_identity: String,
        is_current: bool,
    },
}

fn validate(command: &Command) -> Result<(), Refusal> {
    let checks = [
        crate::agent_revision::is_token(&command.request_id),
        crate::agent_revision::is_bare_digest(&command.revision),
        crate::agent_revision::is_reference(&command.qualification_ref),
        crate::execution_binding::capability::is_name(&command.policy.task_family),
    ];
    if checks.iter().all(|check| *check) {
        Ok(())
    } else {
        Err(Refusal::Malformed)
    }
}

/// The exact question the shell asks the qualification port for `command`.
pub fn query(
    lifecycle: &Lifecycle,
    command: &Command,
    now_ms: u64,
) -> Result<crate::agent_revision::ports::Query, Refusal> {
    validate(command)?;
    Ok(crate::agent_revision::ports::Query {
        definition_identity: lifecycle.definition.clone(),
        profile_revision: command.revision.clone(),
        task_family: command.policy.task_family.clone(),
        required_class: command.policy.required_class,
        now_ms,
    })
}

/// The receipt answers exactly this command: same kind, revision,
/// qualification, policy and the expected rollout generation it advanced.
fn is_same_request(selection: &Choice, command: &Command) -> bool {
    let fields = [
        selection.kind == command.kind,
        selection.revision == command.revision,
        selection.qualification_ref == command.qualification_ref,
        selection.policy == command.policy,
        command.expected_rollout_generation.checked_add(1) == Some(selection.rollout_generation),
    ];
    fields.iter().all(|field| *field)
}

/// Forget the oldest receipt when the window is full. Its request expected
/// an older rollout generation, so repeating it is refused, never reapplied.
fn forget_oldest_receipt(lifecycle: &mut Lifecycle) {
    if super::room(lifecycle.receipts.len(), crate::agent_revision::MAX_RECEIPTS).is_ok() {
        return;
    }
    let oldest = lifecycle
        .receipts
        .iter()
        .min_by_key(|(_, receipt)| receipt.rollout_generation)
        .map(|(request_id, _)| request_id.clone());
    if let Some(request_id) = oldest {
        lifecycle.receipts.remove(&request_id);
    }
}

fn is_in_scope(admitted: &crate::agent_revision::state::Qualification, command: &Command) -> bool {
    admitted.revision == command.revision && admitted.task_family == command.policy.task_family
}

/// Refuse a revision that cannot be selected, before asking about evidence.
fn eligible(lifecycle: &Lifecycle, command: &Command) -> Result<(), Refusal> {
    match super::candidate(lifecycle, &command.revision)?.preparation {
        crate::agent_revision::state::Preparation::Prepared { .. } => {}
        crate::agent_revision::state::Preparation::Unprepared {} => return Err(Refusal::NotPrepared),
        crate::agent_revision::state::Preparation::Pending { .. } => return Err(Refusal::PreparationInFlight),
        crate::agent_revision::state::Preparation::Unknown { .. } => return Err(Refusal::PreparationUncertain),
        crate::agent_revision::state::Preparation::Failed { .. } => return Err(Refusal::PreparationFailed),
        crate::agent_revision::state::Preparation::Superseded { .. } => return Err(Refusal::PreparationSuperseded),
    }
    super::usable(lifecycle, &command.revision)?;
    super::attested(lifecycle, &command.qualification_ref)?;
    let is_bound_elsewhere = lifecycle.qualifications.iter().any(|admitted| {
        admitted.attachment.qualification_ref == command.qualification_ref && !is_in_scope(admitted, command)
    });
    if is_bound_elsewhere {
        return Err(Refusal::QualificationMismatch);
    }
    let is_selected = lifecycle.selected.as_ref().is_some_and(|selected| selected.revision == command.revision);
    if is_selected {
        return Err(Refusal::AlreadySelected);
    }
    if command.kind == Kind::Promotion {
        debug_assert!(lifecycle.candidates.contains_key(&command.revision));
        return Ok(());
    }
    debug_assert_eq!(command.kind, Kind::Rollback);
    if !lifecycle.previously_selected.contains(&command.revision) {
        return Err(Refusal::NotPreviouslySelected);
    }
    let is_recorded = lifecycle.qualifications.iter().any(|admitted| {
        is_in_scope(admitted, command) && admitted.attachment.qualification_ref == command.qualification_ref
    });
    if is_recorded {
        Ok(())
    } else {
        Err(Refusal::QualificationMismatch)
    }
}

/// Accept only a current, canonical answer of the required class that names
/// the requested attachment.
fn accept(
    verdict: &crate::agent_revision::ports::Verdict,
    command: &Command,
) -> Result<crate::agent_revision::ports::Attachment, Refusal> {
    let attachment = match verdict {
        crate::agent_revision::ports::Verdict::Unavailable {} => {
            return Err(Refusal::QualificationUnavailable {
                owner: crate::agent_revision::QUALIFICATION_OWNER.to_owned(),
            });
        }
        crate::agent_revision::ports::Verdict::NotQualified { reason } => {
            return Err(Refusal::NotQualified { reason: *reason });
        }
        crate::agent_revision::ports::Verdict::Qualified(attachment) => attachment,
    };
    if !attachment.is_canonical() {
        return Err(Refusal::NotQualified {
            reason: crate::agent_revision::ports::Unqualified::Malformed,
        });
    }
    if attachment.evidence_class != command.policy.required_class {
        return Err(Refusal::InsufficientClass);
    }
    if attachment.qualification_ref != command.qualification_ref {
        return Err(Refusal::QualificationMismatch);
    }
    debug_assert!(attachment.is_canonical());
    debug_assert_eq!(attachment.evidence_class, command.policy.required_class);
    Ok(attachment.clone())
}

/// Apply `command` with the qualification port's current `verdict`.
pub fn select(
    lifecycle: &Lifecycle,
    command: &Command,
    verdict: &crate::agent_revision::ports::Verdict,
) -> Result<super::Applied<Selected>, Refusal> {
    validate(command)?;
    let identify = |selection: &Choice| {
        crate::agent_revision::identity::promotion(&lifecycle.definition, selection).map_err(|_| Refusal::Malformed)
    };
    if let Some(receipt) = lifecycle.receipts.get(&command.request_id) {
        if !is_same_request(receipt, command) {
            return Err(Refusal::RequestConflict);
        }
        let is_current = lifecycle.selected.as_ref() == Some(receipt);
        let outcome = Selected::AlreadyApplied {
            selection: receipt.clone(),
            promotion_identity: identify(receipt)?,
            is_current,
        };
        return Ok(super::unchanged(lifecycle, outcome));
    }
    eligible(lifecycle, command)?;
    let attachment = accept(verdict, command)?;
    if command.expected_rollout_generation != lifecycle.rollout_generation {
        return Err(Refusal::GenerationConflict);
    }
    let mut next = lifecycle.clone();
    forget_oldest_receipt(&mut next);
    super::room(next.receipts.len(), crate::agent_revision::MAX_RECEIPTS)?;
    let selection = Choice {
        kind: command.kind,
        request_id: command.request_id.clone(),
        revision: command.revision.clone(),
        qualification_ref: command.qualification_ref.clone(),
        policy: command.policy.clone(),
        policy_identity: crate::agent_revision::identity::policy(&command.policy).map_err(|_| Refusal::Malformed)?,
        rollout_generation: super::next(lifecycle.rollout_generation)?,
    };
    let promotion_identity = identify(&selection)?;
    next.qualifications.retain(|admitted| !is_in_scope(admitted, command));
    if super::room(next.qualifications.len(), crate::agent_revision::MAX_QUALIFICATIONS).is_err() {
        super::forget_unreferenced(&mut next);
    }
    super::room(next.qualifications.len(), crate::agent_revision::MAX_QUALIFICATIONS)?;
    next.qualifications.push(crate::agent_revision::state::Qualification {
        revision: command.revision.clone(),
        task_family: command.policy.task_family.clone(),
        attachment,
    });
    next.rollout_generation = selection.rollout_generation;
    next.previously_selected.insert(command.revision.clone());
    next.receipts.insert(command.request_id.clone(), selection.clone());
    next.selected = Some(selection.clone());
    debug_assert!(next.rollout_generation > lifecycle.rollout_generation);
    debug_assert!(next.receipts.contains_key(&command.request_id));
    Ok(super::Applied {
        lifecycle: next,
        outcome: Selected::Applied {
            selection,
            promotion_identity,
        },
    })
}
