use super::*;

#[test]
fn allowed_model_path_matches() {
    let request = capability_request!(GuardedAction::Model, MODEL_RESOURCE, MODEL_ABILITY);
    let contract = policy_contract!(GuardedAction::Model, MODEL_PREFIX, MODEL_ABILITY);
    let grant = ucan_grant!(MODEL_PREFIX, MODEL_ABILITY);
    let caveats = [LocalCaveat::new(LocalPolicyLayer::Project, LocalCaveatDecision::Allow)];
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert!(production_allowed(context));
}

#[test]
fn allowed_sandbox_path_matches() {
    let request = capability_request!(GuardedAction::Exec, EXEC_RESOURCE, EXEC_ABILITY);
    let contract = policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY);
    let grant = ucan_grant!(EXEC_PREFIX, EXEC_ABILITY);
    let sandbox = SandboxConstraint::required(safe_hash(), SandboxState::Enforceable);
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &sandbox,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert!(production_allowed(context));
}

#[test]
fn contract_mismatch_counterexample_matches() {
    let request = capability_request!(GuardedAction::Filesystem, FILE_RESOURCE, OTHER_ABILITY);
    let contract = policy_contract!(GuardedAction::Filesystem, FILE_PREFIX, FILE_ABILITY);
    let grant = ucan_grant!(FILE_PREFIX, OTHER_ABILITY);
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &SandboxConstraint::NotRequired,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert_eq!(model_issue(context), AuthorizationIssue::ContractMismatch);
}

#[test]
fn local_denial_counterexample_matches() {
    let request = capability_request!(GuardedAction::Tool, TOOL_RESOURCE, TOOL_ABILITY);
    let contract = policy_contract!(GuardedAction::Tool, TOOL_PREFIX, TOOL_ABILITY);
    let grant = ucan_grant!(TOOL_PREFIX, TOOL_ABILITY);
    let caveats = [LocalCaveat::new(LocalPolicyLayer::Harness, LocalCaveatDecision::Deny)];
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &caveats,
        sandbox: &SandboxConstraint::NotRequired,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert_eq!(model_issue(context), AuthorizationIssue::LocalPolicyDenied);
}

#[test]
fn missing_grant_counterexample_matches() {
    let request = capability_request!(GuardedAction::Tool, TOOL_RESOURCE, TOOL_ABILITY);
    let contract = policy_contract!(GuardedAction::Tool, TOOL_PREFIX, TOOL_ABILITY);
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[],
        caveats: &[],
        sandbox: &SandboxConstraint::NotRequired,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert_eq!(model_issue(context), AuthorizationIssue::MissingUcanGrant);
}

#[test]
fn malformed_sandbox_counterexample_matches() {
    let request = capability_request!(GuardedAction::Exec, EXEC_RESOURCE, EXEC_ABILITY);
    let contract = policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY);
    let grant = ucan_grant!(EXEC_PREFIX, EXEC_ABILITY);
    let sandbox = SandboxConstraint::required(safe_hash(), SandboxState::Malformed);
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &sandbox,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert_eq!(model_issue(context), AuthorizationIssue::SandboxMalformed);
}

#[test]
fn unavailable_sandbox_counterexample_matches() {
    let request = capability_request!(GuardedAction::Exec, EXEC_RESOURCE, EXEC_ABILITY);
    let contract = policy_contract!(GuardedAction::Exec, EXEC_PREFIX, EXEC_ABILITY);
    let grant = ucan_grant!(EXEC_PREFIX, EXEC_ABILITY);
    let sandbox = SandboxConstraint::required(safe_hash(), SandboxState::Unavailable);
    let context = AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &sandbox,
    };

    assert_eq!(production_issue(context), model_issue(context));
    assert_eq!(model_issue(context), AuthorizationIssue::SandboxUnavailable);
}

fn model_issue(context: AuthorizationContext<'_>) -> AuthorizationIssue {
    super::model::issue(super::model::Input::from_context(&context))
}

fn production_issue(context: AuthorizationContext<'_>) -> AuthorizationIssue {
    authorize(context).issue()
}

fn production_allowed(context: AuthorizationContext<'_>) -> bool {
    authorize(context).is_allowed()
}
