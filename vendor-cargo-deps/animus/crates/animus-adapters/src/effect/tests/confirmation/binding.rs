use super::super::super::*;

#[test]
fn mismatched_and_stale_approvals_are_rejected() {
    let request = super::support::protected_request(super::support::SUMMARY);
    let other_policy = SafeHash::new(super::support::OTHER_POLICY_HASH).expect("other policy hash should parse");

    let mismatched_id = super::support::evaluated_issue(
        &request,
        ConfirmationBinding::new(ConfirmationBindingDraft {
            request_id: super::support::OTHER_REQUEST_ID,
            action_kind: request.action_kind(),
            resource: request.resource(),
            ability: request.ability(),
            policy_hash: request.policy_hash(),
            expiry: request.expiry(),
        })
        .expect("mismatched request binding should parse"),
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );
    let mismatched_resource = super::support::evaluated_issue(
        &request,
        ConfirmationBinding::new(ConfirmationBindingDraft {
            request_id: request.request_id(),
            action_kind: request.action_kind(),
            resource: super::support::OTHER_RESOURCE_LABEL,
            ability: request.ability(),
            policy_hash: request.policy_hash(),
            expiry: request.expiry(),
        })
        .expect("mismatched resource binding should parse"),
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );
    let mismatched_policy = super::support::evaluated_issue(
        &request,
        ConfirmationBinding::new(ConfirmationBindingDraft {
            request_id: request.request_id(),
            action_kind: request.action_kind(),
            resource: request.resource(),
            ability: request.ability(),
            policy_hash: &other_policy,
            expiry: request.expiry(),
        })
        .expect("mismatched policy binding should parse"),
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );
    let stale = super::support::evaluated_issue(
        &request,
        ConfirmationBinding::from_request(&request),
        ConfirmationTick::new(super::support::STALE_TICK_VALUE),
    );

    assert_eq!(mismatched_id, ConfirmationIssue::MismatchedRequest);
    assert_eq!(mismatched_resource, ConfirmationIssue::MismatchedResource);
    assert_eq!(mismatched_policy, ConfirmationIssue::MismatchedPolicy);
    assert_eq!(stale, ConfirmationIssue::StaleDecision);
}
