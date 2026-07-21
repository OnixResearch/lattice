use super::*;

#[test]
fn deny_policy_and_missing_authority_fail_before_dispatch() {
    let policy = deny_policy();
    let denied = resolve(allowed_request(&policy));
    let missing_authority = resolve(denied_authority_request(&policy));

    assert_eq!(denied.status(), DecisionStatus::Denied);
    assert_eq!(denied.reason(), DecisionReason::LocalDeny);
    assert_eq!(denied.local_outcome(), Some(Outcome::Deny));
    assert_eq!(denied.prompt_status(), PromptStatus::NotRequired);
    assert!(!denied.dispatch_allowed());
    assert_eq!(missing_authority.status(), DecisionStatus::Denied);
    assert_eq!(missing_authority.reason(), DecisionReason::MissingAuthority);
    assert_eq!(missing_authority.local_outcome(), Some(Outcome::Deny));
    assert!(!missing_authority.dispatch_allowed());
}

#[test]
fn missing_or_mismatched_policy_denies_dispatch() {
    let policy = allow_policy();
    let missing = resolve(ResolutionRequest {
        policy: None,
        ..allowed_request(&policy)
    });
    let mismatched = resolve(ResolutionRequest {
        resource_label: "tool:write",
        ..allowed_request(&policy)
    });

    assert_eq!(missing.status(), DecisionStatus::Denied);
    assert_eq!(missing.reason(), DecisionReason::NoMatchingPolicy);
    assert_eq!(missing.local_outcome(), None);
    assert_eq!(mismatched.status(), DecisionStatus::Denied);
    assert_eq!(mismatched.reason(), DecisionReason::NoMatchingPolicy);
    assert_eq!(mismatched.local_outcome(), Some(Outcome::Allow));
}

#[test]
fn ask_policy_rejects_denied_or_stale_prompt_decisions() {
    let policy = ask_policy();
    let prompt = resolve(allowed_request(&policy))
        .prompt_request()
        .expect("ask policy should prompt")
        .clone();
    let denied = decision(&policy, prompt.decision_token(), DecisionOutcome::Denied);
    let stale = decision(&policy, STALE_TOKEN, DecisionOutcome::Approved);
    let denied_resolution = resolve(ResolutionRequest {
        prompt_decision: Some(&denied),
        ..allowed_request(&policy)
    });
    let stale_resolution = resolve(ResolutionRequest {
        prompt_decision: Some(&stale),
        ..allowed_request(&policy)
    });

    assert_eq!(denied_resolution.status(), DecisionStatus::Denied);
    assert_eq!(denied_resolution.reason(), DecisionReason::PromptDenied);
    assert_eq!(denied_resolution.prompt_status(), PromptStatus::Denied);
    assert_eq!(stale_resolution.status(), DecisionStatus::Denied);
    assert_eq!(stale_resolution.reason(), DecisionReason::StalePromptToken);
    assert_eq!(stale_resolution.prompt_status(), PromptStatus::Stale);
}

#[test]
fn malformed_or_untrusted_policy_descriptors_are_rejected() {
    let bad_action = Policy::validate(&PolicyDraft {
        action: Some(BAD_ACTION),
        ..policy_draft(ALLOW_OUTCOME_LABEL)
    })
    .expect_err("unknown action should fail");
    assert_eq!(
        bad_action,
        Error::UnsupportedValue {
            field: ACTION_FIELD.as_str(),
            value: Text::from(BAD_ACTION),
        }
    );

    let untrusted = Policy::validate(&PolicyDraft {
        trust_status: Some(UNTRUSTED_LABEL),
        ..policy_draft(ALLOW_OUTCOME_LABEL)
    })
    .expect_err("untrusted policy should fail");
    assert_eq!(
        untrusted,
        Error::UntrustedDescriptor {
            trust_scope: TrustScope::User
        }
    );
}

#[test]
fn precedence_and_receipt_redaction_bounds_are_enforced() {
    let bad_precedence = Policy::validate(&PolicyDraft {
        precedence: 0,
        ..policy_draft(ALLOW_OUTCOME_LABEL)
    })
    .expect_err("out of range precedence should fail");
    assert_eq!(
        bad_precedence,
        Error::PrecedenceOutOfRange {
            min: MIN_PRECEDENCE,
            max: MAX_PRECEDENCE,
            actual: 0,
        }
    );

    let too_many_redactions = MAX_REDACTIONS.saturating_add(1);
    let markers = alloc::vec![REDACTION; too_many_redactions];
    let resource_hash = safe_hash();
    let policy_hash = safe_hash();
    let too_many = Receipt::new(ReceiptDraft {
        action: ACTION,
        resource_hash: &resource_hash,
        authority_status: crate::AuthorizationStatus::Allowed,
        authority_issue: crate::AuthorizationIssue::None,
        local_outcome: Some(Outcome::Allow),
        prompt_status: PromptStatus::NotRequired,
        policy_hash: &policy_hash,
        reason: "none",
        redaction_markers: markers.as_slice(),
    })
    .expect_err("too many redactions should fail");
    assert_eq!(
        too_many,
        Error::TooManyRedactions {
            max_items: MAX_REDACTIONS,
            actual_items: too_many_redactions,
        }
    );
}

#[test]
fn permission_receipt_leak_fixtures_are_rejected() {
    assert_eq!(
        validate_text(RAW_PROMPT_RECEIPT),
        Err(ReceiptLeak::RawPrompt)
    );
    assert_eq!(
        validate_text(CREDENTIAL_RECEIPT),
        Err(ReceiptLeak::Credential)
    );
    assert_eq!(
        validate_text(PROVIDER_RECEIPT),
        Err(ReceiptLeak::ProviderPayload)
    );
    assert_eq!(
        validate_text(TOOL_BODY_RECEIPT),
        Err(ReceiptLeak::RawToolBody)
    );
    assert_eq!(validate_text(PATH_RECEIPT), Err(ReceiptLeak::FullPath));
    assert_eq!(
        validate_text(OUTPUT_RECEIPT),
        Err(ReceiptLeak::UnboundedOutput)
    );
}

fn decision(policy: &Policy, token: &str, outcome: DecisionOutcome) -> Decision {
    Decision::new(DecisionDraft {
        action: ACTION,
        resource_label: RESOURCE,
        session_label: SESSION,
        policy_hash: policy.policy_hash(),
        decision_token: token,
        outcome,
    })
    .expect("prompt decision should validate")
}
