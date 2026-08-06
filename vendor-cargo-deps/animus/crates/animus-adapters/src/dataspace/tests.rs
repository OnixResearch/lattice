use super::AdapterService;

const TENANT: &str = "tenant:local";
const WORKSPACE_A: &str = "workspace:a";
const WORKSPACE_B: &str = "workspace:b";
const SESSION: &str = "session:main";
const BRANCH: &str = "branch:main";
const FACET: &str = "facet:loop";
const ACTOR_MAIN: &str = "agent:main";
const ACTOR_OTHER: &str = "agent:other";
const OWNER: &str = "facet:next";
const ASSERTION_RUNTIME: &str = "assertion:runtime";
const ASSERTION_OTHER: &str = "assertion:other";
const EVENT_CANCEL: &str = "event:cancel";
const EVENT_TRANSFER: &str = "event:transfer";
const BAD_VALUE: &str = "raw_private_payload";
const SAFE_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const ACTOR_KIND_AGENT: &str = "agent";
const ASSERTION_KIND_RUNTIME_EVENT: &str = "runtime_event";
const ASSERTION_STATUS_ACTIVE: &str = "active";
const EVENT_KIND_CANCELLED: &str = "cancelled";
const EVENT_KIND_OWNERSHIP_TRANSFERRED: &str = "ownership_transferred";
const REDACTION_PROMPT_OMITTED: &str = "prompt_omitted";
const REDACTION_PROVIDER_PAYLOAD_OMITTED: &str = "provider_payload_omitted";
const REDACTION_CREDENTIAL_OMITTED: &str = "credential_omitted";
const REDACTION_LOGS_BOUNDED: &str = "logs_bounded";
const ISSUE_RAW_PAYLOAD_REQUESTED: &str = "DATASPACE_RAW_PAYLOAD_REQUESTED";
const ISSUE_CROSS_WORKSPACE_SCOPE: &str = "DATASPACE_CROSS_WORKSPACE_SCOPE";
const ISSUE_INVALID_FIELD: &str = "DATASPACE_INVALID_FIELD";
const ITEM_COUNT: u32 = 3;
const EXPIRES_EARLY: u64 = 10;
const EXPIRES_LATE: u64 = 20;
const CLEANUP_TICK: u64 = 10;
const FIRST_SEQUENCE: u64 = 0;
const EXPECTED_ONE: usize = 1;
const EXPECTED_TWO: usize = 2;

#[derive(Clone, Copy)]
struct ScopeFixture {
    workspace_id: &'static str,
    actor_id: &'static str,
}

const MAIN_SCOPE: ScopeFixture = ScopeFixture {
    workspace_id: WORKSPACE_A,
    actor_id: ACTOR_MAIN,
};
const OTHER_WORKSPACE_SCOPE: ScopeFixture = ScopeFixture {
    workspace_id: WORKSPACE_B,
    actor_id: ACTOR_OTHER,
};
const OTHER_ACTOR_SCOPE: ScopeFixture = ScopeFixture {
    workspace_id: WORKSPACE_A,
    actor_id: ACTOR_OTHER,
};

fn agent_scope(fixture: ScopeFixture) -> super::Scope {
    super::Scope::new(super::ScopeDraft {
        tenant_id: TENANT,
        workspace_id: fixture.workspace_id,
        session_id: SESSION,
        branch_id: BRANCH,
        facet_id: FACET,
        actor_kind: ACTOR_KIND_AGENT,
        actor_id: fixture.actor_id,
    })
    .expect("valid scope should parse")
}

fn runtime_assertion(stable_id: &str, scope: super::Scope, lease_expires_at: Option<u64>) -> super::Assertion {
    super::Assertion::new(super::AssertionDraft {
        stable_id,
        kind: ASSERTION_KIND_RUNTIME_EVENT,
        status: ASSERTION_STATUS_ACTIVE,
        scope,
        redactions: &[
            REDACTION_PROMPT_OMITTED,
            REDACTION_PROVIDER_PAYLOAD_OMITTED,
            REDACTION_CREDENTIAL_OMITTED,
            REDACTION_LOGS_BOUNDED,
        ],
        content_hash: Some(SAFE_HASH),
        item_count: ITEM_COUNT,
        lease_expires_at,
    })
    .expect("valid assertion should parse")
}

fn runtime_subscription(scope: super::Scope) -> super::Subscription {
    super::Subscription::new(super::SubscriptionDraft {
        scope,
        kinds: &[ASSERTION_KIND_RUNTIME_EVENT],
        include_raw_payloads: false,
        cross_workspace: false,
    })
    .expect("valid subscription should parse")
}

fn cancellation_event(scope: super::Scope) -> super::OrchestrationEvent {
    super::OrchestrationEvent::new(super::OrchestrationEventDraft {
        stable_id: EVENT_CANCEL,
        kind: EVENT_KIND_CANCELLED,
        scope,
        target_owner: None,
    })
    .expect("valid cancellation event should parse")
}

#[test]
fn adapter_contract_accepts_bounded_publish_subscribe() {
    let scope = agent_scope(MAIN_SCOPE);
    let assertion = runtime_assertion(ASSERTION_RUNTIME, scope.clone(), None);
    let mut dataspace = super::fake::Backend::new();

    dataspace.publish(&assertion).expect("fake publish should not fail");
    let matches = dataspace.subscribe(&runtime_subscription(scope)).expect("fake subscription should not fail");

    assert_eq!(matches.len(), EXPECTED_ONE);
    assert_eq!(matches[0].stable_id(), ASSERTION_RUNTIME);
    assert_eq!(matches[0].kind(), super::AssertionKind::RuntimeEvent);
    assert_eq!(matches[0].status(), super::AssertionStatus::Active);
    assert_eq!(matches[0].item_count(), ITEM_COUNT);
    assert_eq!(matches[0].content_hash().map(crate::SafeHash::as_str), Some(SAFE_HASH));
    assert!(matches[0].redactions().contains(&super::AssertionRedaction::ProviderPayloadOmitted));
    assert_eq!(dataspace.assertions()[0].sequence(), FIRST_SEQUENCE);
}

#[test]
fn scoped_cancellation_only_updates_matching_scope() {
    let affected_scope = agent_scope(MAIN_SCOPE);
    let unaffected_scope = agent_scope(OTHER_WORKSPACE_SCOPE);
    let mut dataspace = super::fake::Backend::new();
    dataspace.publish_assertion(runtime_assertion(ASSERTION_RUNTIME, affected_scope.clone(), None));
    dataspace.publish_assertion(runtime_assertion(ASSERTION_OTHER, unaffected_scope, None));

    let changed = dataspace.apply_event(&cancellation_event(affected_scope));

    assert_eq!(changed, EXPECTED_ONE);
    assert_eq!(dataspace.assertions().len(), EXPECTED_TWO);
    assert_eq!(dataspace.assertions()[0].assertion().status(), super::AssertionStatus::Cancelled);
    assert_eq!(dataspace.assertions()[1].assertion().status(), super::AssertionStatus::Active);
}

#[test]
fn stale_cleanup_is_deterministic() {
    let mut dataspace = super::fake::Backend::new();
    dataspace.publish_assertion(runtime_assertion(ASSERTION_RUNTIME, agent_scope(MAIN_SCOPE), Some(EXPIRES_EARLY)));
    dataspace.publish_assertion(runtime_assertion(ASSERTION_OTHER, agent_scope(OTHER_ACTOR_SCOPE), Some(EXPIRES_LATE)));

    let removed = dataspace.cleanup_stale(CLEANUP_TICK);

    assert_eq!(removed, EXPECTED_ONE);
    assert_eq!(dataspace.assertions().len(), EXPECTED_ONE);
    assert_eq!(dataspace.assertions()[0].assertion().stable_id(), ASSERTION_OTHER);
}

#[test]
fn ownership_transfer_event_carries_target_owner() {
    let scope = agent_scope(MAIN_SCOPE);
    let event = super::OrchestrationEvent::new(super::OrchestrationEventDraft {
        stable_id: EVENT_TRANSFER,
        kind: EVENT_KIND_OWNERSHIP_TRANSFERRED,
        scope,
        target_owner: Some(OWNER),
    })
    .expect("ownership transfer with target owner should parse");

    assert_eq!(event.kind(), super::OrchestrationEventKind::OwnershipTransferred);
    assert_eq!(event.target_owner(), Some(OWNER));
}

#[test]
fn subscription_rejects_raw_payload_query() {
    let issue = super::Subscription::new(super::SubscriptionDraft {
        scope: agent_scope(MAIN_SCOPE),
        kinds: &[ASSERTION_KIND_RUNTIME_EVENT],
        include_raw_payloads: true,
        cross_workspace: false,
    })
    .expect_err("raw-payload subscriptions should fail");

    assert_eq!(issue, super::SubscriptionIssue::RawPayloadRequested);
    assert_eq!(issue.as_str(), ISSUE_RAW_PAYLOAD_REQUESTED);
}

#[test]
fn subscription_rejects_cross_workspace_query() {
    let issue = super::Subscription::new(super::SubscriptionDraft {
        scope: agent_scope(MAIN_SCOPE),
        kinds: &[ASSERTION_KIND_RUNTIME_EVENT],
        include_raw_payloads: false,
        cross_workspace: true,
    })
    .expect_err("cross-workspace subscriptions should fail");

    assert_eq!(issue, super::SubscriptionIssue::CrossWorkspaceScope);
    assert_eq!(issue.as_str(), ISSUE_CROSS_WORKSPACE_SCOPE);
}

#[test]
fn subscription_rejects_unsupported_query_shape() {
    let issue = super::Subscription::new(super::SubscriptionDraft {
        scope: agent_scope(MAIN_SCOPE),
        kinds: &[BAD_VALUE],
        include_raw_payloads: false,
        cross_workspace: false,
    })
    .expect_err("unsupported query kind should fail");

    assert_eq!(issue.as_str(), ISSUE_INVALID_FIELD);
    match issue {
        super::SubscriptionIssue::InvalidField {
            error: crate::AdapterContractError::UnsupportedDataspaceValue { field, value },
        } => {
            assert_eq!(field, "dataspace_subscription_kind");
            assert_eq!(value, alloc::string::String::from(BAD_VALUE));
        }
        other => panic!("unexpected issue: {other:?}"),
    }
}

#[test]
fn assertion_rejects_empty_redactions() {
    let error = super::Assertion::new(super::AssertionDraft {
        stable_id: ASSERTION_RUNTIME,
        kind: ASSERTION_KIND_RUNTIME_EVENT,
        status: ASSERTION_STATUS_ACTIVE,
        scope: agent_scope(MAIN_SCOPE),
        redactions: &[],
        content_hash: Some(SAFE_HASH),
        item_count: ITEM_COUNT,
        lease_expires_at: None,
    })
    .expect_err("assertions without redaction evidence should fail");

    assert_eq!(error, crate::AdapterContractError::EmptyCollection {
        field: "dataspace_assertion_redactions",
    });
}

#[test]
fn ownership_transfer_rejects_missing_target_owner() {
    let error = super::OrchestrationEvent::new(super::OrchestrationEventDraft {
        stable_id: EVENT_TRANSFER,
        kind: EVENT_KIND_OWNERSHIP_TRANSFERRED,
        scope: agent_scope(MAIN_SCOPE),
        target_owner: None,
    })
    .expect_err("ownership transfer without target owner should fail");

    assert_eq!(error, crate::AdapterContractError::EmptyField {
        field: "dataspace_event_target_owner",
    });
}
