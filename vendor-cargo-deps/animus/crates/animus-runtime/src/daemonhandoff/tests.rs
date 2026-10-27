use super::*;

const BASE_GENERATION: u64 = 7;
const NEXT_GENERATION: u64 = BASE_GENERATION.saturating_add(1);
const QUEUE_GENERATION: u64 = 3;
const ACTIVE_TURN_GENERATION: u64 = 9;
const VAULT_GENERATION: u64 = 11;
const ONE_ITEM: u32 = 1;
const BUSY_WORK_CLASS_COUNT: usize = 10;
const ONE_PROCESS: u32 = 1;
const ZERO_ITEMS: u32 = 0;

fn idle_snapshot() -> StrictIdleSnapshot {
    StrictIdleSnapshot {
        worker: WorkerIdentity::new("worker.old"),
        session: SessionIdentity::new("session.alpha"),
        branch: BranchIdentity::new("branch.main"),
        lease_generation: LeaseGeneration::new(BASE_GENERATION),
        primary_queue_generation: QueueGeneration::new(QUEUE_GENERATION),
        child_queue_generation: QueueGeneration::new(QUEUE_GENERATION),
        steering_generation: QueueGeneration::new(QUEUE_GENERATION),
        active_turn_generation: ActiveTurnGeneration::new(ACTIVE_TURN_GENERATION),
        authority_profile: AuthorityProfileIdentity::new("authority.current"),
        provider: ActivityState::Idle,
        tool: ActivityState::Idle,
        repair: ActivityState::Idle,
        confirmation: ActivityState::Idle,
        cancellation: ActivityState::Idle,
        durable_commit: ActivityState::Idle,
        terminal_transition: ActivityState::Idle,
        primary_queue_depth: ZERO_ITEMS,
        child_queue_depth: ZERO_ITEMS,
        steering_depth: ZERO_ITEMS,
    }
}

fn prepared() -> HandoffTransaction {
    let idle = validate_strict_idle(&idle_snapshot()).expect("strict idle");
    prepare_handoff(&idle).expect("prepared handoff")
}

fn paused() -> HandoffTransaction {
    let prepared = prepared();
    pause_ingress(
        &prepared,
        &IngressPauseObservation {
            handoff: prepared.handoff.clone(),
            primary_queue_generation: prepared.binding.primary_queue_generation,
            child_queue_generation: prepared.binding.child_queue_generation,
            steering_generation: prepared.binding.steering_generation,
            all_ingress_paused: true,
        },
    )
    .expect("paused ingress")
}

fn durable_state() -> DurableHandoffState {
    DurableHandoffState {
        session_head: SessionHeadIdentity::new("session.head.current"),
        composer_draft: ComposerDraftIdentity::new("draft.identity.only"),
        vault_generation: VaultGeneration::new(VAULT_GENERATION),
        active_turn_generation: ActiveTurnGeneration::new(ACTIVE_TURN_GENERATION),
        authority_profile: AuthorityProfileIdentity::new("authority.current"),
        policy: PolicyIdentity::new("policy.handoff.current"),
        commit: CommitIdentity::new("commit.handoff.current"),
    }
}

fn released() -> HandoffTransaction {
    let paused = paused();
    let durable = durable_state();
    release_lease(
        &paused,
        &durable,
        &LeaseReleaseObservation {
            handoff: paused.handoff.clone(),
            released_generation: LeaseGeneration::new(BASE_GENERATION),
            durable_commit: durable.commit.clone(),
            released: true,
        },
    )
    .expect("released lease")
}

fn started() -> HandoffTransaction {
    let released = released();
    start_replacement(
        &released,
        &ProcessStartObservation {
            handoff: released.handoff.clone(),
            process_attempt: ProcessAttemptIdentity::new("process.attempt.one"),
            process: ProcessIdentity::new("process.replacement.one"),
            endpoint: EndpointIdentity::new("endpoint.local.one"),
            started: true,
        },
    )
    .expect("replacement started")
}

fn claim_for(transaction: &HandoffTransaction) -> ReplacementClaim {
    ReplacementClaim {
        handoff: transaction.handoff.clone(),
        session: transaction.binding.session.clone(),
        released_generation: transaction
            .released_generation
            .expect("released generation"),
        process_attempt: transaction
            .process_attempt
            .clone()
            .expect("process attempt"),
        process: transaction.process.clone().expect("process"),
        endpoint: transaction.endpoint.clone().expect("endpoint"),
        claimant: ClaimantIdentity::new("claimant.replacement.one"),
        protocol_version: HANDOFF_PROTOCOL_VERSION,
        freshness_nonce: FreshnessNonce::new("nonce.handoff.one"),
    }
}

fn authentication_for(claim: &ReplacementClaim) -> AuthenticationObservation {
    AuthenticationObservation {
        endpoint: claim.endpoint.clone(),
        process: claim.process.clone(),
        authenticated: true,
        current: true,
        observation: ObservationIdentity::new("authentication.current"),
    }
}

fn claimed() -> HandoffTransaction {
    let started = started();
    let claim = claim_for(&started);
    admit_claim(&started, &claim, &authentication_for(&claim)).expect("claim admitted")
}

fn ready() -> HandoffTransaction {
    let claimed = claimed();
    let durable = claimed.durable_state.as_ref().expect("durable state");
    let ready = commit_readiness(
        &claimed,
        &ReadinessObservation {
            handoff: claimed.handoff.clone(),
            claim: claimed.claim.clone().expect("claim identity"),
            session_head: durable.session_head.clone(),
            composer_draft: durable.composer_draft.clone(),
            vault_generation: durable.vault_generation,
            active_turn_generation: durable.active_turn_generation,
            authority_profile: durable.authority_profile.clone(),
            policy: durable.policy.clone(),
            services: ServiceReadinessIdentity::new("services.replacement.ready"),
            session_load: ReadinessCheck::Passed,
            vault_visibility: ReadinessCheck::Passed,
            authority_admission: ReadinessCheck::Passed,
            recovery_admission: ReadinessCheck::Passed,
            service_readiness: ReadinessCheck::Passed,
            readiness_commit: ReadinessCheck::Passed,
        },
    )
    .expect("ready");
    debug_assert_eq!(ready.state, HandoffState::Ready);
    ready
}

fn cancellation_commit(transaction: &HandoffTransaction) -> HandoffTransaction {
    let plan = plan_cancellation(transaction, CancellationCause::Deadline).expect("cancel plan");
    commit_cancellation(
        transaction,
        &plan,
        &CancellationCommitObservation {
            cancellation: plan.cancellation.clone(),
            committed: true,
            observation: ObservationIdentity::new("cancel.commit.current"),
        },
    )
    .expect("cancelled")
}

fn complete_termination(transaction: &HandoffTransaction) -> ProcessTreeTerminationObservation {
    let mut observation = ProcessTreeTerminationObservation {
        process_attempt: transaction.process_attempt.clone(),
        leader_terminal: true,
        descendants_terminal: true,
        owned_processes: ONE_PROCESS,
        terminal_processes: ONE_PROCESS,
        live_descendants: ZERO_ITEMS,
        termination: TerminationIdentity::new("placeholder"),
    };
    observation.termination = project_termination_identity(&observation);
    observation
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.idle.valid]
// r[verify agent_harness_runtime.daemon_handoff.transaction.success]
// r[verify agent_harness_runtime.daemon_handoff.claim.allowed]
// r[verify agent_harness_runtime.daemon_handoff.session_state.ready]
fn exact_idle_to_ready_handoff_completes_in_order() {
    let ready = ready();
    assert_eq!(ready.state, HandoffState::Ready);
    assert!(ready.claim.is_some());
    assert!(ready.readiness.is_some());
    assert_eq!(
        host_control_status(ready.state),
        animus_adapters::HostSemanticStatus::Waiting
    );

    let completed = complete_handoff(
        &ready,
        &CompletionObservation {
            old_worker: ready.binding.old_worker.clone(),
            readiness: ready.readiness.clone().expect("readiness"),
            exited: true,
            observation: ObservationIdentity::new("old.worker.exited"),
        },
    )
    .expect("complete");
    assert_eq!(completed.state, HandoffState::Complete);
    assert_eq!(
        host_control_status(completed.state),
        animus_adapters::HostSemanticStatus::Done
    );
    validate_handoff_transaction(&completed).expect("valid final chain");
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.idle.busy]
fn every_busy_or_ambiguous_work_class_denies_before_pause() {
    let mut fixtures = Vec::new();

    let mut provider = idle_snapshot();
    provider.provider = ActivityState::Active;
    fixtures.push((provider, WorkClass::Provider));
    let mut tool = idle_snapshot();
    tool.tool = ActivityState::Ambiguous;
    fixtures.push((tool, WorkClass::Tool));
    let mut repair = idle_snapshot();
    repair.repair = ActivityState::Active;
    fixtures.push((repair, WorkClass::Repair));
    let mut confirmation = idle_snapshot();
    confirmation.confirmation = ActivityState::Active;
    fixtures.push((confirmation, WorkClass::Confirmation));
    let mut cancellation = idle_snapshot();
    cancellation.cancellation = ActivityState::Ambiguous;
    fixtures.push((cancellation, WorkClass::Cancellation));
    let mut commit = idle_snapshot();
    commit.durable_commit = ActivityState::Active;
    fixtures.push((commit, WorkClass::DurableCommit));
    let mut terminal = idle_snapshot();
    terminal.terminal_transition = ActivityState::Ambiguous;
    fixtures.push((terminal, WorkClass::TerminalTransition));
    let mut primary = idle_snapshot();
    primary.primary_queue_depth = ONE_ITEM;
    fixtures.push((primary, WorkClass::PrimaryQueue));
    let mut child = idle_snapshot();
    child.child_queue_depth = ONE_ITEM;
    fixtures.push((child, WorkClass::ChildQueue));
    let mut steering = idle_snapshot();
    steering.steering_depth = ONE_ITEM;
    fixtures.push((steering, WorkClass::Steering));

    assert_eq!(fixtures.len(), BUSY_WORK_CLASS_COUNT);
    for (snapshot, expected) in fixtures {
        assert_eq!(
            validate_strict_idle(&snapshot)
                .expect_err("busy class")
                .issue(),
            Issue::Busy(expected)
        );
    }
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.transaction.invalid]
fn skipped_replayed_and_tampered_transitions_fail_closed() {
    let prepared = prepared();
    let process = ProcessStartObservation {
        handoff: prepared.handoff.clone(),
        process_attempt: ProcessAttemptIdentity::new("attempt.early"),
        process: ProcessIdentity::new("process.early"),
        endpoint: EndpointIdentity::new("endpoint.early"),
        started: true,
    };
    assert_eq!(
        start_replacement(&prepared, &process)
            .expect_err("skip pause and release")
            .issue(),
        Issue::InvalidTransition
    );

    let paused = paused();
    let replay = IngressPauseObservation {
        handoff: paused.handoff.clone(),
        primary_queue_generation: paused.binding.primary_queue_generation,
        child_queue_generation: paused.binding.child_queue_generation,
        steering_generation: paused.binding.steering_generation,
        all_ingress_paused: true,
    };
    assert_eq!(
        pause_ingress(&paused, &replay)
            .expect_err("non-idempotent replay")
            .issue(),
        Issue::InvalidTransition
    );

    let mut forged = paused;
    forged.sequence = TransitionSequence::new(BASE_GENERATION);
    assert_eq!(
        validate_handoff_transaction(&forged)
            .expect_err("tampered sequence")
            .issue(),
        Issue::Conflicting
    );
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.claim.denied]
fn stale_process_generation_endpoint_and_protocol_claims_are_denied() {
    let started = started();
    let mut wrong_endpoint = claim_for(&started);
    wrong_endpoint.endpoint = EndpointIdentity::new("endpoint.wrong");
    assert_eq!(
        admit_claim(
            &started,
            &wrong_endpoint,
            &authentication_for(&wrong_endpoint)
        )
        .expect_err("wrong endpoint")
        .issue(),
        Issue::Stale
    );

    let mut wrong_process = claim_for(&started);
    wrong_process.process = ProcessIdentity::new("process.wrong");
    assert_eq!(
        admit_claim(
            &started,
            &wrong_process,
            &authentication_for(&wrong_process)
        )
        .expect_err("wrong process")
        .issue(),
        Issue::Stale
    );

    let mut stale_generation = claim_for(&started);
    stale_generation.released_generation = LeaseGeneration::new(NEXT_GENERATION);
    assert_eq!(
        admit_claim(
            &started,
            &stale_generation,
            &authentication_for(&stale_generation)
        )
        .expect_err("stale generation")
        .issue(),
        Issue::Stale
    );

    let mut wrong_protocol = claim_for(&started);
    wrong_protocol.protocol_version = HANDOFF_PROTOCOL_VERSION.saturating_add(1);
    assert_eq!(
        admit_claim(
            &started,
            &wrong_protocol,
            &authentication_for(&wrong_protocol)
        )
        .expect_err("wrong protocol")
        .issue(),
        Issue::Conflicting
    );
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.claim.denied]
// r[verify agent_harness_runtime.daemon_handoff.cancellation.late_claim]
fn unauthenticated_duplicate_and_late_claims_are_denied() {
    let started = started();
    let claim = claim_for(&started);
    let mut denied = authentication_for(&claim);
    denied.authenticated = false;
    assert_eq!(
        admit_claim(&started, &claim, &denied)
            .expect_err("unauthenticated")
            .issue(),
        Issue::Unauthenticated
    );
    denied.authenticated = true;
    denied.current = false;
    assert_eq!(
        admit_claim(&started, &claim, &denied)
            .expect_err("stale authentication")
            .issue(),
        Issue::Unauthenticated
    );

    let claimed = admit_claim(&started, &claim, &authentication_for(&claim)).expect("claim");
    assert_eq!(
        admit_claim(&claimed, &claim, &authentication_for(&claim))
            .expect_err("duplicate")
            .issue(),
        Issue::DuplicateClaimant
    );

    let cancelled = cancellation_commit(&started);
    assert_eq!(
        admit_claim(&cancelled, &claim, &authentication_for(&claim))
            .expect_err("late claim")
            .issue(),
        Issue::Cancelled
    );
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.cancellation.timeout]
// r[verify agent_harness_runtime.daemon_handoff.cancellation.survivor]
// r[verify agent_harness_runtime.daemon_handoff.verification.incomplete]
fn cancellation_requires_complete_tree_before_next_lease_generation() {
    let cancelled = cancellation_commit(&started());
    let mut incomplete = complete_termination(&cancelled);
    incomplete.descendants_terminal = false;
    incomplete.live_descendants = ONE_ITEM;
    incomplete.termination = project_termination_identity(&incomplete);
    let restore = LeaseRestoreObservation {
        handoff: cancelled.handoff.clone(),
        old_worker: cancelled.binding.old_worker.clone(),
        authority_profile: cancelled.binding.authority_profile.clone(),
        session_head: cancelled
            .durable_state
            .as_ref()
            .expect("durable")
            .session_head
            .clone(),
        restored_generation: LeaseGeneration::new(NEXT_GENERATION),
        authority_current: true,
        store_head_current: true,
        acquired: true,
        observation: ObservationIdentity::new("lease.restore.current"),
    };
    assert_eq!(
        restore_lease(&cancelled, &incomplete, &restore)
            .expect_err("surviving descendant")
            .issue(),
        Issue::TeardownIncomplete
    );

    let restored = restore_lease(&cancelled, &complete_termination(&cancelled), &restore)
        .expect("restored after teardown");
    assert_eq!(restored.state, HandoffState::LeaseRestored);
    assert_eq!(
        restored.restored_generation,
        Some(LeaseGeneration::new(NEXT_GENERATION))
    );
    assert_eq!(
        host_control_status(restored.state),
        animus_adapters::HostSemanticStatus::Idle
    );
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.session_state.drift]
fn readiness_and_rollback_drift_fail_closed() {
    let claimed = claimed();
    let durable = claimed.durable_state.as_ref().expect("durable");
    let drift = ReadinessObservation {
        handoff: claimed.handoff.clone(),
        claim: claimed.claim.clone().expect("claim"),
        session_head: SessionHeadIdentity::new("session.head.stale"),
        composer_draft: durable.composer_draft.clone(),
        vault_generation: durable.vault_generation,
        active_turn_generation: durable.active_turn_generation,
        authority_profile: durable.authority_profile.clone(),
        policy: durable.policy.clone(),
        services: ServiceReadinessIdentity::new("services.ready"),
        session_load: ReadinessCheck::Passed,
        vault_visibility: ReadinessCheck::Passed,
        authority_admission: ReadinessCheck::Passed,
        recovery_admission: ReadinessCheck::Passed,
        service_readiness: ReadinessCheck::Passed,
        readiness_commit: ReadinessCheck::Passed,
    };
    assert_eq!(
        commit_readiness(&claimed, &drift)
            .expect_err("readiness drift")
            .issue(),
        Issue::SessionStateDrift
    );

    let cancelled = cancellation_commit(&started());
    let mut restore = LeaseRestoreObservation {
        handoff: cancelled.handoff.clone(),
        old_worker: cancelled.binding.old_worker.clone(),
        authority_profile: cancelled.binding.authority_profile.clone(),
        session_head: cancelled
            .durable_state
            .as_ref()
            .expect("durable")
            .session_head
            .clone(),
        restored_generation: LeaseGeneration::new(NEXT_GENERATION),
        authority_current: true,
        store_head_current: false,
        acquired: true,
        observation: ObservationIdentity::new("restore.stale"),
    };
    assert_eq!(
        restore_lease(&cancelled, &complete_termination(&cancelled), &restore)
            .expect_err("store drift")
            .issue(),
        Issue::RollbackDrift
    );
    restore.store_head_current = true;
    restore.restored_generation = LeaseGeneration::new(BASE_GENERATION);
    assert_eq!(
        restore_lease(&cancelled, &complete_termination(&cancelled), &restore)
            .expect_err("generation reuse")
            .issue(),
        Issue::RollbackDrift
    );
}

#[test]
// r[verify agent_harness_runtime.daemon_handoff.evidence.safe]
// r[verify agent_harness_runtime.daemon_handoff.evidence.rejects]
fn receipts_are_deterministic_metadata_only_and_reject_leaks() {
    let transaction = ready();
    let first =
        project_handoff_receipt(&transaction, SensitiveEvidence::default()).expect("safe receipt");
    let second = project_handoff_receipt(&transaction, SensitiveEvidence::default())
        .expect("stable receipt");
    assert_eq!(first, second);
    validate_handoff_receipt(&transaction, &first).expect("valid receipt");
    assert_eq!(
        first.vault_generation,
        Some(VaultGeneration::new(VAULT_GENERATION))
    );

    let leaks = [
        SensitiveEvidence::ComposerText,
        SensitiveEvidence::PendingInput,
        SensitiveEvidence::PromptOrTranscript,
        SensitiveEvidence::Credentials,
        SensitiveEvidence::SocketPath,
        SensitiveEvidence::ProcessArguments,
        SensitiveEvidence::ProviderData,
        SensitiveEvidence::TerminalData,
        SensitiveEvidence::RawError,
    ];
    for leak in leaks {
        assert_eq!(
            project_handoff_receipt(&transaction, leak)
                .expect_err("receipt leak")
                .issue(),
            Issue::ReceiptLeak
        );
    }

    let mut overclaim = first;
    overclaim.non_claims.pop();
    assert_eq!(
        validate_handoff_receipt(&transaction, &overclaim)
            .expect_err("missing non-claim")
            .issue(),
        Issue::ReceiptOverclaim
    );
}
