const CORRECTIVE_RECORD_ID: &str = "corrective-record-alpha";
const CORRECTIVE_SOURCE: &str = "host-review-store";
const CORRECTIVE_SUBJECT: &str = "animus-runtime";
const CORRECTIVE_CLASS: &str = "lint";
const CORRECTIVE_SCOPE: &str = "adapter";
const CORRECTIVE_ROUTE: &str = "deterministic-check";
const CORRECTIVE_BUDGET: u16 = 3;
const CORRECTIVE_SMALL_BUDGET: u16 = 1;
const CORRECTIVE_HASH_INPUT: &str = "corrective body summary";
const PROJECT_SCOPE_ONLY: [super::CorrectiveTrustScope; 1] = [super::CorrectiveTrustScope::Project];
const ACTION_ID: &str = "append-corrective-alpha";
const PROMOTE_ACTION_ID: &str = "promote-corrective-alpha";
const STORE_LABEL: &str = "local-corrective-store";
const ACTOR_LABEL: &str = "host-policy";
const RECEIPT_ISSUE: &str = "accepted-by-host";

fn body_hash() -> alloc::string::String {
    super::hash_text(CORRECTIVE_HASH_INPUT)
}

fn corrective_draft(body_hash: &str) -> super::CorrectiveRecordDraft<'_> {
    super::CorrectiveRecordDraft {
        record_id: CORRECTIVE_RECORD_ID,
        kind: super::CorrectiveRecordKind::Mistake,
        source: CORRECTIVE_SOURCE,
        trust_scope: super::CorrectiveTrustScope::Project,
        subject: CORRECTIVE_SUBJECT,
        class: CORRECTIVE_CLASS,
        scope: CORRECTIVE_SCOPE,
        route: CORRECTIVE_ROUTE,
        freshness: super::CorrectiveFreshness::Fresh,
        inclusion: super::CorrectiveInclusionPolicy::WhenRelevant,
        prompt_budget_units: CORRECTIVE_BUDGET,
        body_hash,
        redaction: super::CorrectiveRedactionState::Public,
    }
}

fn corrective_record() -> super::CorrectiveRecord {
    let hash = body_hash();
    let draft = corrective_draft(hash.as_str());
    super::validate_corrective_record(&draft).unwrap()
}

fn admission_policy<'a>(trusted_sources: &'a [&'a str]) -> super::CorrectiveAdmissionPolicy<'a> {
    super::CorrectiveAdmissionPolicy {
        trusted_sources,
        allowed_scopes: &PROJECT_SCOPE_ONLY,
        allow_stale: false,
        prompt_budget_units: CORRECTIVE_BUDGET,
    }
}

#[test]
fn corrective_records_validate_and_require_explicit_admission() {
    // r[verify agent_harness_runtime.hermes_corrective_memory.records]
    // r[verify agent_harness_runtime.hermes_corrective_memory.admission]
    let record = corrective_record();
    let policy = admission_policy(&[CORRECTIVE_SOURCE]);
    let decision = super::admit_corrective_record(&record, &policy);

    assert_eq!(record.kind(), super::CorrectiveRecordKind::Mistake);
    assert_eq!(record.source(), CORRECTIVE_SOURCE);
    assert_eq!(record.body_hash().len(), crate::SAFE_HASH_HEX_CHARS);
    assert_eq!(decision.status(), super::CorrectiveAdmissionStatus::Admitted);
    assert_eq!(decision.reason(), super::CorrectiveAdmissionReason::Accepted);
    assert_eq!(decision.consumed_budget_units(), CORRECTIVE_BUDGET);
    assert_eq!(decision.record_hash().len(), crate::SAFE_HASH_HEX_CHARS);
}

#[test]
fn corrective_admission_rejects_untrusted_always_read_and_budget_failures() {
    // r[verify agent_harness_runtime.hermes_corrective_memory.admission]
    // r[verify agent_harness_runtime.hermes_corrective_memory.verification]
    let hash = body_hash();
    let mut draft = corrective_draft(hash.as_str());
    let record = super::validate_corrective_record(&draft).unwrap();
    let untrusted = super::admit_corrective_record(&record, &admission_policy(&[]));

    assert_eq!(untrusted.status(), super::CorrectiveAdmissionStatus::Denied);
    assert_eq!(
        untrusted.reason(),
        super::CorrectiveAdmissionReason::SourceNotTrusted
    );

    draft.inclusion = super::CorrectiveInclusionPolicy::AlwaysRead;
    let always_read = super::validate_corrective_record(&draft).unwrap();
    let denied = super::admit_corrective_record(&always_read, &admission_policy(&[CORRECTIVE_SOURCE]));
    assert_eq!(denied.status(), super::CorrectiveAdmissionStatus::Denied);
    assert_eq!(
        denied.reason(),
        super::CorrectiveAdmissionReason::AlwaysReadRejected
    );

    draft.inclusion = super::CorrectiveInclusionPolicy::WhenRelevant;
    let budget_policy = super::CorrectiveAdmissionPolicy {
        prompt_budget_units: CORRECTIVE_SMALL_BUDGET,
        ..admission_policy(&[CORRECTIVE_SOURCE])
    };
    let omitted = super::admit_corrective_record(&record, &budget_policy);
    assert_eq!(omitted.status(), super::CorrectiveAdmissionStatus::Omitted);
    assert_eq!(
        omitted.reason(),
        super::CorrectiveAdmissionReason::BudgetExceeded
    );

    draft.body_hash = "not-a-blake3-hash";
    let invalid_hash = super::validate_corrective_record(&draft)
        .expect_err("body hash must be a BLAKE3-shaped identity");
    assert_eq!(invalid_hash.code(), super::ReviewDiagnosticCode::InvalidHash);
}

#[test]
fn corrective_promotions_route_repeated_review_findings() {
    // r[verify agent_harness_runtime.hermes_corrective_memory.promotions]
    let first = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=deterministic-check promoted=no] Add parser fixture.\n",
        sequence: SEQUENCE_ONE,
    })
    .unwrap();
    let second = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=deterministic-check promoted=no] Add another parser fixture.\n",
        sequence: SEQUENCE_TWO,
    })
    .unwrap();
    let records = alloc::vec![first, second];
    let candidates =
        super::corrective_promotion_candidates(records.as_slice(), TWO_FINDING_COUNT).unwrap();

    assert_eq!(candidates.len(), SINGLE_FINDING_COUNT);
    assert_eq!(candidates[ZERO_COUNT].class(), CORRECTIVE_CLASS);
    assert_eq!(candidates[ZERO_COUNT].scope(), CORRECTIVE_SCOPE);
    assert_eq!(
        candidates[ZERO_COUNT].target(),
        super::CorrectivePromotionTarget::DeterministicCheck
    );
    assert_eq!(candidates[ZERO_COUNT].count(), TWO_FINDING_COUNT);
    assert_eq!(
        candidates[ZERO_COUNT].candidate_hash().len(),
        crate::SAFE_HASH_HEX_CHARS
    );
}

#[test]
fn corrective_host_actions_require_store_and_safe_receipts() {
    // r[verify agent_harness_runtime.hermes_corrective_memory.host_actions]
    // r[verify agent_harness_runtime.hermes_corrective_memory.verification]
    let record = corrective_record();
    let append = super::validate_corrective_append_request(&super::CorrectiveAppendRequestDraft {
        action_id: ACTION_ID,
        store_label: STORE_LABEL,
        actor_label: ACTOR_LABEL,
        store_status: super::CorrectiveStoreStatus::Available,
        record: &record,
    })
    .unwrap();
    let missing_store =
        super::validate_corrective_append_request(&super::CorrectiveAppendRequestDraft {
            action_id: ACTION_ID,
            store_label: STORE_LABEL,
            actor_label: ACTOR_LABEL,
            store_status: super::CorrectiveStoreStatus::Missing,
            record: &record,
        })
        .expect_err("missing store must fail closed");
    let evidence_hash = alloc::string::String::from(append.record_hash());
    let receipt = super::validate_corrective_host_action_receipt(
        &super::CorrectiveHostActionReceiptDraft {
            action_kind: super::CorrectiveHostActionKind::AppendRecord,
            store_label: STORE_LABEL,
            status: super::CorrectiveHostActionReceiptStatus::Accepted,
            issue_code: Some(RECEIPT_ISSUE),
            evidence_hashes: &[evidence_hash.as_str()],
        },
    )
    .unwrap();
    let leaking_receipt = super::validate_corrective_host_action_receipt(
        &super::CorrectiveHostActionReceiptDraft {
            action_kind: super::CorrectiveHostActionKind::AppendRecord,
            store_label: STORE_LABEL,
            status: super::CorrectiveHostActionReceiptStatus::Denied,
            issue_code: Some("raw prompt leaked"),
            evidence_hashes: &[evidence_hash.as_str()],
        },
    )
    .expect_err("receipt issue code must be safe");

    assert_eq!(append.action_id(), ACTION_ID);
    assert_eq!(append.record_hash().len(), crate::SAFE_HASH_HEX_CHARS);
    assert_eq!(missing_store.code(), super::ReviewDiagnosticCode::MissingStore);
    assert_eq!(
        receipt.status(),
        super::CorrectiveHostActionReceiptStatus::Accepted
    );
    assert_eq!(receipt.receipt_hash().len(), crate::SAFE_HASH_HEX_CHARS);
    assert_eq!(receipt.evidence_hashes().len(), SINGLE_FINDING_COUNT);
    assert_eq!(
        leaking_receipt.code(),
        super::ReviewDiagnosticCode::UnsafeMetadata
    );
}

#[test]
fn corrective_promote_request_is_explicit_and_store_bound() {
    // r[verify agent_harness_runtime.hermes_corrective_memory.host_actions]
    let first = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=memory-guidance promoted=no] Add memory guidance.\n",
        sequence: SEQUENCE_ONE,
    })
    .unwrap();
    let second = record(RecordInput {
        markdown: "VERDICT: WARN\n\n## Findings\n- [class=lint scope=adapter route=memory-guidance promoted=no] Repeat memory guidance.\n",
        sequence: SEQUENCE_TWO,
    })
    .unwrap();
    let records = alloc::vec![first, second];
    let candidates =
        super::corrective_promotion_candidates(records.as_slice(), TWO_FINDING_COUNT).unwrap();
    let promote = super::validate_corrective_promote_request(
        &super::CorrectivePromoteRequestDraft {
            action_id: PROMOTE_ACTION_ID,
            store_label: STORE_LABEL,
            actor_label: ACTOR_LABEL,
            store_status: super::CorrectiveStoreStatus::Available,
            candidate: &candidates[ZERO_COUNT],
        },
    )
    .unwrap();

    assert_eq!(promote.action_id(), PROMOTE_ACTION_ID);
    assert_eq!(promote.store_label(), STORE_LABEL);
    assert_eq!(promote.actor_label(), ACTOR_LABEL);
    assert_eq!(
        promote.target(),
        super::CorrectivePromotionTarget::MemoryGuidance
    );
    assert_eq!(promote.candidate_hash().len(), crate::SAFE_HASH_HEX_CHARS);
}
