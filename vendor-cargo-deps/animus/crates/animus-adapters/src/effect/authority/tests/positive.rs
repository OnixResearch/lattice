use super::*;

#[test]
fn authorized_model_action_records_safe_receipt() {
    let request = capability_request!(GuardedAction::Model, MODEL_RESOURCE, MODEL_ABILITY);
    let contract = policy_contract!(GuardedAction::Model, MODEL_PREFIX, MODEL_ABILITY);
    let grant = ucan_grant!(MODEL_PREFIX, MODEL_ABILITY);
    let caveats = [LocalCaveat::new(
        LocalPolicyLayer::Project,
        LocalCaveatDecision::Allow,
    )];
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    });

    assert!(receipt.is_allowed());
    assert_eq!(receipt.status(), AuthorizationStatus::Allowed);
    assert_eq!(receipt.issue(), AuthorizationIssue::None);
    assert_eq!(receipt.action(), GuardedAction::Model);
    assert_eq!(receipt.contract(), CONTRACT);
    assert_eq!(receipt.ability(), MODEL_ABILITY);
    assert_ne!(receipt.resource_hash(), MODEL_RESOURCE);
    assert_eq!(receipt.checked_grants(), 1);
    assert_eq!(receipt.caveats().allows(), 1);
    assert!(receipt.sandbox_hash().is_none());
    assert!(!std::format!("{receipt:?}").contains(MODEL_RESOURCE));
}

#[test]
fn policy_fixtures_cover_guarded_action_taxonomy() {
    let fixtures = [
        policy_contract!(GuardedAction::Model, MODEL_PREFIX, MODEL_ABILITY),
        policy_contract!(GuardedAction::Tool, TOOL_PREFIX, TOOL_ABILITY),
        policy_contract!(GuardedAction::Filesystem, FILE_PREFIX, FILE_ABILITY),
        policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY),
        policy_contract!(GuardedAction::Network, NETWORK_PREFIX, NETWORK_ABILITY),
        policy_contract!(GuardedAction::Steel, STEEL_PREFIX, STEEL_ABILITY),
        policy_contract!(
            GuardedAction::AgentRole,
            AGENT_ROLE_PREFIX,
            AGENT_ROLE_ABILITY
        ),
    ];

    assert_eq!(fixtures.len(), GUARDED_ACTION_COUNT);
    assert!(
        fixtures
            .iter()
            .all(|fixture| !fixture.policy_hash().is_empty())
    );
    assert!(
        fixtures
            .iter()
            .all(|fixture| fixture.contract() == CONTRACT)
    );
    assert!(fixtures.iter().all(|fixture| !fixture.ability().is_empty()));
}

#[test]
fn local_ask_with_session_grant_authorizes_bounded_request() {
    let request = capability_request!(GuardedAction::Tool, TOOL_RESOURCE, TOOL_ABILITY);
    let contract = policy_contract!(GuardedAction::Tool, TOOL_PREFIX, TOOL_ABILITY);
    let grant = ucan_grant!(TOOL_PREFIX, TOOL_ABILITY);
    let caveats = [LocalCaveat::new(
        LocalPolicyLayer::Session,
        LocalCaveatDecision::Ask,
    )];
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    });

    assert!(receipt.is_allowed());
    assert_eq!(receipt.caveats().asks(), 1);
    assert_eq!(receipt.checked_grants(), 1);
}

#[test]
fn sandbox_constrained_dispatch_allows_after_grant_checks() {
    let request = capability_request!(GuardedAction::Exec, EXEC_RESOURCE, EXEC_ABILITY);
    let contract = policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY);
    let grant = ucan_grant!(EXEC_PREFIX, EXEC_ABILITY);
    let sandbox = SandboxConstraint::required(safe_hash(), SandboxState::Enforceable);
    let receipt = authorize(AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &sandbox,
    });

    assert!(receipt.is_allowed());
    assert_eq!(
        receipt
            .sandbox_hash()
            .expect("sandbox hash should exist")
            .as_str(),
        VALID_HASH
    );
}
