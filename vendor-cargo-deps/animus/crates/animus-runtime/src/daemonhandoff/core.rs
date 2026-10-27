//! Pure strict-idle, transaction, claim, readiness, and rollback reducers.

type ActivityState = super::data::ActivityState;
type AuthenticationObservation = super::data::AuthenticationObservation;
type CancellationCause = super::data::CancellationCause;
type CancellationCommitObservation = super::data::CancellationCommitObservation;
type CancellationIdentity = super::data::CancellationIdentity;
type CancellationPlan = super::data::CancellationPlan;
type ClaimIdentity = super::data::ClaimIdentity;
type CompletionObservation = super::data::CompletionObservation;
type DurableHandoffState = super::data::DurableHandoffState;
type Error = super::data::Error;
type HandoffBinding = super::data::HandoffBinding;
type HandoffIdentity = super::data::HandoffIdentity;
type HandoffState = super::data::HandoffState;
type HandoffTransaction = super::data::HandoffTransaction;
type IngressPauseObservation = super::data::IngressPauseObservation;
type Issue = super::data::Issue;
type LeaseReleaseObservation = super::data::LeaseReleaseObservation;
type LeaseRestoreObservation = super::data::LeaseRestoreObservation;
type ObservationIdentity = super::data::ObservationIdentity;
type ProcessAttemptIdentity = super::data::ProcessAttemptIdentity;
type ProcessStartObservation = super::data::ProcessStartObservation;
type ProcessTreeTerminationObservation = super::data::ProcessTreeTerminationObservation;
type ReadinessCheck = super::data::ReadinessCheck;
type ReadinessIdentity = super::data::ReadinessIdentity;
type ReadinessObservation = super::data::ReadinessObservation;
type ReplacementClaim = super::data::ReplacementClaim;
type SnapshotIdentity = super::data::SnapshotIdentity;
type StrictIdleDecision = super::data::StrictIdleDecision;
type StrictIdleSnapshot = super::data::StrictIdleSnapshot;
type TerminationIdentity = super::data::TerminationIdentity;
type TransitionIdentity = super::data::TransitionIdentity;
type TransitionSequence = super::data::TransitionSequence;
type WorkClass = super::data::WorkClass;

/// Supported local handoff protocol version.
pub const HANDOFF_PROTOCOL_VERSION: u16 = 1;
/// Maximum bytes in one nominal identity.
pub const MAX_IDENTITY_BYTES: usize = 256;
/// BLAKE3 hexadecimal identity length.
pub const BLAKE3_HEX_BYTES: usize = 64;

const MIN_GENERATION: u64 = 1;
const FIRST_TRANSITION: u64 = 1;
const NEXT_GENERATION_INCREMENT: u64 = 1;
const FRAME_LENGTH_BYTES: usize = core::mem::size_of::<u64>();
const FRAME_LENGTH_OVERFLOW_TAG: &[u8] = b"animus.daemon-handoff.frame-length-overflow.v1";
const IDLE_SNAPSHOT_DOMAIN: &str = "animus.daemon-handoff.idle.v1";
const HANDOFF_DOMAIN: &str = "animus.daemon-handoff.transaction.v1";
const OBSERVATION_DOMAIN: &str = "animus.daemon-handoff.observation.v1";
const TRANSITION_DOMAIN: &str = "animus.daemon-handoff.transition.v1";
const CLAIM_DOMAIN: &str = "animus.daemon-handoff.claim.v1";
const READINESS_DOMAIN: &str = "animus.daemon-handoff.readiness.v1";
const CANCELLATION_DOMAIN: &str = "animus.daemon-handoff.cancellation.v1";
const TERMINATION_DOMAIN: &str = "animus.daemon-handoff.termination.v1";

/// Validates exact strict-idle facts and binds their canonical identity.
///
/// # Errors
///
/// Returns a bounded identity, generation, or busy-work issue.
// r[impl agent_harness_runtime.daemon_handoff.idle]
pub fn validate_strict_idle(snapshot: &StrictIdleSnapshot) -> Result<StrictIdleDecision, Error> {
    validate_text(snapshot.worker.as_str())?;
    validate_text(snapshot.session.as_str())?;
    validate_text(snapshot.branch.as_str())?;
    validate_text(snapshot.authority_profile.as_str())?;
    validate_generation(snapshot.lease_generation.value())?;
    validate_generation(snapshot.primary_queue_generation.value())?;
    validate_generation(snapshot.child_queue_generation.value())?;
    validate_generation(snapshot.steering_generation.value())?;
    validate_generation(snapshot.active_turn_generation.value())?;
    require_idle(snapshot.provider, WorkClass::Provider)?;
    require_idle(snapshot.tool, WorkClass::Tool)?;
    require_idle(snapshot.repair, WorkClass::Repair)?;
    require_idle(snapshot.confirmation, WorkClass::Confirmation)?;
    require_idle(snapshot.cancellation, WorkClass::Cancellation)?;
    require_idle(snapshot.durable_commit, WorkClass::DurableCommit)?;
    require_idle(snapshot.terminal_transition, WorkClass::TerminalTransition)?;
    require_zero(snapshot.primary_queue_depth, WorkClass::PrimaryQueue)?;
    require_zero(snapshot.child_queue_depth, WorkClass::ChildQueue)?;
    require_zero(snapshot.steering_depth, WorkClass::Steering)?;

    let snapshot_identity = SnapshotIdentity::new(strict_idle_digest(snapshot));
    debug_assert!(!snapshot_identity.as_str().is_empty());
    Ok(StrictIdleDecision {
        snapshot: snapshot.clone(),
        snapshot_identity,
    })
}

/// Creates the first monotonic handoff transition from exact strict idle.
///
/// # Errors
///
/// Returns an error if the decision was forged or its facts no longer validate.
// r[impl agent_harness_runtime.daemon_handoff.transaction]
pub fn prepare_handoff(decision: &StrictIdleDecision) -> Result<HandoffTransaction, Error> {
    let validated = validate_strict_idle(&decision.snapshot)?;
    if validated.snapshot_identity != decision.snapshot_identity {
        return Err(Error::new(Issue::Conflicting));
    }
    let handoff = HandoffIdentity::new(digest(
        HANDOFF_DOMAIN,
        &[decision.snapshot_identity.as_str()],
    ));
    let binding = HandoffBinding {
        old_worker: decision.snapshot.worker.clone(),
        session: decision.snapshot.session.clone(),
        branch: decision.snapshot.branch.clone(),
        lease_generation: decision.snapshot.lease_generation,
        primary_queue_generation: decision.snapshot.primary_queue_generation,
        child_queue_generation: decision.snapshot.child_queue_generation,
        steering_generation: decision.snapshot.steering_generation,
        active_turn_generation: decision.snapshot.active_turn_generation,
        authority_profile: decision.snapshot.authority_profile.clone(),
        idle_snapshot: decision.snapshot_identity.clone(),
    };
    let observation = ObservationIdentity::new(decision.snapshot_identity.as_str());
    let transaction = initial_transaction(binding, handoff, observation);
    debug_assert_eq!(transaction.state, HandoffState::Prepared);
    Ok(transaction)
}

/// Advances a prepared handoff after exact ingress pause.
///
/// # Errors
///
/// Returns stale, conflicting, or invalid-transition before another effect.
// r[impl agent_harness_runtime.daemon_handoff.transaction]
pub fn pause_ingress(
    current: &HandoffTransaction,
    observation: &IngressPauseObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    require_state(current, HandoffState::Prepared)?;
    if observation.handoff != current.handoff {
        return Err(Error::new(Issue::Stale));
    }
    if observation.primary_queue_generation != current.binding.primary_queue_generation {
        return Err(Error::new(Issue::Stale));
    }
    if observation.child_queue_generation != current.binding.child_queue_generation {
        return Err(Error::new(Issue::Stale));
    }
    if observation.steering_generation != current.binding.steering_generation {
        return Err(Error::new(Issue::Stale));
    }
    if !observation.all_ingress_paused {
        return Err(Error::new(Issue::Conflicting));
    }
    let identity = observation_digest(
        "ingress_paused",
        &[
            observation.handoff.as_str(),
            &observation.primary_queue_generation.value().to_string(),
            &observation.child_queue_generation.value().to_string(),
            &observation.steering_generation.value().to_string(),
            bool_label(observation.all_ingress_paused),
        ],
    );
    let next = next_transaction(current, HandoffState::IngressPaused, identity)?;
    debug_assert_eq!(next.state, HandoffState::IngressPaused);
    Ok(next)
}

/// Advances after durable session-state commit and exact lease release.
///
/// # Errors
///
/// Returns drift, stale lease, or invalid-transition before replacement start.
// r[impl agent_harness_runtime.daemon_handoff.transaction]
// r[impl agent_harness_runtime.daemon_handoff.session_state]
pub fn release_lease(
    current: &HandoffTransaction,
    durable: &DurableHandoffState,
    observation: &LeaseReleaseObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    require_state(current, HandoffState::IngressPaused)?;
    validate_durable_state(current, durable)?;
    if observation.handoff != current.handoff
        || observation.released_generation != current.binding.lease_generation
        || observation.durable_commit != durable.commit
    {
        return Err(Error::new(Issue::LeaseMismatch));
    }
    if !observation.released {
        return Err(Error::new(Issue::Conflicting));
    }
    let identity = observation_digest(
        "lease_released",
        &[
            observation.handoff.as_str(),
            &observation.released_generation.value().to_string(),
            observation.durable_commit.as_str(),
            durable.session_head.as_str(),
            durable.composer_draft.as_str(),
            &durable.vault_generation.value().to_string(),
            &durable.active_turn_generation.value().to_string(),
            durable.authority_profile.as_str(),
            durable.policy.as_str(),
            bool_label(observation.released),
        ],
    );
    let mut next = next_transaction(current, HandoffState::LeaseReleased, identity)?;
    next.durable_state = Some(durable.clone());
    next.released_generation = Some(observation.released_generation);
    debug_assert!(next.durable_state.is_some());
    Ok(next)
}

/// Advances after one replacement process tree starts.
///
/// # Errors
///
/// Returns stale, invalid, or service-failure without recording a started tree.
// r[impl agent_harness_runtime.daemon_handoff.transaction]
pub fn start_replacement(
    current: &HandoffTransaction,
    observation: &ProcessStartObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    require_state(current, HandoffState::LeaseReleased)?;
    validate_text(observation.process_attempt.as_str())?;
    validate_text(observation.process.as_str())?;
    validate_text(observation.endpoint.as_str())?;
    if observation.handoff != current.handoff {
        return Err(Error::new(Issue::Stale));
    }
    if !observation.started {
        return Err(Error::new(Issue::ServiceFailure));
    }
    let identity = observation_digest(
        "replacement_started",
        &[
            observation.handoff.as_str(),
            observation.process_attempt.as_str(),
            observation.process.as_str(),
            observation.endpoint.as_str(),
            bool_label(observation.started),
        ],
    );
    let mut next = next_transaction(current, HandoffState::ReplacementStarted, identity)?;
    next.process_attempt = Some(observation.process_attempt.clone());
    next.process = Some(observation.process.clone());
    next.endpoint = Some(observation.endpoint.clone());
    debug_assert!(next.process.is_some());
    Ok(next)
}

/// Admits one exact authenticated replacement claimant.
///
/// # Errors
///
/// Returns stale, duplicate, unauthenticated, cancelled, or invalid before claim ownership.
// r[impl agent_harness_runtime.daemon_handoff.claim]
pub fn admit_claim(
    current: &HandoffTransaction,
    claim: &ReplacementClaim,
    authentication: &AuthenticationObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    match current.state {
        HandoffState::Cancelled | HandoffState::LeaseRestored => {
            return Err(Error::new(Issue::Cancelled));
        }
        HandoffState::Claimed | HandoffState::Ready | HandoffState::Complete => {
            return Err(Error::new(Issue::DuplicateClaimant));
        }
        HandoffState::ReplacementStarted => {}
        HandoffState::Prepared | HandoffState::IngressPaused | HandoffState::LeaseReleased => {
            return Err(Error::new(Issue::InvalidTransition));
        }
    }
    validate_claim_fields(claim)?;
    validate_text(authentication.observation.as_str())?;
    let expected_attempt = current
        .process_attempt
        .as_ref()
        .ok_or_else(invalid_transition)?;
    let expected_process = current.process.as_ref().ok_or_else(invalid_transition)?;
    let expected_endpoint = current.endpoint.as_ref().ok_or_else(invalid_transition)?;
    let expected_generation = current.released_generation.ok_or_else(invalid_transition)?;
    let is_exact = claim.handoff == current.handoff
        && claim.session == current.binding.session
        && claim.released_generation == expected_generation
        && claim.process_attempt == *expected_attempt
        && claim.process == *expected_process
        && claim.endpoint == *expected_endpoint
        && authentication.endpoint == claim.endpoint
        && authentication.process == claim.process;
    if !is_exact {
        return Err(Error::new(Issue::Stale));
    }
    if claim.protocol_version != HANDOFF_PROTOCOL_VERSION {
        return Err(Error::new(Issue::Conflicting));
    }
    if !authentication.authenticated || !authentication.current {
        return Err(Error::new(Issue::Unauthenticated));
    }
    let claim_identity = ClaimIdentity::new(claim_digest(claim, authentication));
    let identity = observation_digest(
        "claimed",
        &[
            claim_identity.as_str(),
            authentication.observation.as_str(),
            bool_label(authentication.authenticated),
            bool_label(authentication.current),
        ],
    );
    let mut next = next_transaction(current, HandoffState::Claimed, identity)?;
    next.claimant = Some(claim.claimant.clone());
    next.claim = Some(claim_identity);
    debug_assert!(next.claim.is_some());
    Ok(next)
}

/// Commits exact replacement readiness after all durable and service checks.
///
/// # Errors
///
/// Returns state drift, stale claim, or invalid-transition before old-worker exit.
// r[impl agent_harness_runtime.daemon_handoff.claim]
// r[impl agent_harness_runtime.daemon_handoff.session_state]
pub fn commit_readiness(
    current: &HandoffTransaction,
    observation: &ReadinessObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    if matches!(
        current.state,
        HandoffState::Cancelled | HandoffState::LeaseRestored
    ) {
        return Err(Error::new(Issue::Cancelled));
    }
    require_state(current, HandoffState::Claimed)?;
    validate_text(observation.services.as_str())?;
    let durable = current
        .durable_state
        .as_ref()
        .ok_or_else(invalid_transition)?;
    let claim = current.claim.as_ref().ok_or_else(invalid_transition)?;
    let is_exact = observation.handoff == current.handoff
        && observation.claim == *claim
        && observation.session_head == durable.session_head
        && observation.composer_draft == durable.composer_draft
        && observation.vault_generation == durable.vault_generation
        && observation.active_turn_generation == durable.active_turn_generation
        && observation.authority_profile == durable.authority_profile
        && observation.policy == durable.policy;
    if !is_exact {
        return Err(Error::new(Issue::SessionStateDrift));
    }
    let is_admitted = readiness_passed(observation.session_load)
        && readiness_passed(observation.vault_visibility)
        && readiness_passed(observation.authority_admission)
        && readiness_passed(observation.recovery_admission)
        && readiness_passed(observation.service_readiness)
        && readiness_passed(observation.readiness_commit);
    if !is_admitted {
        return Err(Error::new(Issue::SessionStateDrift));
    }
    let readiness = ReadinessIdentity::new(readiness_digest(observation));
    let identity = observation_digest(
        "ready",
        &[
            readiness.as_str(),
            observation.services.as_str(),
            readiness_label(observation.readiness_commit),
        ],
    );
    let mut next = next_transaction(current, HandoffState::Ready, identity)?;
    next.readiness = Some(readiness);
    debug_assert!(next.readiness.is_some());
    Ok(next)
}

/// Completes handoff only after exact replacement readiness and old-worker exit.
///
/// # Errors
///
/// Returns stale or invalid-transition without claiming completion.
// r[impl agent_harness_runtime.daemon_handoff.transaction]
pub fn complete_handoff(
    current: &HandoffTransaction,
    observation: &CompletionObservation,
) -> Result<HandoffTransaction, Error> {
    validate_transaction(current)?;
    require_state(current, HandoffState::Ready)?;
    validate_text(observation.observation.as_str())?;
    let readiness = current.readiness.as_ref().ok_or_else(invalid_transition)?;
    if observation.old_worker != current.binding.old_worker || observation.readiness != *readiness {
        return Err(Error::new(Issue::Stale));
    }
    if !observation.exited {
        return Err(Error::new(Issue::Conflicting));
    }
    let identity = observation_digest(
        "complete",
        &[
            observation.old_worker.as_str(),
            observation.readiness.as_str(),
            observation.observation.as_str(),
            bool_label(observation.exited),
        ],
    );
    let next = next_transaction(current, HandoffState::Complete, identity)?;
    debug_assert_eq!(next.state, HandoffState::Complete);
    Ok(next)
}

/// Plans cancellation without terminating a process or restoring a lease.
///
/// # Errors
///
/// Returns invalid-transition for states that do not own a released lease.
// r[impl agent_harness_runtime.daemon_handoff.cancellation]
pub fn plan_cancellation(
    current: &HandoffTransaction,
    cause: CancellationCause,
) -> Result<CancellationPlan, Error> {
    validate_transaction(current)?;
    if !matches!(
        current.state,
        HandoffState::LeaseReleased | HandoffState::ReplacementStarted | HandoffState::Claimed
    ) {
        return Err(Error::new(Issue::InvalidTransition));
    }
    let cancellation = CancellationIdentity::new(digest(
        CANCELLATION_DOMAIN,
        &[
            current.handoff.as_str(),
            current.transition.as_str(),
            current
                .process_attempt
                .as_ref()
                .map_or("none", ProcessAttemptIdentity::as_str),
            cause.as_str(),
        ],
    ));
    debug_assert!(!cancellation.as_str().is_empty());
    Ok(CancellationPlan {
        handoff: current.handoff.clone(),
        prior_transition: current.transition.clone(),
        process_attempt: current.process_attempt.clone(),
        cause,
        cancellation,
    })
}

/// Reduces a durable cancellation commit before any termination request.
///
/// # Errors
///
/// Returns stale or conflicting without recording cancellation.
// r[impl agent_harness_runtime.daemon_handoff.cancellation]
pub fn commit_cancellation(
    current: &HandoffTransaction,
    plan: &CancellationPlan,
    observation: &CancellationCommitObservation,
) -> Result<HandoffTransaction, Error> {
    let expected = plan_cancellation(current, plan.cause)?;
    if expected != *plan
        || observation.cancellation != plan.cancellation
        || observation.cancellation != expected.cancellation
    {
        return Err(Error::new(Issue::Stale));
    }
    validate_text(observation.observation.as_str())?;
    if !observation.committed {
        return Err(Error::new(Issue::Conflicting));
    }
    let identity = observation_digest(
        "cancelled",
        &[
            plan.cancellation.as_str(),
            plan.cause.as_str(),
            observation.observation.as_str(),
            bool_label(observation.committed),
        ],
    );
    let mut next = next_transaction(current, HandoffState::Cancelled, identity)?;
    next.cancellation = Some(plan.cancellation.clone());
    debug_assert!(next.cancellation.is_some());
    Ok(next)
}

/// Restores a later old-worker lease only after complete replacement-tree teardown.
///
/// # Errors
///
/// Returns incomplete teardown, rollback drift, stale generation, or invalid-transition.
// r[impl agent_harness_runtime.daemon_handoff.cancellation]
pub fn restore_lease(
    current: &HandoffTransaction,
    termination: &ProcessTreeTerminationObservation,
    observation: &LeaseRestoreObservation,
) -> Result<HandoffTransaction, Error> {
    validate_lease_restoration_preconditions(current, termination, observation)?;
    if !observation.acquired {
        return Err(Error::new(Issue::LeaseMismatch));
    }
    let identity = observation_digest(
        "lease_restored",
        &[
            termination.termination.as_str(),
            observation.handoff.as_str(),
            observation.old_worker.as_str(),
            observation.authority_profile.as_str(),
            observation.session_head.as_str(),
            &observation.restored_generation.value().to_string(),
            observation.observation.as_str(),
            bool_label(observation.authority_current),
            bool_label(observation.store_head_current),
            bool_label(observation.acquired),
        ],
    );
    let mut next = next_transaction(current, HandoffState::LeaseRestored, identity)?;
    next.restored_generation = Some(observation.restored_generation);
    debug_assert!(next.restored_generation.is_some());
    Ok(next)
}

/// Validates teardown, authority, store head, and next generation before lease mutation.
///
/// # Errors
///
/// Returns incomplete teardown, rollback drift, stale generation, or invalid-transition.
pub fn validate_lease_restoration_preconditions(
    current: &HandoffTransaction,
    termination: &ProcessTreeTerminationObservation,
    observation: &LeaseRestoreObservation,
) -> Result<(), Error> {
    validate_transaction(current)?;
    require_state(current, HandoffState::Cancelled)?;
    validate_termination(current, termination)?;
    validate_text(observation.observation.as_str())?;
    let durable = current
        .durable_state
        .as_ref()
        .ok_or_else(invalid_transition)?;
    let released = current.released_generation.ok_or_else(invalid_transition)?;
    let expected_generation = released
        .value()
        .checked_add(NEXT_GENERATION_INCREMENT)
        .ok_or_else(|| Error::new(Issue::InvalidGeneration))?;
    let is_exact = observation.handoff == current.handoff
        && observation.old_worker == current.binding.old_worker
        && observation.authority_profile == current.binding.authority_profile
        && observation.session_head == durable.session_head
        && observation.restored_generation.value() == expected_generation;
    if !is_exact || !observation.authority_current || !observation.store_head_current {
        return Err(Error::new(Issue::RollbackDrift));
    }
    debug_assert!(is_exact);
    Ok(())
}

/// Maps handoff state to the existing transport-neutral Host Control status shape.
#[must_use]
// r[impl agent_harness_runtime.daemon_handoff.claim]
pub fn host_control_status(state: HandoffState) -> animus_adapters::HostSemanticStatus {
    let status = match state {
        HandoffState::Prepared | HandoffState::IngressPaused => {
            animus_adapters::HostSemanticStatus::Running
        }
        HandoffState::LeaseReleased
        | HandoffState::ReplacementStarted
        | HandoffState::Claimed
        | HandoffState::Ready => animus_adapters::HostSemanticStatus::Waiting,
        HandoffState::Cancelled => animus_adapters::HostSemanticStatus::Failed,
        HandoffState::LeaseRestored => animus_adapters::HostSemanticStatus::Idle,
        HandoffState::Complete => animus_adapters::HostSemanticStatus::Done,
    };
    debug_assert!(matches!(
        status,
        animus_adapters::HostSemanticStatus::Idle
            | animus_adapters::HostSemanticStatus::Running
            | animus_adapters::HostSemanticStatus::Waiting
            | animus_adapters::HostSemanticStatus::Done
            | animus_adapters::HostSemanticStatus::Failed
    ));
    status
}

fn initial_transaction(
    binding: HandoffBinding,
    handoff: HandoffIdentity,
    observation: ObservationIdentity,
) -> HandoffTransaction {
    let sequence = TransitionSequence::new(FIRST_TRANSITION);
    let transition = transition_identity(TransitionFrame {
        handoff: &handoff,
        prior: None,
        state: HandoffState::Prepared,
        sequence,
        observation: &observation,
    });
    debug_assert_eq!(sequence.value(), FIRST_TRANSITION);
    debug_assert!(!transition.as_str().is_empty());
    HandoffTransaction {
        binding,
        handoff,
        state: HandoffState::Prepared,
        sequence,
        prior_transition: None,
        transition,
        observation,
        durable_state: None,
        released_generation: None,
        process_attempt: None,
        process: None,
        endpoint: None,
        claimant: None,
        claim: None,
        readiness: None,
        cancellation: None,
        restored_generation: None,
    }
}

fn next_transaction(
    current: &HandoffTransaction,
    state: HandoffState,
    observation: ObservationIdentity,
) -> Result<HandoffTransaction, Error> {
    let next_sequence = current
        .sequence
        .value()
        .checked_add(NEXT_GENERATION_INCREMENT)
        .ok_or_else(|| Error::new(Issue::InvalidGeneration))?;
    let sequence = TransitionSequence::new(next_sequence);
    let transition = transition_identity(TransitionFrame {
        handoff: &current.handoff,
        prior: Some(&current.transition),
        state,
        sequence,
        observation: &observation,
    });
    debug_assert!(next_sequence > current.sequence.value());
    debug_assert_ne!(transition, current.transition);
    let mut next = current.clone();
    next.state = state;
    next.sequence = sequence;
    next.prior_transition = Some(current.transition.clone());
    next.transition = transition;
    next.observation = observation;
    Ok(next)
}

/// Revalidates one current handoff transaction before it crosses a boundary.
///
/// # Errors
///
/// Returns a bounded identity, shape, or transition-chain issue.
pub fn validate_handoff_transaction(current: &HandoffTransaction) -> Result<(), Error> {
    validate_binding(&current.binding)?;
    validate_hash(current.handoff.as_str())?;
    validate_hash(current.transition.as_str())?;
    validate_hash(current.observation.as_str())?;
    validate_generation(current.sequence.value())?;
    if let Some(prior) = &current.prior_transition {
        validate_hash(prior.as_str())?;
    }
    let expected = transition_identity(TransitionFrame {
        handoff: &current.handoff,
        prior: current.prior_transition.as_ref(),
        state: current.state,
        sequence: current.sequence,
        observation: &current.observation,
    });
    if expected != current.transition {
        return Err(Error::new(Issue::Conflicting));
    }
    validate_state_shape(current)?;
    debug_assert_eq!(expected, current.transition);
    Ok(())
}

fn validate_transaction(current: &HandoffTransaction) -> Result<(), Error> {
    validate_handoff_transaction(current)
}

fn validate_state_shape(current: &HandoffTransaction) -> Result<(), Error> {
    let has_durable = current.durable_state.is_some() && current.released_generation.is_some();
    let has_process = current.process_attempt.is_some()
        && current.process.is_some()
        && current.endpoint.is_some();
    let has_claim = current.claimant.is_some() && current.claim.is_some();
    let is_valid = match current.state {
        HandoffState::Prepared | HandoffState::IngressPaused => {
            !has_durable && !has_process && !has_claim && current.readiness.is_none()
        }
        HandoffState::LeaseReleased => has_durable && !has_process && !has_claim,
        HandoffState::ReplacementStarted => has_durable && has_process && !has_claim,
        HandoffState::Claimed => has_durable && has_process && has_claim,
        HandoffState::Ready | HandoffState::Complete => {
            has_durable && has_process && has_claim && current.readiness.is_some()
        }
        HandoffState::Cancelled | HandoffState::LeaseRestored => {
            has_durable && current.cancellation.is_some()
        }
    };
    if !is_valid {
        return Err(Error::new(Issue::Conflicting));
    }
    if current.state == HandoffState::LeaseRestored && current.restored_generation.is_none() {
        return Err(Error::new(Issue::Conflicting));
    }
    debug_assert!(is_valid);
    Ok(())
}

fn validate_binding(binding: &HandoffBinding) -> Result<(), Error> {
    validate_text(binding.old_worker.as_str())?;
    validate_text(binding.session.as_str())?;
    validate_text(binding.branch.as_str())?;
    validate_text(binding.authority_profile.as_str())?;
    validate_generation(binding.lease_generation.value())?;
    validate_generation(binding.primary_queue_generation.value())?;
    validate_generation(binding.child_queue_generation.value())?;
    validate_generation(binding.steering_generation.value())?;
    validate_generation(binding.active_turn_generation.value())?;
    validate_hash(binding.idle_snapshot.as_str())
}

fn validate_durable_state(
    current: &HandoffTransaction,
    durable: &DurableHandoffState,
) -> Result<(), Error> {
    validate_text(durable.session_head.as_str())?;
    validate_text(durable.composer_draft.as_str())?;
    validate_text(durable.policy.as_str())?;
    validate_text(durable.commit.as_str())?;
    validate_generation(durable.vault_generation.value())?;
    validate_generation(durable.active_turn_generation.value())?;
    if durable.active_turn_generation != current.binding.active_turn_generation
        || durable.authority_profile != current.binding.authority_profile
    {
        return Err(Error::new(Issue::SessionStateDrift));
    }
    Ok(())
}

fn validate_claim_fields(claim: &ReplacementClaim) -> Result<(), Error> {
    validate_hash(claim.handoff.as_str())?;
    validate_text(claim.session.as_str())?;
    validate_generation(claim.released_generation.value())?;
    validate_text(claim.process_attempt.as_str())?;
    validate_text(claim.process.as_str())?;
    validate_text(claim.endpoint.as_str())?;
    validate_text(claim.claimant.as_str())?;
    validate_text(claim.freshness_nonce.as_str())
}

fn validate_termination(
    current: &HandoffTransaction,
    observation: &ProcessTreeTerminationObservation,
) -> Result<(), Error> {
    if observation.process_attempt != current.process_attempt {
        return Err(Error::new(Issue::Stale));
    }
    validate_hash(observation.termination.as_str())?;
    let expected = TerminationIdentity::new(termination_digest(observation));
    if expected != observation.termination {
        return Err(Error::new(Issue::Conflicting));
    }
    let is_complete = observation.leader_terminal
        && observation.descendants_terminal
        && observation.live_descendants == 0
        && observation.terminal_processes == observation.owned_processes;
    if !is_complete {
        return Err(Error::new(Issue::TeardownIncomplete));
    }
    if current.process_attempt.is_some() && observation.owned_processes == 0 {
        return Err(Error::new(Issue::TeardownIncomplete));
    }
    debug_assert!(is_complete);
    Ok(())
}

/// Derives the canonical process-tree termination identity.
#[must_use]
pub fn project_termination_identity(
    observation: &ProcessTreeTerminationObservation,
) -> TerminationIdentity {
    let identity = TerminationIdentity::new(termination_digest(observation));
    debug_assert!(!identity.as_str().is_empty());
    identity
}

#[derive(Clone, Copy)]
struct TransitionFrame<'a> {
    handoff: &'a HandoffIdentity,
    prior: Option<&'a TransitionIdentity>,
    state: HandoffState,
    sequence: TransitionSequence,
    observation: &'a ObservationIdentity,
}

fn transition_identity(frame: TransitionFrame<'_>) -> TransitionIdentity {
    TransitionIdentity::new(digest(
        TRANSITION_DOMAIN,
        &[
            frame.handoff.as_str(),
            frame.prior.map_or("none", TransitionIdentity::as_str),
            frame.state.as_str(),
            &frame.sequence.value().to_string(),
            frame.observation.as_str(),
        ],
    ))
}

fn strict_idle_digest(snapshot: &StrictIdleSnapshot) -> String {
    digest(
        IDLE_SNAPSHOT_DOMAIN,
        &[
            snapshot.worker.as_str(),
            snapshot.session.as_str(),
            snapshot.branch.as_str(),
            &snapshot.lease_generation.value().to_string(),
            &snapshot.primary_queue_generation.value().to_string(),
            &snapshot.child_queue_generation.value().to_string(),
            &snapshot.steering_generation.value().to_string(),
            &snapshot.active_turn_generation.value().to_string(),
            snapshot.authority_profile.as_str(),
            activity_label(snapshot.provider),
            activity_label(snapshot.tool),
            activity_label(snapshot.repair),
            activity_label(snapshot.confirmation),
            activity_label(snapshot.cancellation),
            activity_label(snapshot.durable_commit),
            activity_label(snapshot.terminal_transition),
            &snapshot.primary_queue_depth.to_string(),
            &snapshot.child_queue_depth.to_string(),
            &snapshot.steering_depth.to_string(),
        ],
    )
}

fn claim_digest(claim: &ReplacementClaim, authentication: &AuthenticationObservation) -> String {
    digest(
        CLAIM_DOMAIN,
        &[
            claim.handoff.as_str(),
            claim.session.as_str(),
            &claim.released_generation.value().to_string(),
            claim.process_attempt.as_str(),
            claim.process.as_str(),
            claim.endpoint.as_str(),
            claim.claimant.as_str(),
            &claim.protocol_version.to_string(),
            claim.freshness_nonce.as_str(),
            authentication.observation.as_str(),
            bool_label(authentication.authenticated),
            bool_label(authentication.current),
        ],
    )
}

fn readiness_digest(observation: &ReadinessObservation) -> String {
    digest(
        READINESS_DOMAIN,
        &[
            observation.handoff.as_str(),
            observation.claim.as_str(),
            observation.session_head.as_str(),
            observation.composer_draft.as_str(),
            &observation.vault_generation.value().to_string(),
            &observation.active_turn_generation.value().to_string(),
            observation.authority_profile.as_str(),
            observation.policy.as_str(),
            observation.services.as_str(),
            readiness_label(observation.session_load),
            readiness_label(observation.vault_visibility),
            readiness_label(observation.authority_admission),
            readiness_label(observation.recovery_admission),
            readiness_label(observation.service_readiness),
            readiness_label(observation.readiness_commit),
        ],
    )
}

fn termination_digest(observation: &ProcessTreeTerminationObservation) -> String {
    digest(
        TERMINATION_DOMAIN,
        &[
            observation
                .process_attempt
                .as_ref()
                .map_or("none", ProcessAttemptIdentity::as_str),
            bool_label(observation.leader_terminal),
            bool_label(observation.descendants_terminal),
            &observation.owned_processes.to_string(),
            &observation.terminal_processes.to_string(),
            &observation.live_descendants.to_string(),
        ],
    )
}

fn observation_digest(kind: &str, fields: &[&str]) -> ObservationIdentity {
    let mut values = Vec::with_capacity(fields.len().saturating_add(1));
    values.push(kind);
    values.extend_from_slice(fields);
    ObservationIdentity::new(digest(OBSERVATION_DOMAIN, &values))
}

fn digest(domain: &str, fields: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain.as_bytes());
    for field in fields {
        update_frame(&mut hasher, field.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn update_frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    let Ok(length_bytes) = u64::try_from(value.len()) else {
        hasher.update(FRAME_LENGTH_OVERFLOW_TAG);
        return;
    };
    let encoded = length_bytes.to_le_bytes();
    assert_eq!(encoded.len(), FRAME_LENGTH_BYTES);
    hasher.update(&encoded);
    hasher.update(value);
}

fn require_state(current: &HandoffTransaction, expected: HandoffState) -> Result<(), Error> {
    if current.state == expected {
        return Ok(());
    }
    Err(Error::new(Issue::InvalidTransition))
}

fn require_idle(state: ActivityState, class: WorkClass) -> Result<(), Error> {
    if state == ActivityState::Idle {
        return Ok(());
    }
    Err(Error::new(Issue::Busy(class)))
}

fn require_zero(value: u32, class: WorkClass) -> Result<(), Error> {
    if value == 0 {
        return Ok(());
    }
    Err(Error::new(Issue::Busy(class)))
}

fn validate_generation(value: u64) -> Result<(), Error> {
    if value >= MIN_GENERATION {
        return Ok(());
    }
    Err(Error::new(Issue::InvalidGeneration))
}

fn validate_hash(value: &str) -> Result<(), Error> {
    if value.len() == BLAKE3_HEX_BYTES && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(Error::new(Issue::InvalidIdentity))
}

fn validate_text(value: &str) -> Result<(), Error> {
    let is_valid = !value.is_empty()
        && value.len() <= MAX_IDENTITY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'));
    if is_valid {
        return Ok(());
    }
    Err(Error::new(Issue::InvalidIdentity))
}

const fn invalid_transition() -> Error {
    Error::new(Issue::InvalidTransition)
}

const fn activity_label(state: ActivityState) -> &'static str {
    match state {
        ActivityState::Idle => "idle",
        ActivityState::Active => "active",
        ActivityState::Ambiguous => "ambiguous",
    }
}

const fn readiness_passed(state: ReadinessCheck) -> bool {
    matches!(state, ReadinessCheck::Passed)
}

const fn readiness_label(state: ReadinessCheck) -> &'static str {
    match state {
        ReadinessCheck::Passed => "passed",
        ReadinessCheck::Failed => "failed",
    }
}

const fn bool_label(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
