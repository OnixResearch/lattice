use super::*;

const GENERATION_VALUE: u64 = 7;
const STALE_GENERATION_VALUE: u64 = 6;
const MAX_ENTRIES_VALUE: u32 = 16;
const MAX_ENTRY_BYTES_VALUE: u32 = 1_024;
const OVER_LIMIT_ENTRY_BYTES_VALUE: u32 = 1_025;
const MAX_TOTAL_BYTES_VALUE: u64 = 8_192;
const MAX_ATTACHMENTS_VALUE: u32 = 4;
const ENTRY_BYTES_VALUE: u32 = 64;
const FIRST_SEQUENCE_VALUE: u64 = 1;
const SECOND_SEQUENCE_VALUE: u64 = 2;
const THIRD_SEQUENCE_VALUE: u64 = 3;
const FOURTH_SEQUENCE_VALUE: u64 = 4;
const FIFTH_SEQUENCE_VALUE: u64 = 5;
const SIXTH_SEQUENCE_VALUE: u64 = 6;
const EXPECTED_CLASS_COUNT: usize = 6;
const EXPECTED_CLASS_COUNT_U32: u32 = 6;
const SELECTED_COUNT: usize = 2;
const ZERO_CALLS: u32 = 0;
const ONE_CALL: u32 = 1;
const TWO_CALLS: u32 = 2;
const BODY_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_BODY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ATTACHMENT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const LOADED_BODY: &str = "body available only after current admission";

fn policy() -> Policy {
    Policy {
        entries: MAX_ENTRIES_VALUE,
        entry_bytes: ByteCount::new(MAX_ENTRY_BYTES_VALUE),
        total_bytes: MAX_TOTAL_BYTES_VALUE,
        attachments_per_entry: MAX_ATTACHMENTS_VALUE,
    }
}

fn boundary() -> Boundary {
    Boundary {
        id: Identity::new("recovery.primary"),
        conversation: ConversationIdentity::new("conversation.alpha"),
        branch: BranchIdentity::new("branch.main"),
        store: StoreIdentity::new("store.explicit"),
        authority: AuthorityIdentity::new("authority.current"),
        policy: PolicyIdentity::new("policy.recovery"),
        generation: Generation::new(GENERATION_VALUE),
    }
}

fn entry(sequence: u64, class: EntryClass) -> Entry {
    debug_assert!(sequence > 0);
    debug_assert!(!BODY_HASH.is_empty());
    let body_identity = if sequence == SECOND_SEQUENCE_VALUE {
        BodyIdentity::new(SECOND_BODY_HASH)
    } else {
        BodyIdentity::new(BODY_HASH)
    };
    let attachments = if class == EntryClass::AttachmentReference {
        vec![AttachmentIdentity::new(ATTACHMENT_HASH)]
    } else {
        Vec::new()
    };
    Entry {
        id: EntryIdentity::new(format!("entry.{sequence}")),
        conversation: boundary().conversation,
        generation: Generation::new(GENERATION_VALUE),
        sequence: Sequence::new(sequence),
        class,
        body_identity,
        body_bytes: ByteCount::new(ENTRY_BYTES_VALUE),
        attachments,
        provenance: ProvenanceIdentity::new(format!("route.{sequence}")),
    }
}

fn entries() -> Vec<Entry> {
    vec![
        entry(FIRST_SEQUENCE_VALUE, EntryClass::ActiveAccepted),
        entry(SECOND_SEQUENCE_VALUE, EntryClass::SteeringInFlight),
        entry(THIRD_SEQUENCE_VALUE, EntryClass::SteeringQueued),
        entry(FOURTH_SEQUENCE_VALUE, EntryClass::FollowUp),
        entry(FIFTH_SEQUENCE_VALUE, EntryClass::Command),
        entry(SIXTH_SEQUENCE_VALUE, EntryClass::AttachmentReference),
    ]
}

fn facts() -> StartupFacts {
    StartupFacts {
        boundary: boundary(),
        entries: entries(),
        completed: None,
        cleanup: CleanupState::NotRequired,
        uncertain_commit: false,
    }
}

fn projection() -> Projection {
    project_startup(&policy(), &facts()).expect("valid recovery projection")
}

fn disposition_request(projection: &Projection) -> DispositionRequest {
    DispositionRequest {
        id: DispositionIdentity::new("disposition.restore-one"),
        projection: projection.identity.clone(),
        boundary: projection.boundary.clone(),
        expected_generation: projection.boundary.generation,
        selected: vec![EntryIdentity::new("entry.1"), EntryIdentity::new("entry.3")],
        kind: DispositionKind::Restore,
        authorized: true,
    }
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.contents.ordered]
// r[verify agent_harness_runtime.pending_input_vault.recovery.review]
// r[verify agent_harness_runtime.pending_input_vault.verification]
// r[verify agent_harness_runtime.pending_input_vault.verification.complete]
fn mixed_classes_keep_exact_order_and_require_review() {
    let projected = projection();
    assert_eq!(projected.state, RecoveryState::ReviewRequired);
    assert!(!ordinary_ingress_allowed(&projected.state));
    assert_eq!(projected.entries.len(), EXPECTED_CLASS_COUNT);
    assert_eq!(projected.entries[0].class, EntryClass::ActiveAccepted);
    assert_eq!(projected.entries[1].class, EntryClass::SteeringInFlight);
    assert_eq!(projected.entries[2].class, EntryClass::SteeringQueued);
    assert_eq!(projected.entries[3].class, EntryClass::FollowUp);
    assert_eq!(projected.entries[4].class, EntryClass::Command);
    assert_eq!(projected.entries[5].class, EntryClass::AttachmentReference);
    let inspection = inspect_projection(&projected).expect("inspection");
    assert_eq!(inspection.entries.len(), EXPECTED_CLASS_COUNT);
    assert_eq!(inspection.entries[0].sequence.value(), FIRST_SEQUENCE_VALUE);
    assert_eq!(inspection.entries[5].attachment_count, ONE_CALL);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.verification.complete]
fn empty_state_allows_ingress_and_discard_emits_no_admission() {
    let mut empty = facts();
    empty.entries.clear();
    let no_vault = project_startup(&policy(), &empty).expect("empty projection");
    assert_eq!(no_vault.state, RecoveryState::NoVault);
    assert!(no_vault.entries.is_empty());
    assert!(ordinary_ingress_allowed(&no_vault.state));
    assert!(inspect_projection(&no_vault).is_err());

    let projected = projection();
    let mut request = disposition_request(&projected);
    request.id = DispositionIdentity::new("disposition.discard-two");
    request.kind = DispositionKind::Discard;
    let DispositionDecision::Planned(plan) =
        plan_disposition(&projected, &request, None).expect("discard plan")
    else {
        panic!("discard must produce a plan");
    };
    let CommitOutcome::Completed(completed) = reduce_commit(
        &projected,
        &plan,
        &CommitObservation {
            plan_identity: plan.plan_identity.clone(),
            generation: plan.expected_generation,
            status: CommitStatus::Committed,
            restore_material_retained: false,
        },
    )
    .expect("discard commit") else {
        panic!("discard must complete");
    };
    assert_eq!(completed.kind, DispositionKind::Discard);
    assert_eq!(completed.selected.len(), SELECTED_COUNT);
    assert!(completed.admission_requests.is_empty());
    assert_eq!(completed.cleanup, CleanupState::Required);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.contents.conflict]
fn malformed_entry_facts_fail_before_projection() {
    let current_boundary = boundary();
    let mut draft = EntryDraft {
        id: EntryIdentity::new("entry.valid"),
        conversation: current_boundary.conversation.clone(),
        generation: current_boundary.generation,
        sequence: Sequence::new(FIRST_SEQUENCE_VALUE),
        class: EntryClass::ActiveAccepted,
        body_identity: BodyIdentity::new(BODY_HASH),
        body_bytes: ByteCount::new(ENTRY_BYTES_VALUE),
        attachments: Vec::new(),
        provenance: ProvenanceIdentity::new("route.valid"),
    };
    assert!(validate_entry(&policy(), &current_boundary, draft.clone()).is_ok());
    draft.body_identity = BodyIdentity::new("not-a-hash");
    assert_eq!(
        validate_entry(&policy(), &current_boundary, draft.clone())
            .expect_err("hash must fail")
            .issue(),
        Issue::InvalidBodyIdentity
    );
    draft.body_identity = BodyIdentity::new(BODY_HASH);
    draft.conversation = ConversationIdentity::new("conversation.other");
    assert_eq!(
        validate_entry(&policy(), &current_boundary, draft)
            .expect_err("scope must fail")
            .issue(),
        Issue::CrossConversation
    );
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.corruption.blocked]
fn every_corruption_class_blocks_without_partial_suffix() {
    let mut fixtures = Vec::new();

    let mut duplicate = facts();
    duplicate.entries[1].id = duplicate.entries[0].id.clone();
    fixtures.push((duplicate, CorruptionClass::Duplicate));

    let mut missing = facts();
    missing.entries[1].sequence = Sequence::new(THIRD_SEQUENCE_VALUE);
    fixtures.push((missing, CorruptionClass::MissingSequence));

    let mut reordered = facts();
    reordered.entries[1].sequence = Sequence::new(FIRST_SEQUENCE_VALUE);
    fixtures.push((reordered, CorruptionClass::Reordered));

    let mut mixed = facts();
    mixed.entries[0].generation = Generation::new(STALE_GENERATION_VALUE);
    fixtures.push((mixed, CorruptionClass::MixedGeneration));

    let mut cross = facts();
    cross.entries[0].conversation = ConversationIdentity::new("conversation.other");
    fixtures.push((cross, CorruptionClass::CrossConversation));

    let mut over_limit_bytes = facts();
    over_limit_bytes.entries[0].body_bytes = ByteCount::new(OVER_LIMIT_ENTRY_BYTES_VALUE);
    fixtures.push((over_limit_bytes, CorruptionClass::OverBudget));

    let mut malformed = facts();
    malformed.entries[0].body_identity = BodyIdentity::new("bad");
    fixtures.push((malformed, CorruptionClass::Malformed));

    let mut uncertain = facts();
    uncertain.uncertain_commit = true;
    fixtures.push((uncertain, CorruptionClass::UncertainCommit));

    for (fixture, expected) in fixtures {
        let projected = project_startup(&policy(), &fixture).expect("blocked projection");
        assert_eq!(projected.state, RecoveryState::BlockedCorrupt(expected));
        assert!(projected.entries.is_empty());
        assert!(!ordinary_ingress_allowed(&projected.state));
    }
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.corruption.blocked]
fn blocked_identity_binds_exact_hidden_corruption_facts() {
    let mut first = facts();
    first.entries[0].body_identity = BodyIdentity::new("bad-first");
    let mut second = facts();
    second.entries[0].body_identity = BodyIdentity::new("bad-second");
    let first_projection = project_startup(&policy(), &first).expect("first blocked projection");
    let second_projection = project_startup(&policy(), &second).expect("second blocked projection");
    assert_eq!(
        first_projection.state,
        RecoveryState::BlockedCorrupt(CorruptionClass::Malformed)
    );
    assert_eq!(
        second_projection.state,
        RecoveryState::BlockedCorrupt(CorruptionClass::Malformed)
    );
    assert!(first_projection.entries.is_empty());
    assert!(second_projection.entries.is_empty());
    assert_ne!(first_projection.identity, second_projection.identity);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.transactions.repeat]
// r[verify agent_harness_runtime.pending_input_vault.recovery.restore]
fn exact_restore_is_idempotent_and_emits_admission_references_only() {
    let projected = projection();
    let request = disposition_request(&projected);
    let decision = plan_disposition(&projected, &request, None).expect("plan");
    let DispositionDecision::Planned(plan) = decision else {
        panic!("new disposition must produce plan");
    };
    let outcome = reduce_commit(
        &projected,
        &plan,
        &CommitObservation {
            plan_identity: plan.plan_identity.clone(),
            generation: plan.expected_generation,
            status: CommitStatus::Committed,
            restore_material_retained: true,
        },
    )
    .expect("commit reduction");
    let CommitOutcome::Completed(completed) = outcome else {
        panic!("commit must complete");
    };
    assert_eq!(completed.admission_requests.len(), SELECTED_COUNT);
    assert_eq!(completed.cleanup, CleanupState::Required);
    assert_eq!(completed.admission_requests[0].entry.as_str(), "entry.1");
    assert_eq!(completed.admission_requests[1].entry.as_str(), "entry.3");

    let repeated = plan_disposition(&projected, &request, Some(&completed)).expect("repeat");
    assert_eq!(
        repeated,
        DispositionDecision::Repeated((*completed).clone())
    );
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.transactions.drift]
fn changed_selection_generation_scope_and_authority_fail_closed() {
    let projected = projection();
    let request = disposition_request(&projected);
    let DispositionDecision::Planned(plan) =
        plan_disposition(&projected, &request, None).expect("plan")
    else {
        panic!("new request must plan");
    };
    let CommitOutcome::Completed(completed) = reduce_commit(
        &projected,
        &plan,
        &CommitObservation {
            plan_identity: plan.plan_identity.clone(),
            generation: plan.expected_generation,
            status: CommitStatus::Committed,
            restore_material_retained: true,
        },
    )
    .expect("complete") else {
        panic!("commit must complete");
    };

    let mut changed = request.clone();
    changed.selected = vec![EntryIdentity::new("entry.2")];
    assert_eq!(
        plan_disposition(&projected, &changed, Some(&completed))
            .expect_err("changed selection")
            .issue(),
        Issue::Conflict
    );
    let mut noncanonical = request.clone();
    noncanonical.selected.reverse();
    assert_eq!(
        plan_disposition(&projected, &noncanonical, None)
            .expect_err("selection order drift")
            .issue(),
        Issue::Conflict
    );
    let mut stale = request.clone();
    stale.expected_generation = Generation::new(STALE_GENERATION_VALUE);
    assert_eq!(
        plan_disposition(&projected, &stale, None)
            .expect_err("stale generation")
            .issue(),
        Issue::Stale
    );
    let mut drifted = request.clone();
    drifted.boundary.authority = AuthorityIdentity::new("authority.drifted");
    assert_eq!(
        plan_disposition(&projected, &drifted, None)
            .expect_err("authority drift")
            .issue(),
        Issue::Stale
    );
    let mut denied = request;
    denied.authorized = false;
    assert_eq!(
        plan_disposition(&projected, &denied, None)
            .expect_err("authority denial")
            .issue(),
        Issue::Unauthorized
    );
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.transactions.cleanup]
fn completed_evidence_survives_cleanup_failure_without_repeat_work() {
    let projected = projection();
    let request = disposition_request(&projected);
    let DispositionDecision::Planned(plan) =
        plan_disposition(&projected, &request, None).expect("plan")
    else {
        panic!("new request must plan");
    };
    let CommitOutcome::Completed(completed) = reduce_commit(
        &projected,
        &plan,
        &CommitObservation {
            plan_identity: plan.plan_identity.clone(),
            generation: plan.expected_generation,
            status: CommitStatus::Committed,
            restore_material_retained: true,
        },
    )
    .expect("complete") else {
        panic!("commit must complete");
    };
    let retained = *completed;
    let mut restart = facts();
    restart.completed = Some(retained.clone());
    restart.cleanup = CleanupState::Failed;
    let recovered = project_startup(&policy(), &restart).expect("restart projection");
    assert_eq!(recovered.state, RecoveryState::CleanupRequired);
    assert!(recovered.entries.len() == EXPECTED_CLASS_COUNT);
    assert!(!ordinary_ingress_allowed(&recovered.state));
    let repeated = plan_disposition(&recovered, &request, Some(&retained))
        .expect("exact restart repeat must resolve from completed evidence");
    assert_eq!(repeated, DispositionDecision::Repeated(retained));
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.corruption.control]
fn blocked_state_exposes_only_authorized_recovery_controls() {
    let mut corrupt = facts();
    corrupt.uncertain_commit = true;
    let projected = project_startup(&policy(), &corrupt).expect("blocked");
    let mut request = RecoveryControlRequest {
        boundary: projected.boundary.clone(),
        projection: projected.identity.clone(),
        kind: RecoveryControlKind::QuarantineExport,
        authorized: false,
    };
    assert_eq!(
        plan_recovery_control(&projected, &request)
            .expect_err("unauthorized")
            .issue(),
        Issue::Unauthorized
    );
    request.authorized = true;
    for kind in [
        RecoveryControlKind::Inspect,
        RecoveryControlKind::QuarantineExport,
        RecoveryControlKind::RepairImport,
        RecoveryControlKind::DestructiveDiscard,
    ] {
        request.kind = kind;
        let plan = plan_recovery_control(&projected, &request).expect("authorized control");
        assert_eq!(plan.kind, kind);
        assert!(!plan.plan_identity.as_str().is_empty());
    }
    let mut stale = request;
    stale.boundary.store = StoreIdentity::new("store.other");
    assert_eq!(
        plan_recovery_control(&projected, &stale)
            .expect_err("stale recovery control")
            .issue(),
        Issue::Stale
    );
    assert!(!ordinary_ingress_allowed(&projected.state));
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.evidence.safe]
// r[verify agent_harness_runtime.pending_input_vault.evidence.rejects]
fn evidence_is_metadata_only_and_rejects_invalid_claims() {
    let projected = projection();
    let receipt = project_receipt(&projected, None, SensitiveDraft::default()).expect("receipt");
    assert_eq!(receipt.entry_count, EXPECTED_CLASS_COUNT_U32);
    assert!(validate_receipt(&receipt).is_ok());
    assert!(!receipt.id.as_str().is_empty());
    assert_eq!(receipt.disposition, None);

    let mut malformed_receipt = receipt.clone();
    malformed_receipt.id = ReceiptIdentity::new("bad");
    assert_eq!(
        validate_receipt(&malformed_receipt)
            .expect_err("invalid receipt identity")
            .issue(),
        Issue::InvalidReceipt
    );
    let mut inconsistent_counts = receipt.clone();
    inconsistent_counts.class_counts.pop();
    assert_eq!(
        validate_receipt(&inconsistent_counts)
            .expect_err("inconsistent receipt counts")
            .issue(),
        Issue::InvalidReceipt
    );

    let mut overclaim = receipt.clone();
    overclaim.claims.push("power-loss-durable".to_string());
    assert_eq!(
        validate_receipt(&overclaim).expect_err("overclaim").issue(),
        Issue::ReceiptOverclaim
    );
    let mut weakened = receipt;
    weakened.non_claims.clear();
    assert_eq!(
        validate_receipt(&weakened)
            .expect_err("missing non-claim")
            .issue(),
        Issue::MissingNonClaim
    );
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.evidence.rejects]
fn sensitive_receipt_fields_are_rejected() {
    let projected = projection();
    assert_eq!(projected.state, RecoveryState::ReviewRequired);
    assert_eq!(projected.entries.len(), EXPECTED_CLASS_COUNT);
    for sensitive in [
        SensitiveDraft {
            body: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            attachment: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            prompt: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            tool_data: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            provider_data: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            credential: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            environment_value: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            physical_path: Some("private"),
            ..SensitiveDraft::default()
        },
        SensitiveDraft {
            raw_store_error: Some("private"),
            ..SensitiveDraft::default()
        },
    ] {
        assert_eq!(
            project_receipt(&projected, None, sensitive)
                .expect_err("sensitive evidence")
                .issue(),
            Issue::ReceiptLeak
        );
    }
}

#[derive(Clone)]
struct FixtureService {
    facts: StartupFacts,
    load_calls: u32,
    commit_calls: u32,
    cleanup_calls: u32,
    body_calls: u32,
    recovery_calls: u32,
    cleanup_observation: CleanupObservation,
}

impl FixtureService {
    fn new() -> Self {
        Self {
            facts: facts(),
            load_calls: ZERO_CALLS,
            commit_calls: ZERO_CALLS,
            cleanup_calls: ZERO_CALLS,
            body_calls: ZERO_CALLS,
            recovery_calls: ZERO_CALLS,
            cleanup_observation: CleanupObservation::Complete,
        }
    }
}

impl Service for FixtureService {
    fn load(&mut self, _request: &LoadRequest) -> Result<StartupFacts, ServiceIssue> {
        self.load_calls += ONE_CALL;
        Ok(self.facts.clone())
    }

    fn commit(&mut self, plan: &DispositionPlan) -> Result<CommitObservation, ServiceIssue> {
        self.commit_calls += ONE_CALL;
        Ok(CommitObservation {
            plan_identity: plan.plan_identity.clone(),
            generation: plan.expected_generation,
            status: CommitStatus::Committed,
            restore_material_retained: plan.kind == DispositionKind::Restore,
        })
    }

    fn cleanup(
        &mut self,
        _completed: &CompletedDisposition,
    ) -> Result<CleanupObservation, ServiceIssue> {
        self.cleanup_calls += ONE_CALL;
        Ok(self.cleanup_observation)
    }

    fn load_body(&mut self, request: &AdmissionRequest) -> Result<LoadedBody, ServiceIssue> {
        self.body_calls += ONE_CALL;
        Ok(LoadedBody {
            entry: request.entry.clone(),
            body_identity: request.body_identity.clone(),
            body: LOADED_BODY.to_string(),
        })
    }

    fn recover(
        &mut self,
        _plan: &RecoveryControlPlan,
    ) -> Result<RecoveryControlObservation, ServiceIssue> {
        self.recovery_calls += ONE_CALL;
        Ok(RecoveryControlObservation::Complete)
    }
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.boundary.supplied]
fn injected_service_never_loads_bodies_or_dispatches_during_startup() {
    let mut service = FixtureService::new();
    let projected = load_startup(&mut service, &policy(), &boundary()).expect("load");
    assert_eq!(projected.state, RecoveryState::ReviewRequired);
    assert_eq!(service.load_calls, ONE_CALL);
    assert_eq!(service.commit_calls, ZERO_CALLS);
    assert_eq!(service.cleanup_calls, ZERO_CALLS);
    assert_eq!(service.body_calls, ZERO_CALLS);
    assert_eq!(service.recovery_calls, ZERO_CALLS);

    let request = disposition_request(&projected);
    let DispositionDecision::Planned(plan) =
        plan_disposition(&projected, &request, None).expect("plan")
    else {
        panic!("new request must plan");
    };
    let CommitOutcome::Completed(completed) =
        commit_plan(&mut service, &projected, &plan).expect("commit")
    else {
        panic!("commit must complete");
    };
    assert_eq!(service.commit_calls, ONE_CALL);
    assert_eq!(service.body_calls, ZERO_CALLS);

    let cleaned = cleanup_completed(&mut service, &completed).expect("cleanup");
    assert_eq!(cleaned.cleanup, CleanupState::Complete);
    assert_eq!(service.cleanup_calls, ONE_CALL);
    assert_eq!(service.body_calls, ZERO_CALLS);

    let loaded = load_admitted_body(&mut service, &completed.admission_requests[0])
        .expect("current admission body");
    assert_eq!(loaded.body, LOADED_BODY);
    assert_eq!(service.body_calls, ONE_CALL);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.transactions.drift]
fn forged_mutation_plans_fail_before_service_calls() {
    let projected = projection();
    let request = disposition_request(&projected);
    let DispositionDecision::Planned(mut plan) =
        plan_disposition(&projected, &request, None).expect("plan")
    else {
        panic!("new request must plan");
    };
    assert_eq!(
        reduce_commit(
            &projected,
            &plan,
            &CommitObservation {
                plan_identity: plan.plan_identity.clone(),
                generation: plan.expected_generation,
                status: CommitStatus::Committed,
                restore_material_retained: false,
            },
        )
        .expect_err("restore material was not retained")
        .issue(),
        Issue::Conflict
    );
    plan.plan_identity = PlanIdentity::new(BODY_HASH);
    let mut service = FixtureService::new();
    assert_eq!(
        commit_plan(&mut service, &projected, &plan)
            .expect_err("forged plan")
            .issue(),
        Issue::Conflict
    );
    assert_eq!(service.commit_calls, ZERO_CALLS);

    let mut corrupt = facts();
    corrupt.uncertain_commit = true;
    let blocked = project_startup(&policy(), &corrupt).expect("blocked");
    let request = RecoveryControlRequest {
        boundary: blocked.boundary.clone(),
        projection: blocked.identity.clone(),
        kind: RecoveryControlKind::QuarantineExport,
        authorized: true,
    };
    let mut recovery_plan = plan_recovery_control(&blocked, &request).expect("control plan");
    recovery_plan.boundary.store = StoreIdentity::new("store.other");
    assert_eq!(
        execute_recovery_control(&mut service, &blocked.boundary, &recovery_plan)
            .expect_err("forged recovery plan")
            .issue(),
        Issue::Conflict
    );
    assert_eq!(service.recovery_calls, ZERO_CALLS);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.transactions.cleanup]
fn failed_cleanup_can_retry_without_repeating_disposition_commit() {
    let projected = projection();
    let request = disposition_request(&projected);
    let DispositionDecision::Planned(plan) =
        plan_disposition(&projected, &request, None).expect("plan")
    else {
        panic!("new request must plan");
    };
    let mut service = FixtureService::new();
    service.cleanup_observation = CleanupObservation::Failed;
    let CommitOutcome::Completed(completed) =
        commit_plan(&mut service, &projected, &plan).expect("commit")
    else {
        panic!("commit must complete");
    };
    let failed = cleanup_completed(&mut service, &completed).expect("failed cleanup observation");
    assert_eq!(failed.cleanup, CleanupState::Failed);
    assert_eq!(service.commit_calls, ONE_CALL);
    assert_eq!(service.cleanup_calls, ONE_CALL);

    service.cleanup_observation = CleanupObservation::Complete;
    let cleaned = cleanup_completed(&mut service, &failed).expect("cleanup retry");
    assert_eq!(cleaned.cleanup, CleanupState::Complete);
    assert_eq!(service.commit_calls, ONE_CALL);
    assert_eq!(service.cleanup_calls, TWO_CALLS);
    assert_eq!(service.body_calls, ZERO_CALLS);
}

#[test]
// r[verify agent_harness_runtime.pending_input_vault.boundary.ambient]
// r[verify agent_harness_runtime.pending_input_vault.verification.incomplete]
fn missing_service_and_mismatched_loaded_facts_fail_without_fallback() {
    struct Unavailable;

    impl Service for Unavailable {
        fn load(&mut self, _request: &LoadRequest) -> Result<StartupFacts, ServiceIssue> {
            Err(ServiceIssue::Unavailable)
        }

        fn commit(&mut self, _plan: &DispositionPlan) -> Result<CommitObservation, ServiceIssue> {
            Err(ServiceIssue::Unavailable)
        }

        fn cleanup(
            &mut self,
            _completed: &CompletedDisposition,
        ) -> Result<CleanupObservation, ServiceIssue> {
            Err(ServiceIssue::Unavailable)
        }

        fn load_body(&mut self, _request: &AdmissionRequest) -> Result<LoadedBody, ServiceIssue> {
            Err(ServiceIssue::Unavailable)
        }

        fn recover(
            &mut self,
            _plan: &RecoveryControlPlan,
        ) -> Result<RecoveryControlObservation, ServiceIssue> {
            Err(ServiceIssue::Unavailable)
        }
    }

    let mut invalid_boundary = boundary();
    invalid_boundary.store = StoreIdentity::new("");
    let mut untouched = FixtureService::new();
    assert_eq!(
        load_startup(&mut untouched, &policy(), &invalid_boundary)
            .expect_err("invalid explicit boundary")
            .issue(),
        Issue::UnsafeMetadata
    );
    assert_eq!(untouched.load_calls, ZERO_CALLS);

    let mut service = Unavailable;
    assert_eq!(
        load_startup(&mut service, &policy(), &boundary())
            .expect_err("missing service")
            .issue(),
        Issue::ServiceUnavailable
    );

    let mut mismatched = FixtureService::new();
    mismatched.facts.boundary.conversation = ConversationIdentity::new("conversation.other");
    assert_eq!(
        load_startup(&mut mismatched, &policy(), &boundary())
            .expect_err("mismatched load")
            .issue(),
        Issue::Stale
    );
}
