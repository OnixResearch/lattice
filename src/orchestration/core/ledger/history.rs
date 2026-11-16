#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    RoundStarted,
    RetryRoundStarted,
    DispatchAcknowledged,
    RoundCompleted,
    RoundFailed,
    ProposalSubmitted,
    ProposalApproved,
    ProposalRejected,
    Paused,
    Resumed,
    CancellationRequested,
    CancellationObserved,
    DispatchAbandoned,
    BudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub instance_id: crate::orchestration::identity::InstanceId,
    pub sequence: u64,
    pub prior_hash: Option<crate::orchestration::identity::PreviousEventId>,
    pub event_hash: crate::orchestration::identity::EventId,
    pub state_hash: crate::orchestration::identity::StateDigest,
    pub kind: EventKind,
    pub fact: super::super::Fact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub instance: super::super::Instance,
    pub event: Option<Event>,
    pub duplicate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReplayReport {
    pub valid: bool,
    pub instance: super::super::Instance,
    pub diagnostics: Vec<super::super::Diagnostic>,
}

pub(in crate::orchestration::core) fn build_event(
    prior: &super::super::Instance,
    next: &super::super::Instance,
    kind: EventKind,
    fact: super::super::Fact,
) -> Result<Event, super::super::Diagnostic> {
    debug_assert_eq!(prior.id, next.id);
    debug_assert_eq!(next.version, prior.version.saturating_add(super::super::INSTANCE_VERSION_INCREMENT));
    let sequence = prior
        .last_event_sequence
        .map(|sequence| sequence.saturating_add(super::super::EVENT_SEQUENCE_INCREMENT))
        .unwrap_or(super::super::FIRST_EVENT_SEQUENCE);
    let state_hash = semantic_state_hash(next)?;
    let mut event = Event {
        instance_id: next.id.clone(),
        sequence,
        prior_hash: prior.event_head.as_ref().map(crate::orchestration::identity::PreviousEventId::from_ledger_head),
        event_hash: crate::orchestration::identity::EventId::from_generated(
            "0".repeat(blake3::OUT_LEN.saturating_mul(super::super::HEX_CHARS_PER_BYTE)),
        ),
        state_hash,
        kind,
        fact,
    };
    event.event_hash = event_identity(&event)?;
    Ok(event)
}

pub(super) fn event_identity(
    event: &Event,
) -> Result<crate::orchestration::identity::EventId, super::super::Diagnostic> {
    let mut unhashed = event.clone();
    unhashed.event_hash.clear_for_hash();
    Ok(crate::orchestration::identity::EventId::from_generated(
        super::super::model::canonical::hash_serializable(super::super::EVENT_HASH_LABEL, &unhashed)?,
    ))
}

pub(super) fn semantic_state_hash(
    instance: &super::super::Instance,
) -> Result<crate::orchestration::identity::StateDigest, super::super::Diagnostic> {
    let mut state = instance.clone();
    state.last_event_sequence = None;
    state.event_head = None;
    Ok(crate::orchestration::identity::StateDigest::from_generated(
        super::super::model::canonical::hash_serializable(super::super::STATE_HASH_LABEL, &state)?,
    ))
}

// r[impl lattice.agent_orchestration.events]
// r[impl lattice.agent_orchestration.nominal_identities.events]
pub fn replay_events(
    initial: &super::super::Instance,
    events: &[Event],
    expected_head: Option<&crate::orchestration::identity::LedgerHeadId>,
) -> ReplayReport {
    let mut instance = initial.clone();
    let mut diagnostics = Vec::with_capacity(super::super::MAX_REPLAY_DIAGNOSTICS);
    for recorded in events {
        match replay_recorded(&instance, recorded) {
            Ok(next) => instance = next,
            Err(error) => {
                diagnostics.push(error);
                break;
            }
        }
    }
    debug_assert_eq!(instance.id, initial.id, "replay cannot change the instance identity");
    if replay_head_mismatch(&diagnostics, instance.event_head.as_ref(), expected_head) {
        diagnostics.push(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ReplayHeadMismatch,
            "orchestration event chain does not match the supplied current head or checkpoint",
        ));
    }
    debug_assert!(diagnostics.len() <= super::super::MAX_REPLAY_DIAGNOSTICS, "replay stops at its first mismatch");
    ReplayReport {
        valid: diagnostics.is_empty(),
        instance,
        diagnostics,
    }
}

fn replay_recorded(
    instance: &super::super::Instance,
    recorded: &Event,
) -> Result<super::super::Instance, super::super::Diagnostic> {
    let transition = super::super::transition(instance, recorded.fact.clone()).map_err(|error| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ReplayMismatch,
            format!("orchestration replay rejected event {}: {}", recorded.sequence, error.message),
        )
    })?;
    let generated = transition.event.ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ReplayMismatch,
            "replay event repeats an already accepted fact identity",
        )
    })?;
    if generated != *recorded {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::ReplayMismatch,
            format!("orchestration event sequence {} does not match deterministic replay", recorded.sequence),
        ));
    }
    debug_assert!(!transition.duplicate, "a replayed event must advance the accepted fact set");
    debug_assert_eq!(transition.instance.last_event_sequence, Some(recorded.sequence));
    Ok(transition.instance)
}

pub(super) fn replay_head_mismatch(
    diagnostics: &[super::super::Diagnostic],
    actual_head: Option<&crate::orchestration::identity::LedgerHeadId>,
    expected_head: Option<&crate::orchestration::identity::LedgerHeadId>,
) -> bool {
    if !diagnostics.is_empty() {
        return false;
    }
    actual_head != expected_head
}
