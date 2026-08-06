use super::*;

#[test]
fn trusted_allow_policy_executes_after_authority_passes() {
    let policy = allow_policy();
    let resolution = resolve(allowed_request(&policy));

    assert_eq!(policy.action(), ACTION);
    assert_eq!(policy.resource_label(), RESOURCE);
    assert_eq!(policy.outcome(), Outcome::Allow);
    assert_eq!(policy.source_identity(), SOURCE);
    assert_eq!(policy.trust_scope(), TrustScope::User);
    assert_eq!(policy.trust_status(), TrustStatus::Trusted);
    assert_eq!(policy.precedence(), MIN_PRECEDENCE);
    assert_eq!(policy.prompt_summary_policy(), PromptSummaryPolicy::Redacted);
    assert!(!policy.policy_hash().is_empty());
    assert_eq!(resolution.status(), DecisionStatus::Execute);
    assert_eq!(resolution.reason(), DecisionReason::None);
    assert_eq!(resolution.local_outcome(), Some(Outcome::Allow));
    assert_eq!(resolution.prompt_status(), PromptStatus::NotRequired);
    assert!(resolution.prompt_request().is_none());
    assert!(resolution.dispatch_allowed());
}

#[test]
fn ask_policy_creates_scoped_prompt_and_approved_decision_executes() {
    let policy = ask_policy();
    let initial = resolve(allowed_request(&policy));
    let prompt = initial.prompt_request().expect("ask policy should prompt");
    let decision = Decision::new(DecisionDraft {
        action: ACTION,
        resource_label: RESOURCE,
        session_label: SESSION,
        policy_hash: policy.policy_hash(),
        decision_token: prompt.decision_token(),
        outcome: DecisionOutcome::Approved,
    })
    .expect("prompt decision should validate");
    let continued = resolve(ResolutionRequest {
        prompt_decision: Some(&decision),
        ..allowed_request(&policy)
    });

    assert_eq!(initial.status(), DecisionStatus::PromptRequired);
    assert_eq!(initial.reason(), DecisionReason::PromptRequired);
    assert_eq!(initial.local_outcome(), Some(Outcome::Ask));
    assert_eq!(initial.prompt_status(), PromptStatus::Required);
    assert_eq!(prompt.action(), ACTION);
    assert_eq!(prompt.resource_label(), RESOURCE);
    assert_eq!(prompt.session_label(), SESSION);
    assert_eq!(prompt.policy_hash(), policy.policy_hash());
    assert_eq!(prompt.redacted_summary(), SUMMARY);
    assert_eq!(continued.status(), DecisionStatus::Execute);
    assert_eq!(continued.prompt_status(), PromptStatus::Approved);
    assert!(continued.dispatch_allowed());
}

#[test]
fn safe_permission_receipt_records_hashes_and_markers_only() {
    let resource_hash = safe_hash();
    let policy_hash = safe_hash();
    let receipt = Receipt::new(ReceiptDraft {
        action: ACTION,
        resource_hash: &resource_hash,
        authority_status: crate::AuthorizationStatus::Allowed,
        authority_issue: crate::AuthorizationIssue::None,
        local_outcome: Some(Outcome::Allow),
        prompt_status: PromptStatus::NotRequired,
        policy_hash: &policy_hash,
        reason: "none",
        redaction_markers: &[REDACTION, SECOND_REDACTION],
    })
    .expect("safe permission receipt should validate");

    assert_eq!(receipt.action(), ACTION);
    assert_eq!(receipt.resource_hash().as_str(), HASH);
    assert_eq!(receipt.authority_status(), crate::AuthorizationStatus::Allowed);
    assert_eq!(receipt.authority_issue(), crate::AuthorizationIssue::None);
    assert_eq!(receipt.local_outcome(), Some(Outcome::Allow));
    assert_eq!(receipt.prompt_status(), PromptStatus::NotRequired);
    assert_eq!(receipt.policy_hash().as_str(), HASH);
    assert_eq!(receipt.reason(), "none");
    assert_eq!(receipt.redaction_markers(), &[Text::from(REDACTION), Text::from(SECOND_REDACTION)]);
    validate_text(SAFE_RECEIPT_TEXT).expect("safe receipt should pass leak scan");
}
