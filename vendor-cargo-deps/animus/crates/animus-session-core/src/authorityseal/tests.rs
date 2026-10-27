#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const OPERATIONS: &[&str] = &[HASH_A, HASH_B];
const RESOURCES: &[&str] = &[HASH_C];
const EXPECTED: &[RuntimeAuthorityExpectedViewDraft<'static>] = &[
    expected(RuntimeAuthorityViewClass::SessionLineage),
    expected(RuntimeAuthorityViewClass::DurableBinding),
    expected(RuntimeAuthorityViewClass::AuthorityProfile),
    expected(RuntimeAuthorityViewClass::EnforcementObservation),
    expected(RuntimeAuthorityViewClass::OperationSet),
    expected(RuntimeAuthorityViewClass::AdapterCohort),
    expected(RuntimeAuthorityViewClass::ResourceProfile),
    expected(RuntimeAuthorityViewClass::ResourceBudget),
    expected(RuntimeAuthorityViewClass::RuntimeServices),
];
const OBSERVED: &[RuntimeAuthorityObservedViewDraft<'static>] = &[
    observed(RuntimeAuthorityViewClass::SessionLineage),
    observed(RuntimeAuthorityViewClass::DurableBinding),
    observed(RuntimeAuthorityViewClass::AuthorityProfile),
    observed(RuntimeAuthorityViewClass::EnforcementObservation),
    observed(RuntimeAuthorityViewClass::OperationSet),
    observed(RuntimeAuthorityViewClass::AdapterCohort),
    observed(RuntimeAuthorityViewClass::ResourceProfile),
    observed(RuntimeAuthorityViewClass::ResourceBudget),
    observed(RuntimeAuthorityViewClass::RuntimeServices),
];

const fn expected(class: RuntimeAuthorityViewClass) -> RuntimeAuthorityExpectedViewDraft<'static> {
    RuntimeAuthorityExpectedViewDraft {
        class,
        identity: HASH_A,
    }
}

const fn observed(class: RuntimeAuthorityViewClass) -> RuntimeAuthorityObservedViewDraft<'static> {
    RuntimeAuthorityObservedViewDraft {
        class,
        identity: HASH_A,
        status: RuntimeAuthorityObservationStatus::Current,
    }
}

fn draft() -> RuntimeAuthorityUnitDraft<'static> {
    RuntimeAuthorityUnitDraft {
        session_id: "session.fixture",
        dispatch_id: "dispatch.fixture",
        session_lineage_identity: HASH_A,
        durable_binding_identity: HASH_A,
        authority_profile_identity: HASH_A,
        enforcement_observation_identity: HASH_A,
        operation_set_identity: HASH_A,
        adapter_cohort_identity: HASH_A,
        resource_profile_identity: HASH_A,
        resource_budget_identity: HASH_A,
        runtime_services_identity: HASH_A,
        operation_identities: OPERATIONS,
        resource_identities: RESOURCES,
        expected_views: EXPECTED,
        observed_views: OBSERVED,
    }
}

fn receipt_draft() -> RuntimeAuthorityReceiptDraft<'static> {
    RuntimeAuthorityReceiptDraft {
        sensitive: RuntimeAuthoritySensitiveDraft::default(),
        semantic_claims: &[],
    }
}

fn plan() -> RuntimeAuthoritySealPlan {
    prepare_runtime_authority_seal(draft(), receipt_draft()).expect("valid seal plan")
}

fn sealed_unit() -> RuntimeAuthorityUnit {
    let plan = plan();
    commit_runtime_authority_seal(&plan, plan.receipt().receipt_identity.as_str())
        .expect("seal commit")
}

fn recheck() -> RuntimeAuthorityRecheckDraft<'static> {
    RuntimeAuthorityRecheckDraft {
        session_lineage_identity: HASH_A,
        authority_profile_identity: HASH_A,
        durable_binding_identity: HASH_A,
        enforcement_observation_identity: HASH_A,
        operation_set_identity: HASH_A,
        adapter_cohort_identity: HASH_A,
        resource_profile_identity: HASH_A,
        resource_budget_identity: HASH_A,
        runtime_services_identity: HASH_A,
        operation_identities: OPERATIONS,
        resource_identities: RESOURCES,
        admitted: true,
    }
}

fn has_issue(error: &RuntimeAuthoritySealError, code: RuntimeAuthorityIssueCode) -> bool {
    error.issues().iter().any(|issue| issue.code() == code)
}

fn set_declared_identity<'a>(
    draft: &mut RuntimeAuthorityUnitDraft<'a>,
    class: RuntimeAuthorityViewClass,
    identity: &'a str,
) {
    match class {
        RuntimeAuthorityViewClass::SessionLineage => draft.session_lineage_identity = identity,
        RuntimeAuthorityViewClass::DurableBinding => draft.durable_binding_identity = identity,
        RuntimeAuthorityViewClass::AuthorityProfile => draft.authority_profile_identity = identity,
        RuntimeAuthorityViewClass::EnforcementObservation => {
            draft.enforcement_observation_identity = identity;
        }
        RuntimeAuthorityViewClass::OperationSet => draft.operation_set_identity = identity,
        RuntimeAuthorityViewClass::AdapterCohort => draft.adapter_cohort_identity = identity,
        RuntimeAuthorityViewClass::ResourceProfile => draft.resource_profile_identity = identity,
        RuntimeAuthorityViewClass::ResourceBudget => draft.resource_budget_identity = identity,
        RuntimeAuthorityViewClass::RuntimeServices => draft.runtime_services_identity = identity,
    }
}

#[test]
fn prepares_commits_runs_and_completes_one_unit() {
    // r[verify agent_harness_runtime.runtime_authority_seal.phase]
    // r[verify agent_harness_runtime.runtime_authority_seal.receipt]
    // r[verify agent_harness_runtime.runtime_authority_seal.boundary]
    // r[verify agent_harness_runtime.runtime_authority_seal.verification]
    let plan = plan();
    assert_eq!(plan.unit().state(), RuntimeAuthorityUnitState::Preparing);
    assert!(validate_runtime_authority_seal_receipt(plan.unit(), plan.receipt()).is_ok());
    let sealed = commit_runtime_authority_seal(&plan, plan.receipt().receipt_identity.as_str())
        .expect("seal commit");
    let running = start_runtime_authority_effect(&sealed, recheck()).expect("start effect");
    let completed =
        transition_runtime_authority_unit(&running, RuntimeAuthorityTransition::Complete)
            .expect("complete unit");
    assert_eq!(completed.state(), RuntimeAuthorityUnitState::Completed);
    assert_eq!(completed.unit_identity(), plan.unit().unit_identity());
}

#[test]
fn canonical_order_is_deterministic() {
    // r[verify agent_harness_runtime.runtime_authority_seal.runtime_view.exact]
    let first = plan();
    let reversed_operations = [HASH_B, HASH_A];
    let mut reversed_expected = EXPECTED.to_vec();
    let mut reversed_observed = OBSERVED.to_vec();
    reversed_expected.reverse();
    reversed_observed.reverse();
    let mut input = draft();
    input.operation_identities = &reversed_operations;
    input.expected_views = reversed_expected.as_slice();
    input.observed_views = reversed_observed.as_slice();
    let second = prepare_runtime_authority_seal(input, receipt_draft()).expect("reversed");
    assert_eq!(first.unit().unit_identity(), second.unit().unit_identity());
    assert_eq!(
        first.receipt().receipt_identity,
        second.receipt().receipt_identity
    );
}

#[test]
fn each_changed_declared_authority_creates_a_new_unit_identity() {
    // r[verify agent_harness_runtime.runtime_authority_seal.monotonic.new_unit]
    let first = plan();
    for class in RuntimeAuthorityViewClass::ALL {
        let mut expected = EXPECTED.to_vec();
        let mut observed = OBSERVED.to_vec();
        let expected_view = expected
            .iter_mut()
            .find(|view| view.class == class)
            .expect("required expected view");
        expected_view.identity = HASH_B;
        let observed_view = observed
            .iter_mut()
            .find(|view| view.class == class)
            .expect("required observed view");
        observed_view.identity = HASH_B;
        let mut input = draft();
        set_declared_identity(&mut input, class, HASH_B);
        input.expected_views = expected.as_slice();
        input.observed_views = observed.as_slice();
        let second = prepare_runtime_authority_seal(input, receipt_draft()).expect("new unit");
        assert_ne!(first.unit().unit_identity(), second.unit().unit_identity());
        assert_ne!(
            first.receipt().receipt_identity,
            second.receipt().receipt_identity
        );
    }
}

#[test]
fn rejects_missing_duplicate_and_bad_runtime_views() {
    // r[verify agent_harness_runtime.runtime_authority_seal.runtime_view]
    // r[verify agent_harness_runtime.runtime_authority_seal.runtime_view.mismatch]
    let mut missing = OBSERVED.to_vec();
    let _ = missing.pop();
    let mut input = draft();
    input.observed_views = missing.as_slice();
    let error = prepare_runtime_authority_seal(input, receipt_draft()).expect_err("missing");
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::MissingView));

    let mut duplicate = OBSERVED.to_vec();
    duplicate.push(OBSERVED[0]);
    let mut input = draft();
    input.observed_views = duplicate.as_slice();
    let error = prepare_runtime_authority_seal(input, receipt_draft()).expect_err("duplicate");
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::DuplicateView));

    for (status, code) in [
        (
            RuntimeAuthorityObservationStatus::Stale,
            RuntimeAuthorityIssueCode::StaleView,
        ),
        (
            RuntimeAuthorityObservationStatus::Unknown,
            RuntimeAuthorityIssueCode::UnknownView,
        ),
        (
            RuntimeAuthorityObservationStatus::Unsupported,
            RuntimeAuthorityIssueCode::UnsupportedView,
        ),
    ] {
        let mut changed = OBSERVED.to_vec();
        changed[0].status = status;
        let mut input = draft();
        input.observed_views = changed.as_slice();
        let error = prepare_runtime_authority_seal(input, receipt_draft()).expect_err("status");
        assert!(has_issue(&error, code));
    }

    let mut mismatch = OBSERVED.to_vec();
    mismatch[0].identity = HASH_B;
    let mut input = draft();
    input.observed_views = mismatch.as_slice();
    let error = prepare_runtime_authority_seal(input, receipt_draft()).expect_err("mismatch");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::ViewIdentityMismatch
    ));

    let mut declared_mismatch = draft();
    declared_mismatch.authority_profile_identity = HASH_B;
    let error = prepare_runtime_authority_seal(declared_mismatch, receipt_draft())
        .expect_err("declared view mismatch");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::ViewIdentityMismatch
    ));
}

#[test]
fn issue_order_is_deterministic() {
    let first_operations = ["bad-hash", HASH_A, HASH_A];
    let second_operations = [HASH_A, HASH_A, "bad-hash"];
    let mut first = draft();
    first.session_id = "/unsafe";
    first.operation_identities = &first_operations;
    let mut second = draft();
    second.session_id = "/unsafe";
    second.operation_identities = &second_operations;
    let first_error = prepare_runtime_authority_seal(first, receipt_draft()).expect_err("first");
    let second_error = prepare_runtime_authority_seal(second, receipt_draft()).expect_err("second");
    assert_eq!(first_error.issues(), second_error.issues());
}

#[test]
fn rejects_unsafe_duplicate_and_empty_inputs() {
    let mut unsafe_input = draft();
    unsafe_input.session_id = "/private/session";
    unsafe_input.authority_profile_identity = "not-a-hash";
    let error = prepare_runtime_authority_seal(unsafe_input, receipt_draft()).expect_err("unsafe");
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::UnsafeLabel));
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::UnsafeIdentity));

    let duplicate_operations = [HASH_A, HASH_A];
    let duplicate_resources = [HASH_C, HASH_C];
    let mut duplicate = draft();
    duplicate.operation_identities = &duplicate_operations;
    duplicate.resource_identities = &duplicate_resources;
    let error = prepare_runtime_authority_seal(duplicate, receipt_draft()).expect_err("duplicate");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::DuplicateOperation
    ));
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::DuplicateResource
    ));

    let mut empty = draft();
    empty.operation_identities = &[];
    let error = prepare_runtime_authority_seal(empty, receipt_draft()).expect_err("empty");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::EmptyOperationSet
    ));
}

#[test]
fn seal_commit_and_current_recheck_fail_closed() {
    // r[verify agent_harness_runtime.runtime_authority_seal.phase.early_effect]
    // r[verify agent_harness_runtime.runtime_authority_seal.boundary.authorization]
    let plan = plan();
    let mismatch = commit_runtime_authority_seal(&plan, HASH_B).expect_err("wrong receipt");
    assert!(has_issue(
        &mismatch,
        RuntimeAuthorityIssueCode::ReceiptMismatch
    ));
    let early = start_runtime_authority_effect(plan.unit(), recheck()).expect_err("early effect");
    assert!(has_issue(
        &early,
        RuntimeAuthorityIssueCode::InvalidStateTransition
    ));

    let sealed = sealed_unit();
    let mut denied = recheck();
    denied.admitted = false;
    let error = start_runtime_authority_effect(&sealed, denied).expect_err("denied");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::CurrentEnforcementDenied
    ));
    let narrowed_operations = [HASH_A];
    let mut narrowed = recheck();
    narrowed.operation_identities = &narrowed_operations;
    let error = start_runtime_authority_effect(&sealed, narrowed).expect_err("narrowed");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::CurrentEnforcementDenied
    ));
    let widened_operations = [HASH_A, HASH_B, HASH_C];
    let mut widened = recheck();
    widened.operation_identities = &widened_operations;
    let error = start_runtime_authority_effect(&sealed, widened).expect_err("widened");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::PostSealWidening
    ));
}

#[test]
fn narrowing_is_allowed_but_widening_and_mutation_are_denied() {
    // r[verify agent_harness_runtime.runtime_authority_seal.monotonic]
    // r[verify agent_harness_runtime.runtime_authority_seal.monotonic.denied]
    let sealed = sealed_unit();
    assert!(validate_runtime_authority_narrowing(&sealed, &[HASH_A], &[]).is_ok());
    let error = validate_runtime_authority_narrowing(&sealed, &[HASH_A, HASH_C], &[])
        .expect_err("widening");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::PostSealWidening
    ));
    for mutation in [
        RuntimeAuthorityMutationKind::CapabilityAddition,
        RuntimeAuthorityMutationKind::ProfileRebind,
        RuntimeAuthorityMutationKind::AdapterReplacement,
        RuntimeAuthorityMutationKind::ResourceExpansion,
        RuntimeAuthorityMutationKind::AmbientDiscovery,
        RuntimeAuthorityMutationKind::Fallback,
        RuntimeAuthorityMutationKind::RepeatedSeal,
    ] {
        let error = transition_runtime_authority_unit(
            &sealed,
            RuntimeAuthorityTransition::AttemptMutation(mutation),
        )
        .expect_err("post-seal mutation");
        assert!(has_issue(
            &error,
            RuntimeAuthorityIssueCode::PostSealWidening
        ));
    }
}

#[test]
fn terminal_units_cannot_be_reused() {
    let sealed = sealed_unit();
    let cancelled = transition_runtime_authority_unit(&sealed, RuntimeAuthorityTransition::Cancel)
        .expect("cancel");
    let error = transition_runtime_authority_unit(
        &cancelled,
        RuntimeAuthorityTransition::AttemptMutation(RuntimeAuthorityMutationKind::StaleReuse),
    )
    .expect_err("stale reuse");
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::StaleUnitReuse));
}

#[test]
fn receipts_reject_sensitive_data_overclaims_and_tampering() {
    // r[verify agent_harness_runtime.runtime_authority_seal.receipt.overclaim]
    let sensitive = RuntimeAuthorityReceiptDraft {
        sensitive: RuntimeAuthoritySensitiveDraft {
            credential: Some("secret"),
            ..RuntimeAuthoritySensitiveDraft::default()
        },
        semantic_claims: &[],
    };
    let error = prepare_runtime_authority_seal(draft(), sensitive).expect_err("secret");
    assert!(has_issue(&error, RuntimeAuthorityIssueCode::UnsafeReceipt));
    let claims = RuntimeAuthorityReceiptDraft {
        sensitive: RuntimeAuthoritySensitiveDraft::default(),
        semantic_claims: &["this proves authorization"],
    };
    let error = prepare_runtime_authority_seal(draft(), claims).expect_err("overclaim");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::AuthorityOverclaim
    ));
    let plan = plan();
    let mut tampered = plan.receipt().clone();
    tampered.authority_profile_identity = HASH_B.to_owned();
    let error = validate_runtime_authority_seal_receipt(plan.unit(), &tampered)
        .expect_err("tampered receipt");
    assert!(has_issue(
        &error,
        RuntimeAuthorityIssueCode::ReceiptMismatch
    ));
}
