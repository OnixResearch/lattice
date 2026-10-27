use crate::support::*;

fn target_request(action: EnforcementAction) -> EnforcementRequest {
    EnforcementRequest::new(
        action.contract_id(),
        action.resource_prefix().to_owned() + "target/abc",
        action.ability(),
    )
}

fn state_sync_request(action: EnforcementAction) -> EnforcementRequest {
    EnforcementRequest::new(
        action.contract_id(),
        action.resource_prefix().to_owned() + "publish/abc",
        action.ability(),
    )
}

#[test]
fn all_onixos_actions_have_distinct_reviewed_bindings() {
    let actions = [
        EnforcementAction::OnixosTargetCandidateActivation,
        EnforcementAction::OnixosTargetGenerationPersistence,
        EnforcementAction::OnixosTargetRollback,
        EnforcementAction::OnixosStateSyncPublish,
        EnforcementAction::OnixosStateSyncDelete,
        EnforcementAction::OnixosStateSyncAdmin,
    ];
    let profiles = actions
        .iter()
        .map(|action| action.operation_profile_id())
        .collect::<Vec<_>>();
    for (index, left) in profiles.iter().enumerate() {
        for right in &profiles[index + 1..] {
            assert_ne!(
                left, right,
                "each target action must map to a distinct profile"
            );
        }
    }
    // Each action binds its exact request without a generic action constructor.
    for action in actions {
        let request = if action.resource_prefix().starts_with("onix://") {
            state_sync_request(action)
        } else {
            target_request(action)
        };
        let binding =
            bind_enforcement_action(action, &request).expect("exact target request must bind");
        assert_eq!(binding.profile(), action.operation_profile_id());
        assert_eq!(request.contract(), action.contract_id());
        assert_eq!(request.ability(), action.ability());
    }
}

#[test]
fn onixos_action_substitutions_fail_before_replay() {
    for action in [
        EnforcementAction::OnixosTargetCandidateActivation,
        EnforcementAction::OnixosTargetGenerationPersistence,
        EnforcementAction::OnixosTargetRollback,
        EnforcementAction::OnixosStateSyncPublish,
        EnforcementAction::OnixosStateSyncDelete,
        EnforcementAction::OnixosStateSyncAdmin,
    ] {
        // Wrong contract: same ability, wrong contract id.
        let wrong_contract = target_request(action);
        assert_eq!(
            bind_enforcement_action(
                EnforcementAction::DocumentWrite,
                &EnforcementRequest::new(
                    wrong_contract.contract(),
                    wrong_contract.resource(),
                    action.ability(),
                ),
            ),
            Err(UcanEnforcementReason::OperationMismatch),
            "wrong contract must fail: {action:?}"
        );
        // Wrong ability for the same contract.
        let wrong_ability = EnforcementRequest::new(
            action.contract_id(),
            action.resource_prefix().to_owned() + "target/abc",
            "document/read",
        );
        assert_eq!(
            bind_enforcement_action(action, &wrong_ability),
            Err(UcanEnforcementReason::OperationMismatch),
            "wrong ability must fail: {action:?}"
        );
        // Resource outside the reviewed prefix must fail closed.
        let outside_prefix = EnforcementRequest::new(
            action.contract_id(),
            "did:key:z6Mkother/abc",
            action.ability(),
        );
        assert_eq!(
            bind_enforcement_action(action, &outside_prefix),
            Err(UcanEnforcementReason::ScopeDenied),
            "resource outside reviewed prefix must fail: {action:?}"
        );
    }
}

#[test]
fn state_sync_publish_authority_does_not_attenuate_delete_or_admin() {
    let publish = EnforcementAction::OnixosStateSyncPublish;
    let request = state_sync_request(publish);

    assert_eq!(
        bind_enforcement_action(EnforcementAction::OnixosStateSyncDelete, &request),
        Err(UcanEnforcementReason::OperationMismatch)
    );
    assert_eq!(
        bind_enforcement_action(EnforcementAction::OnixosStateSyncAdmin, &request),
        Err(UcanEnforcementReason::OperationMismatch)
    );
    assert_eq!(
        bind_enforcement_action(
            EnforcementAction::OnixosTargetRollback,
            &EnforcementRequest::new(
                publish.contract_id(),
                publish.resource_prefix().to_owned() + "publish/abc",
                publish.ability(),
            ),
        ),
        Err(UcanEnforcementReason::OperationMismatch)
    );
}
