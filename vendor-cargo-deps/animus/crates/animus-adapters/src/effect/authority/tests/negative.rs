use super::*;

#[test]
fn capability_request_rejects_malformed_fields() {
    let error = CapabilityRequest::new(GuardedAction::Model, CONTRACT, EMPTY_FIELD, MODEL_ABILITY)
        .expect_err("empty resource should fail");

    assert!(matches!(error, crate::AdapterContractError::EmptyField { .. }));
}

#[test]
fn local_allow_without_ucan_grant_denies_before_dispatch() {
    let request = capability_request!(GuardedAction::Tool, TOOL_RESOURCE, TOOL_ABILITY);
    let contract = policy_contract!(GuardedAction::Tool, TOOL_PREFIX, TOOL_ABILITY);
    let caveats = [LocalCaveat::new(LocalPolicyLayer::User, LocalCaveatDecision::Allow)];
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    });

    assert!(!receipt.is_allowed());
    assert_eq!(receipt.issue(), AuthorizationIssue::MissingUcanGrant);
    assert_eq!(receipt.checked_grants(), 0);
}

#[test]
fn contract_resource_or_ability_mismatch_denies() {
    let request = capability_request!(GuardedAction::Filesystem, FILE_RESOURCE, OTHER_ABILITY);
    let contract = policy_contract!(GuardedAction::Filesystem, FILE_PREFIX, FILE_ABILITY);
    let grant = ucan_grant!(FILE_PREFIX, OTHER_ABILITY);
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &SandboxConstraint::NotRequired,
    });

    assert!(!receipt.is_allowed());
    assert_eq!(receipt.issue(), AuthorizationIssue::ContractMismatch);
}

#[test]
fn local_policy_denial_fails_closed() {
    let request = capability_request!(GuardedAction::Network, NETWORK_RESOURCE, NETWORK_ABILITY);
    let contract = policy_contract!(GuardedAction::Network, NETWORK_PREFIX, NETWORK_ABILITY);
    let grant = ucan_grant!(NETWORK_PREFIX, NETWORK_ABILITY);
    let caveats = [LocalCaveat::new(LocalPolicyLayer::Harness, LocalCaveatDecision::Deny)];
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    });

    assert!(!receipt.is_allowed());
    assert_eq!(receipt.issue(), AuthorizationIssue::LocalPolicyDenied);
}

#[test]
fn malformed_or_unavailable_sandbox_denies_after_grant_checks() {
    let request = capability_request!(GuardedAction::Exec, EXEC_RESOURCE, EXEC_ABILITY);
    let contract = policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY);
    let grant = ucan_grant!(EXEC_PREFIX, EXEC_ABILITY);
    let malformed = SandboxConstraint::required(safe_hash(), SandboxState::Malformed);
    let unavailable = SandboxConstraint::required(safe_hash(), SandboxState::Unavailable);

    let malformed_receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: core::slice::from_ref(&grant),
        caveats: &[],
        sandbox: &malformed,
    });
    let unavailable_receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &unavailable,
    });

    assert_eq!(malformed_receipt.issue(), AuthorizationIssue::SandboxMalformed);
    assert_eq!(unavailable_receipt.issue(), AuthorizationIssue::SandboxUnavailable);
}

#[test]
fn receipt_validation_rejects_raw_material() {
    assert_eq!(validate_authorization_receipt_text(LEAKING_RECEIPT_TEXT), Err(AuthorizationLeak::RawPrompt));
    assert!(validate_authorization_receipt_text(SAFE_RECEIPT_TEXT).is_ok());
}
