use super::super::*;
use super::support::*;

#[test]
fn recipe_backed_request_accepts_explicit_admitted_recipe() {
    let recipe = admitted_recipe();
    let delegation = SubagentDelegation::validate(&valid_delegation(&recipe))
        .expect("delegation should validate");

    assert_eq!(delegation.id(), CHILD_ID);
    assert_eq!(delegation.recipe_id(), Some(RECIPE_ID));
    assert_eq!(delegation.parameter_bindings()[0], PARAM_TOPIC);
    assert_eq!(delegation.tool_ceilings()[0], TOOL_READ);
    assert_eq!(
        delegation.return_mode(),
        SubagentReturnMode::StatusPlusSummary
    );
}

#[test]
fn explicit_request_denies_automatic_recursive_untrusted_and_unapproved() {
    let admitted = admitted_recipe();
    let pending = pending_recipe();

    assert_eq!(
        SubagentDelegation::validate(&SubagentDelegationDraft {
            delegation_source: SOURCE_AUTOMATIC,
            ..valid_delegation(&admitted)
        }),
        Err(SubagentError::ImplicitDelegationDenied)
    );
    assert_eq!(
        SubagentDelegation::validate(&SubagentDelegationDraft {
            recursive: true,
            ..valid_delegation(&admitted)
        }),
        Err(SubagentError::RecursiveDelegationDenied)
    );
    assert_eq!(
        SubagentDelegation::validate(&SubagentDelegationDraft {
            recipe_trust: Some(&pending),
            ..valid_delegation(&pending)
        }),
        Err(SubagentError::UntrustedRecipeDenied)
    );
    assert_eq!(
        SubagentDelegation::validate(&SubagentDelegationDraft {
            delegation_source: SOURCE_ADMITTED_AUTOMATION,
            automation_admitted: false,
            ..valid_delegation(&admitted)
        }),
        Err(SubagentError::AutomationDelegationDenied)
    );
    assert_eq!(
        SubagentDelegation::validate(&SubagentDelegationDraft {
            max_turns: ZERO_TURNS,
            ..valid_delegation(&admitted)
        }),
        Err(SubagentError::InvalidTurnCeiling)
    );
}

#[test]
fn capability_intersection_allows_only_everywhere_admitted_tools() {
    let resolution = resolve_subagent_capabilities(SubagentCapabilityInput {
        parent_policy: &[TOOL_READ, TOOL_WRITE, SECRET_READ],
        recipe_or_profile: &[TOOL_READ],
        request: &[TOOL_READ, TOOL_WRITE, SECRET_READ],
        workspace_scope: &[TOOL_READ],
        host_admission: &[TOOL_READ],
    })
    .expect("capabilities should parse");

    assert_eq!(resolution.effective(), &[Text::from(TOOL_READ)]);
    assert_eq!(resolution.denied().len(), 2);
    assert!(
        resolution
            .denied()
            .iter()
            .any(|denial| denial.capability() == TOOL_WRITE)
    );
    assert!(
        resolution
            .denied()
            .iter()
            .any(|denial| denial.capability() == SECRET_READ)
    );
}
