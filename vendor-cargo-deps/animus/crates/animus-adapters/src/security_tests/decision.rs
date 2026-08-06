use super::*;

#[test]
fn benign_review_allows_with_safe_receipt() {
    let input = review_input(SecurityActionClass::ReadOnly, &[SAFE_ARGUMENT], &[CONTEXT_SUMMARY]);
    let scan = scan_review_input(&input);
    let review = advisory(SecurityAdvisoryOutcome::Allow);
    let decision = bind_security_review(&SecurityReviewBinding {
        input: &input,
        scan_report: &scan,
        advisory_reviews: core::slice::from_ref(&review),
        permissions: SecurityPermissionSnapshot::all_granted(),
        fail_policy: SecurityFailPolicy::default(),
    });

    assert!(decision.dispatch_allowed());
    assert_eq!(decision.status(), SecurityDecisionStatus::Allowed);
    assert_eq!(decision.issue(), SecurityDecisionIssue::None);
    assert_eq!(decision.receipt().status(), SecurityDecisionStatus::Allowed);
    assert_eq!(decision.receipt().finding_count(), 0);
    assert!(validate_security_review_receipt_text(&alloc::format!("{:?}", decision.receipt())).is_ok());
}

#[test]
fn reviewer_allow_cannot_grant_authority_bypass_sandbox_or_suppress_confirmation() {
    let input = review_input(SecurityActionClass::ReadOnly, &[SAFE_ARGUMENT], &[CONTEXT_SUMMARY]);
    let scan = scan_review_input(&input);
    let allow = advisory(SecurityAdvisoryOutcome::Allow);
    let missing_authority = with_permission(&input, &scan, &allow, denied_authority());
    let missing_sandbox = with_permission(&input, &scan, &allow, denied_sandbox());
    let missing_confirmation = with_permission(&input, &scan, &allow, denied_confirmation());

    assert_eq!(missing_authority.status(), SecurityDecisionStatus::Denied);
    assert_eq!(missing_authority.issue(), SecurityDecisionIssue::MissingAuthority);
    assert!(!missing_authority.dispatch_allowed());
    assert_eq!(missing_sandbox.status(), SecurityDecisionStatus::Denied);
    assert_eq!(missing_sandbox.issue(), SecurityDecisionIssue::SandboxDenied);
    assert_eq!(missing_confirmation.status(), SecurityDecisionStatus::RequiresConfirmation);
    assert_eq!(missing_confirmation.issue(), SecurityDecisionIssue::ConfirmationRequired);
    assert!(!missing_confirmation.dispatch_allowed());
}

#[test]
fn fail_behavior_denies_high_risk_outage_and_allows_low_risk_fallback() {
    let high_risk = review_input(SecurityActionClass::NetworkTransfer, &["unknown-upload-domain"], &[CONTEXT_SUMMARY]);
    let low_risk = review_input(SecurityActionClass::ReadOnly, &[SAFE_ARGUMENT], &[CONTEXT_SUMMARY]);
    let outage = SecurityScanReport::unavailable(SecurityScannerStatus::Timeout);
    let low_scan = scan_review_input(&low_risk);
    let unavailable_review = advisory(SecurityAdvisoryOutcome::Unavailable);
    let high_decision = bind_security_review(&SecurityReviewBinding {
        input: &high_risk,
        scan_report: &outage,
        advisory_reviews: &[],
        permissions: SecurityPermissionSnapshot::all_granted(),
        fail_policy: SecurityFailPolicy::default(),
    });
    let low_decision = bind_security_review(&SecurityReviewBinding {
        input: &low_risk,
        scan_report: &low_scan,
        advisory_reviews: core::slice::from_ref(&unavailable_review),
        permissions: SecurityPermissionSnapshot::all_granted(),
        fail_policy: SecurityFailPolicy::default(),
    });

    assert_eq!(high_decision.status(), SecurityDecisionStatus::Denied);
    assert_eq!(high_decision.issue(), SecurityDecisionIssue::ReviewUnavailable);
    assert!(low_decision.dispatch_allowed());
    assert_eq!(low_decision.issue(), SecurityDecisionIssue::None);
}

#[test]
fn blocking_finding_stops_dispatch_and_receipt_is_redacted() {
    let input = SecurityReviewInput::validate(&SecurityReviewInputDraft {
        raw_evidence: &[RAW_PROMPT_SENTINEL, CREDENTIAL_SENTINEL, PRIVATE_PATH_SENTINEL],
        ..draft(SecurityActionClass::FilesystemMutation, &["recursive-delete"], &[CONTEXT_SUMMARY])
    })
    .expect("dangerous input should validate with hashed raw evidence");
    let scan = scan_review_input(&input);
    let decision = bind_security_review(&SecurityReviewBinding {
        input: &input,
        scan_report: &scan,
        advisory_reviews: &[],
        permissions: SecurityPermissionSnapshot::all_granted(),
        fail_policy: SecurityFailPolicy::default(),
    });
    let rendered_receipt = alloc::format!("{:?}", decision.receipt());

    assert_eq!(decision.status(), SecurityDecisionStatus::Denied);
    assert_eq!(decision.issue(), SecurityDecisionIssue::BlockingFinding);
    assert!(!decision.dispatch_allowed());
    assert!(decision.receipt().finding_count() > 0);
    assert_eq!(decision.receipt().risk_class(), SecurityRiskClass::High);
    assert!(validate_security_review_receipt_text(&rendered_receipt).is_ok());
    assert!(!rendered_receipt.contains(RAW_PROMPT_SENTINEL));
    assert!(!rendered_receipt.contains(CREDENTIAL_SENTINEL));
    assert!(!rendered_receipt.contains(PRIVATE_PATH_SENTINEL));
}

#[test]
fn receipt_validation_rejects_leaking_text() {
    assert_eq!(validate_security_review_receipt_text(RAW_PROMPT_SENTINEL), Err(SecurityReceiptLeak::RawPrompt));
    assert_eq!(validate_security_review_receipt_text(CREDENTIAL_SENTINEL), Err(SecurityReceiptLeak::Credential));
    assert_eq!(validate_security_review_receipt_text(PRIVATE_PATH_SENTINEL), Err(SecurityReceiptLeak::PrivatePath));
}

fn with_permission(
    input: &SecurityReviewInput,
    scan: &SecurityScanReport,
    review: &SecurityAdvisoryReview,
    permissions: SecurityPermissionSnapshot,
) -> SecurityDecision {
    bind_security_review(&SecurityReviewBinding {
        input,
        scan_report: scan,
        advisory_reviews: core::slice::from_ref(review),
        permissions,
        fail_policy: SecurityFailPolicy::default(),
    })
}

fn denied_authority() -> SecurityPermissionSnapshot {
    SecurityPermissionSnapshot {
        authority: SecurityPermissionGate::Denied,
        ..SecurityPermissionSnapshot::all_granted()
    }
}

fn denied_sandbox() -> SecurityPermissionSnapshot {
    SecurityPermissionSnapshot {
        sandbox: SecurityPermissionGate::Denied,
        ..SecurityPermissionSnapshot::all_granted()
    }
}

fn denied_confirmation() -> SecurityPermissionSnapshot {
    SecurityPermissionSnapshot {
        confirmation: SecurityPermissionGate::Denied,
        ..SecurityPermissionSnapshot::all_granted()
    }
}
